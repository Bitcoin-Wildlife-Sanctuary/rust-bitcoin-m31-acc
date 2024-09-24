use crate::dsl::building_blocks::qm31::{qm31_mul_cm31_limbs, qm31_mul_m31_limbs};
use crate::dsl::modules::quotients::denominator_inverse_limbs_from_prepared;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use std::collections::HashMap;

pub fn generate_dsl(
    _: &Hints,
    cache: &mut HashMap<String, Zipper>,
    query_idx: usize,
) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let global_state_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("global_state").unwrap().hash.as_ref().to_vec()),
    )?;

    let composition_results_cur_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("composition_results_{}", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unpack `composition_results_cur_hash`
    let res = unzip_elements(
        &mut dsl,
        composition_results_cur_hash,
        cache
            .get(&format!("composition_results_{}", query_idx))
            .unwrap(),
    )?;
    assert_eq!(res.len(), 12);

    let term_trace_to_composition_l = res[0];
    let term_trace_to_composition_r = res[1];
    let sum_num_interaction_logc_s_l = res[2];
    let sum_num_interaction_logc_s_r = res[3];
    let x_var = res[4];
    let y_var = res[5];
    let prepared_oods_var = (res[6], res[7]);
    let prepared_oods_shifted_by_1_var = (res[8], res[9]);
    let fri_fold_random_coeff_var = res[10];
    let expected_entry_quotient = res[11];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the denominator inverse
    let denominator_inverse_var = denominator_inverse_limbs_from_prepared(
        &mut dsl,
        table,
        prepared_oods_var.0,
        prepared_oods_var.1,
        x_var,
        y_var,
    )?;

    // Step 3: multiply the denominator inverse
    let res_1_l = qm31_mul_cm31_limbs(
        &mut dsl,
        table,
        term_trace_to_composition_l,
        denominator_inverse_var.0,
    )?;
    let res_1_r = qm31_mul_cm31_limbs(
        &mut dsl,
        table,
        term_trace_to_composition_r,
        denominator_inverse_var.1,
    )?;

    // Step 4: compute the other denominator inverse
    let denominator_inverse_shifted_var = denominator_inverse_limbs_from_prepared(
        &mut dsl,
        table,
        prepared_oods_shifted_by_1_var.0,
        prepared_oods_shifted_by_1_var.1,
        x_var,
        y_var,
    )?;

    // Step 5: multiply the denominator inverse for shifted
    let res_2_l = qm31_mul_cm31_limbs(
        &mut dsl,
        table,
        sum_num_interaction_logc_s_l,
        denominator_inverse_shifted_var.0,
    )?;
    let res_2_r = qm31_mul_cm31_limbs(
        &mut dsl,
        table,
        sum_num_interaction_logc_s_r,
        denominator_inverse_shifted_var.1,
    )?;

    // Step 6: compute the sum
    let res_l = dsl.execute("qm31_add", &[res_1_l, res_2_l])?[0];
    let res_r = dsl.execute("qm31_add", &[res_1_r, res_2_r])?[0];

    // Step 7: invert the y_var
    let y_limbs_var = dsl.execute("m31_to_limbs", &[y_var])?[0];
    let y_inverse_limbs_var = dsl.execute("m31_limbs_inverse", &[table, y_limbs_var])?[0];

    // Step 8: do the first folding
    let ifft_results_vars = {
        let new_v0 = dsl.execute("qm31_add", &[res_l, res_r])?[0];

        let diff = dsl.execute("qm31_sub", &[res_l, res_r])?[0];

        let new_v1 = qm31_mul_m31_limbs(&mut dsl, table, diff, y_inverse_limbs_var)?;

        (new_v0, new_v1)
    };

    let second_limbs_var = dsl.execute("qm31_to_limbs", &[ifft_results_vars.1])?[0];
    let folding_alpha_limbs_var = dsl.execute("qm31_to_limbs", &[fri_fold_random_coeff_var])?[0];

    let second_times_folding_alpha_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, second_limbs_var, folding_alpha_limbs_var],
    )?[0];

    let entry_quotient = dsl.execute(
        "qm31_add",
        &[second_times_folding_alpha_var, ifft_results_vars.0],
    )?[0];

    let _ = dsl.execute(
        "qm31_equalverify",
        &[entry_quotient, expected_entry_quotient],
    )?;

    dsl.set_program_output("hash", global_state_hash)?;

    Ok(dsl)
}
