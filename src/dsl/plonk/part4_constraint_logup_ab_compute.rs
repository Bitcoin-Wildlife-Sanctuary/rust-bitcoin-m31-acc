use crate::dsl::modules::fiat_shamir::eval_from_partial_evals;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
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

    let constraint_algebra_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("constraint_algebra_result")
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
    assert_eq!(res.len(), 31);

    let mut res = res.into_iter();

    let z_var = res.next().unwrap();
    let alpha_var = res.next().unwrap();
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

    // denominator_1 = a_wire + alpha * a_val - z
    // denominator_2 = b_wire + alpha * b_val - z
    // num_aggregated = denominator_1 + denominator_2
    // denominator_aggregated = denominator_1 * denominator_2

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

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

    // unzip `constraint_algebra_result_hash`
    let res = unzip_elements(
        &mut dsl,
        constraint_algebra_result_hash,
        cache.get("constraint_algebra_result").unwrap(),
    )?;
    assert_eq!(res.len(), 1);

    let constraint_algebra_result_var = res[0];

    let list_constraint_logup_ab_result = vec![
        constraint_algebra_result_var,
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
