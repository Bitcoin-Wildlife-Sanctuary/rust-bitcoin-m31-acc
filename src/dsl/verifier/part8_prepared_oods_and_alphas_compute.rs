use crate::dsl::modules::prepare::{power_alpha_six, prepare_pair_vanishing};
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<&str, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let prepared_oods1_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("prepared_oods1").unwrap().hash.as_ref().to_vec()),
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

    // unzip `prepared_oods1`
    let res = unzip_elements(
        &mut dsl,
        prepared_oods1_hash,
        cache.get("prepared_oods1").unwrap(),
    )?;
    assert_eq!(res.len(), 23);

    let oods_point_x_var = res[0];
    let oods_point_y_var = res[1];
    let prepared_oods_shifted_by_0_var = (res[2], res[3]);
    let prepared_oods_shifted_by_1_var = (res[4], res[5]);
    let prepared_oods_shifted_by_2_var = (res[6], res[7]);
    let column_line_coeff_trace_0_var = (res[8], res[9]);
    let column_line_coeff_trace_1_var = (res[10], res[11]);
    let column_line_coeff_trace_2_var = (res[12], res[13]);
    let column_line_coeff_composition_vars = [
        (res[14], res[15]),
        (res[16], res[17]),
        (res[18], res[19]),
        (res[20], res[21]),
    ];
    let random_coeff_2_var = res[22];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: prepare the oods
    let prepared_oods_var =
        prepare_pair_vanishing(&mut dsl, table, oods_point_x_var, oods_point_y_var)?;

    // Step 3: compute the power series of alpha
    let alphas = power_alpha_six(&mut dsl, table, random_coeff_2_var)?;

    let list_prepared_oods2 = [
        prepared_oods_shifted_by_0_var.0,
        prepared_oods_shifted_by_0_var.1,
        prepared_oods_shifted_by_1_var.0,
        prepared_oods_shifted_by_1_var.1,
        prepared_oods_shifted_by_2_var.0,
        prepared_oods_shifted_by_2_var.1,
        prepared_oods_var.0,
        prepared_oods_var.1,
        column_line_coeff_trace_0_var.0,
        column_line_coeff_trace_0_var.1,
        column_line_coeff_trace_1_var.0,
        column_line_coeff_trace_1_var.1,
        column_line_coeff_trace_2_var.0,
        column_line_coeff_trace_2_var.1,
        column_line_coeff_composition_vars[0].0,
        column_line_coeff_composition_vars[0].1,
        column_line_coeff_composition_vars[1].0,
        column_line_coeff_composition_vars[1].1,
        column_line_coeff_composition_vars[2].0,
        column_line_coeff_composition_vars[2].1,
        column_line_coeff_composition_vars[3].0,
        column_line_coeff_composition_vars[3].1,
        alphas[0],
        alphas[1],
        alphas[2],
        alphas[3],
        alphas[4],
        alphas[5],
    ];

    let (pack_prepared_oods2_hash, pack_prepared_oods2) =
        zip_elements(&mut dsl, &list_prepared_oods2)?;

    cache.insert("prepared_oods2", pack_prepared_oods2);
    dsl.set_program_output("hash", pack_prepared_oods2_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
