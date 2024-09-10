use crate::dsl::building_blocks::qm31::qm31_mul_m31_limbs;
use crate::dsl::tools::{unzip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
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

    let cur_aggregation4_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_aggregation4", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip the cur_aggregation1_result_hash
    let res = unzip_elements(
        &mut dsl,
        cur_aggregation4_hash,
        cache
            .get(format!("query{}_aggregation4", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 5);

    let y_var = res[0];
    let sum_left = res[1];
    let sum_right = res[2];
    let circle_poly_alpha_var = res[3];
    let expected_entry_quotient = res[4];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: invert the y_var
    let y_limbs_var = dsl.execute("m31_to_limbs", &[y_var])?[0];
    let y_inverse_limbs_var = dsl.execute("m31_limbs_inverse", &[table, y_limbs_var])?[0];

    // Step 3: do the first folding
    let ifft_results_vars = {
        let new_v0 = dsl.execute("qm31_add", &[sum_left, sum_right])?[0];

        let diff = dsl.execute("qm31_sub", &[sum_left, sum_right])?[0];

        let new_v1 = qm31_mul_m31_limbs(&mut dsl, table, diff, y_inverse_limbs_var)?;

        (new_v0, new_v1)
    };

    let second_limbs_var = dsl.execute("qm31_to_limbs", &[ifft_results_vars.1])?[0];
    let folding_alpha_limbs_var = dsl.execute("qm31_to_limbs", &[circle_poly_alpha_var])?[0];

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
