use crate::dsl::building_blocks::point::add_constant_m31_point;
use crate::dsl::fibonacci::hints::{Hints, FIB_LOG_SIZE};
use crate::dsl::modules::prepare::column_line_coeffs;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use std::collections::HashMap;
use stwo_prover::core::poly::circle::CanonicCoset;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let column_line_coeff1_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("column_line_coeff1")
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

    // unzip `column_line_coeff1`
    let res = unzip_elements(
        &mut dsl,
        column_line_coeff1_hash,
        cache.get("column_line_coeff1").unwrap(),
    )?;
    assert_eq!(res.len(), 14);

    let trace_oods_values_vars = [res[0], res[1], res[2]];
    let composition_oods_raw_values_vars = [res[3], res[4], res[5], res[6]];
    let oods_point_x_var = res[7];
    let oods_point_y_var = res[8];
    let oods_shifted_by_0_x_var = res[9];
    let oods_shifted_by_0_y_var = res[10];
    let oods_shifted_by_1_x_var = res[11];
    let oods_shifted_by_1_y_var = res[12];
    let random_coeff_2_var = res[13];

    let domain = CanonicCoset::new(FIB_LOG_SIZE);
    let shift_2 = domain.at(2);

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the last shifted point
    let (oods_shifted_by_2_x_var, oods_shifted_by_2_y_var) =
        add_constant_m31_point(&mut dsl, table, oods_point_x_var, oods_point_y_var, shift_2)?;

    // Step 3: compute the first column line coeffs
    let column_line_coeff_trace_0_var = column_line_coeffs(
        &mut dsl,
        table,
        oods_shifted_by_0_y_var,
        &[trace_oods_values_vars[0]],
    )?[0];
    let column_line_coeff_trace_1_var = column_line_coeffs(
        &mut dsl,
        table,
        oods_shifted_by_1_y_var,
        &[trace_oods_values_vars[1]],
    )?[0];
    let column_line_coeff_trace_2_var = column_line_coeffs(
        &mut dsl,
        table,
        oods_shifted_by_2_y_var,
        &[trace_oods_values_vars[2]],
    )?[0];

    let list_column_line_coeff2 = [
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
        oods_point_x_var,
        oods_point_y_var,
        oods_shifted_by_0_x_var,
        oods_shifted_by_0_y_var,
        oods_shifted_by_1_x_var,
        oods_shifted_by_1_y_var,
        oods_shifted_by_2_x_var,
        oods_shifted_by_2_y_var,
        column_line_coeff_trace_0_var.0,
        column_line_coeff_trace_0_var.1,
        column_line_coeff_trace_1_var.0,
        column_line_coeff_trace_1_var.1,
        column_line_coeff_trace_2_var.0,
        column_line_coeff_trace_2_var.1,
        random_coeff_2_var,
    ];

    let (pack_column_line_coeff2_hash, pack_column_line_coeff2) =
        zip_elements(&mut dsl, &list_column_line_coeff2)?;

    cache.insert("column_line_coeff2".to_string(), pack_column_line_coeff2);
    dsl.set_program_output("hash", pack_column_line_coeff2_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
