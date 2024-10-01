use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::modules::fiat_shamir::eval_from_partial_evals;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir1_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir1_result")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    let fiat_shamir2_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir2_result")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `fiat_shamir1_result_hash`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir1_result_hash,
        cache.get("fiat_shamir1_result").unwrap(),
    )?;
    assert_eq!(res.len(), 32);

    let mut res = res.into_iter();

    let z_var = res.next().unwrap();
    let alpha_var = res.next().unwrap();
    let composition_fold_random_coeff_var = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();

    let mut trace_oods_values_vars = vec![];
    let mut interaction_oods_values_vars = vec![];
    let mut constant_oods_values_vars = vec![];
    let mut composition_oods_raw_values_vars = vec![];

    for _ in 0..4 {
        trace_oods_values_vars.push(res.next().unwrap());
    }
    for _ in 0..12 {
        interaction_oods_values_vars.push(res.next().unwrap());
    }
    for _ in 0..4 {
        constant_oods_values_vars.push(res.next().unwrap());
    }
    for _ in 0..4 {
        composition_oods_raw_values_vars.push(res.next().unwrap());
    }

    assert!(res.next().is_none());

    // Step 1: find the algebra-related OODS values
    let a_val_var = trace_oods_values_vars[1];
    let b_val_var = trace_oods_values_vars[2];
    let c_val_var = trace_oods_values_vars[3];
    let op_val_var = constant_oods_values_vars[3];

    // Step 2: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 3: compute a_val + b_val
    let a_plus_b_val_var = dsl.execute("qm31_add", &[a_val_var, b_val_var])?[0];

    // Step 4: compute a_val * b_val
    let a_limbs_var = dsl.execute("qm31_to_limbs", &[a_val_var])?[0];
    let b_limbs_var = dsl.execute("qm31_to_limbs", &[b_val_var])?[0];
    let a_times_b_val_var = dsl.execute("qm31_limbs_mul", &[table, a_limbs_var, b_limbs_var])?[0];

    // Step 5: compute op * (a_val + b_val)
    let op_limbs_var = dsl.execute("qm31_to_limbs", &[op_val_var])?[0];
    let a_plus_b_limbs_var = dsl.execute("qm31_to_limbs", &[a_plus_b_val_var])?[0];
    let term1_var = dsl.execute("qm31_limbs_mul", &[table, op_limbs_var, a_plus_b_limbs_var])?[0];

    // Step 6: compute 1 - op
    let neg_op_val_var = dsl.execute("qm31_neg", &[op_val_var])?[0];
    let one_minus_op_val_var = dsl.execute("qm31_1add", &[neg_op_val_var])?[0];
    let one_minus_op_limbs_var = dsl.execute("qm31_to_limbs", &[one_minus_op_val_var])?[0];

    // Step 7: compute (1 - op) * a_val * b_val
    let a_times_b_limbs_var = dsl.execute("qm31_to_limbs", &[a_times_b_val_var])?[0];
    let term2_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, one_minus_op_limbs_var, a_times_b_limbs_var],
    )?[0];

    // Step 8: get the sum
    let mut sum_var = dsl.execute("qm31_sub", &[c_val_var, term1_var])?[0];
    sum_var = dsl.execute("qm31_sub", &[sum_var, term2_var])?[0];

    // Step 9: compute the square of the composition_fold_random_coeff
    let composition_fold_random_coeff_limbs_var =
        dsl.execute("qm31_to_limbs", &[composition_fold_random_coeff_var])?[0];
    let composition_fold_random_coeff_squared_var = dsl.execute(
        "qm31_limbs_mul",
        &[
            table,
            composition_fold_random_coeff_limbs_var,
            composition_fold_random_coeff_limbs_var,
        ],
    )?[0];

    // Step 10: multiply the sum with the randomizer
    let sum_limbs_var = dsl.execute("qm31_to_limbs", &[sum_var])?[0];
    let composition_fold_random_coeff_squared_limbs_var = dsl.execute(
        "qm31_to_limbs",
        &[composition_fold_random_coeff_squared_var],
    )?[0];
    let sum_multiplied_by_randomizer = dsl.execute(
        "qm31_limbs_mul",
        &[
            table,
            sum_limbs_var,
            composition_fold_random_coeff_squared_limbs_var,
        ],
    )?[0];

    // c_val - op * (a_val + b_val) - (E::F::one() - op) * a_val * b_val

    // denominator_1 = a_wire + alpha * a_val - z
    // denominator_2 = b_wire + alpha * b_val - z
    // num_aggregated = denominator_1 + denominator_2
    // denominator_aggregated = denominator_1 * denominator_2

    // Step 1: push the table
    let alpha_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_var])?[0];
    let a_val_var = trace_oods_values_vars[1];
    let a_limbs_var = dsl.execute("qm31_to_limbs", &[a_val_var])?[0];

    let alpha_times_a_val_var =
        dsl.execute("qm31_limbs_mul", &[table, alpha_limbs_var, a_limbs_var])?[0];

    let a_wire_var = constant_oods_values_vars[0];
    let mut denominator_1_var = dsl.execute("qm31_add", &[a_wire_var, alpha_times_a_val_var])?[0];
    denominator_1_var = dsl.execute("qm31_sub", &[denominator_1_var, z_var])?[0];

    let b_val_var = trace_oods_values_vars[2];
    let b_limbs_var = dsl.execute("qm31_to_limbs", &[b_val_var])?[0];

    let alpha_times_b_val_var =
        dsl.execute("qm31_limbs_mul", &[table, alpha_limbs_var, b_limbs_var])?[0];

    let b_wire_var = constant_oods_values_vars[1];
    let mut denominator_2_var = dsl.execute("qm31_add", &[b_wire_var, alpha_times_b_val_var])?[0];
    denominator_2_var = dsl.execute("qm31_sub", &[denominator_2_var, z_var])?[0];

    let num_aggregated_var = dsl.execute("qm31_add", &[denominator_1_var, denominator_2_var])?[0];
    let denominator_1_limbs_var = dsl.execute("qm31_to_limbs", &[denominator_1_var])?[0];
    let denominator_2_limbs_var = dsl.execute("qm31_to_limbs", &[denominator_2_var])?[0];

    let denom_aggregated_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, denominator_1_limbs_var, denominator_2_limbs_var],
    )?[0];

    // a_b_logup_var * denom - num

    let a_b_logup_var = eval_from_partial_evals(
        &mut dsl,
        interaction_oods_values_vars[0],
        interaction_oods_values_vars[1],
        interaction_oods_values_vars[2],
        interaction_oods_values_vars[3],
    )?;
    let a_b_logup_limbs_var = dsl.execute("qm31_to_limbs", &[a_b_logup_var])?[0];
    let denom_aggregated_limbs_var = dsl.execute("qm31_to_limbs", &[denom_aggregated_var])?[0];
    let a_b_logup_times_denom_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, denom_aggregated_limbs_var, a_b_logup_limbs_var],
    )?[0];

    let constraint_logup_ab_without_randomizer_var =
        dsl.execute("qm31_sub", &[a_b_logup_times_denom_var, num_aggregated_var])?[0];

    let list_constraint_logup_ab_result = vec![
        sum_multiplied_by_randomizer,
        constraint_logup_ab_without_randomizer_var,
    ];
    let (pack_constraint_logup_ab_result_hash, pack_constraint_logup_ab_result) =
        zip_elements(&mut dsl, &list_constraint_logup_ab_result)?;

    cache.insert(
        "constraint_logup_ab_result".to_string(),
        pack_constraint_logup_ab_result,
    );

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", fiat_shamir2_result_hash)?;
    dsl.set_program_output("hash", pack_constraint_logup_ab_result_hash)?;

    Ok(dsl)
}
