use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::modules::prepare::{column_line_coeffs, prepare_pair_vanishing};
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let column_line_coeff2_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("column_line_coeff2")
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

    // unzip `column_line_coeff2`
    let res = unzip_elements(
        &mut dsl,
        column_line_coeff2_hash,
        cache.get("column_line_coeff2").unwrap(),
    )?;
    assert_eq!(res.len(), 19);

    let composition_oods_raw_values_vars = [res[0], res[1], res[2], res[3]];
    let oods_point_x_var = res[4];
    let oods_point_y_var = res[5];
    let oods_shifted_by_0_x_var = res[6];
    let oods_shifted_by_0_y_var = res[7];
    let oods_shifted_by_1_x_var = res[8];
    let oods_shifted_by_1_y_var = res[9];
    let oods_shifted_by_2_x_var = res[10];
    let oods_shifted_by_2_y_var = res[11];
    let column_line_coeff_trace_0_var = (res[12], res[13]);
    let column_line_coeff_trace_1_var = (res[14], res[15]);
    let column_line_coeff_trace_2_var = (res[16], res[17]);
    let random_coeff_2_var = res[18];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the column line coeffs for composition
    let column_line_coeff_composition_vars = column_line_coeffs(
        &mut dsl,
        table,
        oods_point_y_var,
        &composition_oods_raw_values_vars,
    )?;

    // Step 3: prepare the oods_shifted_by_0/1/2
    let prepared_oods_shifted_by_0_var = prepare_pair_vanishing(
        &mut dsl,
        table,
        oods_shifted_by_0_x_var,
        oods_shifted_by_0_y_var,
    )?;
    let prepared_oods_shifted_by_1_var = prepare_pair_vanishing(
        &mut dsl,
        table,
        oods_shifted_by_1_x_var,
        oods_shifted_by_1_y_var,
    )?;
    let prepared_oods_shifted_by_2_var = prepare_pair_vanishing(
        &mut dsl,
        table,
        oods_shifted_by_2_x_var,
        oods_shifted_by_2_y_var,
    )?;

    let list_prepared_oods1 = [
        oods_point_x_var,
        oods_point_y_var,
        prepared_oods_shifted_by_0_var.0,
        prepared_oods_shifted_by_0_var.1,
        prepared_oods_shifted_by_1_var.0,
        prepared_oods_shifted_by_1_var.1,
        prepared_oods_shifted_by_2_var.0,
        prepared_oods_shifted_by_2_var.1,
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
        random_coeff_2_var,
    ];

    let (pack_prepared_oods1_hash, pack_prepared_oods1) =
        zip_elements(&mut dsl, &list_prepared_oods1)?;

    cache.insert("prepared_oods1".to_string(), pack_prepared_oods1);
    dsl.set_program_output("hash", pack_prepared_oods1_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
