use crate::dsl::building_blocks::point::get_random_point_full;
use crate::dsl::modules::fiat_shamir::{
    eval_from_partial_evals, step_constraint_numerator_evaluation,
};
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use fibonacci_example::FIB_LOG_SIZE;
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir_verify1_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir_verify1")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;
    let after_fiat_shamir_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("after_fiat_shamir")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `fiat_shamir_verify1`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir_verify1_hash,
        cache.get("fiat_shamir_verify1").unwrap(),
    )?;
    assert_eq!(res.len(), 10);

    let random_coeff_1_var = res[0];
    let trace_oods_values_vars = [res[1], res[2], res[3]];
    let composition_oods_raw_values_vars = [res[4], res[5], res[6], res[7]];
    let before_oods_channel_var = res[8];
    let random_coeff_2_var = res[9];

    // Step 1: allocate the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: draw the OODS point
    let (_, oods_point_x_var, oods_point_y_var) =
        get_random_point_full(&mut dsl, table, before_oods_channel_var)?;

    // Step 5: step_constraint_numerator_evaluation
    let step_constraint_numerator_var = step_constraint_numerator_evaluation(
        &mut dsl,
        table,
        trace_oods_values_vars[0],
        trace_oods_values_vars[1],
        trace_oods_values_vars[2],
        oods_point_x_var,
        oods_point_y_var,
        FIB_LOG_SIZE,
    )?;

    // Step 3: eval from partial evals
    let composition_oods_value_var = eval_from_partial_evals(
        &mut dsl,
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
    )?;

    let list_fiat_shamir_verify2 = [
        random_coeff_1_var,
        trace_oods_values_vars[0],
        trace_oods_values_vars[1],
        trace_oods_values_vars[2],
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
        oods_point_x_var,
        oods_point_y_var,
        step_constraint_numerator_var,
        composition_oods_value_var,
        random_coeff_2_var,
    ];

    let (pack_fiat_shamir_verify2_hash, pack_fiat_shamir_verify2) =
        zip_elements(&mut dsl, &list_fiat_shamir_verify2)?;

    cache.insert("fiat_shamir_verify2".to_string(), pack_fiat_shamir_verify2);
    dsl.set_program_output("hash", pack_fiat_shamir_verify2_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
