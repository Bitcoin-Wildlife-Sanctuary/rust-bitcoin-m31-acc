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

    let constraint_logup_ab_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("constraint_logup_ab_result")
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
    let before_oods_channel_var = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let claimed_sum_divided_var = res.next().unwrap();

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

    // unzip `constraint_logup_ab_result`
    let res = unzip_elements(
        &mut dsl,
        constraint_logup_ab_result_hash,
        cache.get("constraint_logup_ab_result").unwrap(),
    )?;
    assert_eq!(res.len(), 2);

    let constraint_algebra_result_var = res[0];
    let constraint_logup_ab_without_randomizer_var = res[1];

    // (combine_ef(c_logup[:][0]) - combine_ef(c_logup[:][1]) - combine_ef(a_b_logup_val) + claim / (1 << log_n)) * denominator - nominator

    // num_c = -mult
    // denom_c = c_wire + alpha * c_val - z

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the denominator
    let alpha_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_var])?[0];
    let c_val_var = trace_oods_values_vars[3];
    let c_limbs_var = dsl.execute("qm31_to_limbs", &[c_val_var])?[0];

    let alpha_times_c_val_var =
        dsl.execute("qm31_limbs_mul", &[table, alpha_limbs_var, c_limbs_var])?[0];

    let c_wire_var = constant_oods_values_vars[2];
    let mut denom_var = dsl.execute("qm31_add", &[c_wire_var, alpha_times_c_val_var])?[0];
    denom_var = dsl.execute("qm31_sub", &[denom_var, z_var])?[0];

    // Step 3: compute the combine_ef(c_logup[:][0]) - combine_ef(c_logup[:][1]) - combine_ef(a_b_logup_val) + claim / (1 << log_n)

    let c_logup_var = eval_from_partial_evals(
        &mut dsl,
        interaction_oods_values_vars[4],
        interaction_oods_values_vars[6],
        interaction_oods_values_vars[8],
        interaction_oods_values_vars[10],
    )?;

    let c_logup_next_var = eval_from_partial_evals(
        &mut dsl,
        interaction_oods_values_vars[5],
        interaction_oods_values_vars[7],
        interaction_oods_values_vars[9],
        interaction_oods_values_vars[11],
    )?;

    let a_b_logup_var = eval_from_partial_evals(
        &mut dsl,
        interaction_oods_values_vars[0],
        interaction_oods_values_vars[1],
        interaction_oods_values_vars[2],
        interaction_oods_values_vars[3],
    )?;

    let mut diff_var = dsl.execute("qm31_sub", &[c_logup_var, c_logup_next_var])?[0];
    diff_var = dsl.execute("qm31_sub", &[diff_var, a_b_logup_var])?[0];
    diff_var = dsl.execute("qm31_add", &[diff_var, claimed_sum_divided_var])?[0];

    // Step 4: compute the last constraint evaluation

    let diff_limbs_var = dsl.execute("qm31_to_limbs", &[diff_var])?[0];
    let denom_limbs_var = dsl.execute("qm31_to_limbs", &[denom_var])?[0];

    let diff_times_denom_var =
        dsl.execute("qm31_limbs_mul", &[table, diff_limbs_var, denom_limbs_var])?[0];
    let mult_var = trace_oods_values_vars[0];
    let constraint_logup_c_var = dsl.execute("qm31_add", &[diff_times_denom_var, mult_var])?[0];

    let constraint_algebra_plus_logup_c_var = dsl.execute(
        "qm31_add",
        &[constraint_logup_c_var, constraint_algebra_result_var],
    )?[0];

    let list_constraint_logup_c_result = vec![
        constraint_logup_ab_without_randomizer_var,
        constraint_algebra_plus_logup_c_var,
        before_oods_channel_var,
    ];
    let (pack_constraint_logup_c_result_hash, pack_constraint_logup_c_result) =
        zip_elements(&mut dsl, &list_constraint_logup_c_result)?;

    cache.insert(
        "constraint_logup_c_result".to_string(),
        pack_constraint_logup_c_result,
    );

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", fiat_shamir2_result_hash)?;
    dsl.set_program_output("hash", pack_constraint_logup_c_result_hash)?;

    Ok(dsl)
}
