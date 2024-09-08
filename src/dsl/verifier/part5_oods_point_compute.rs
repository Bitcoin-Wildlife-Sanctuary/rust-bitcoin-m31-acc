use crate::dsl::building_blocks::point::add_constant_m31_point;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use fibonacci_example::FIB_LOG_SIZE;
use std::collections::HashMap;
use stwo_prover::core::poly::circle::CanonicCoset;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<&str, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir_verify4_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir_verify4")
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

    // unzip `fiat_shamir_verify4`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir_verify4_hash,
        cache.get("fiat_shamir_verify4").unwrap(),
    )?;
    assert_eq!(res.len(), 10);

    let trace_oods_values_vars = [res[0], res[1], res[2]];
    let composition_oods_raw_values_vars = [res[3], res[4], res[5], res[6]];
    let oods_point_x_var = res[7];
    let oods_point_y_var = res[8];
    let random_coeff_2_var = res[9];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: mask the point
    let domain = CanonicCoset::new(FIB_LOG_SIZE);
    let shift_0 = domain.at(0);
    let shift_1 = domain.at(1);

    let (oods_shifted_by_0_x_var, oods_shifted_by_0_y_var) =
        add_constant_m31_point(&mut dsl, table, oods_point_x_var, oods_point_y_var, shift_0)?;
    let (oods_shifted_by_1_x_var, oods_shifted_by_1_y_var) =
        add_constant_m31_point(&mut dsl, table, oods_point_x_var, oods_point_y_var, shift_1)?;

    let list_column_line_coeff1 = [
        trace_oods_values_vars[0],
        trace_oods_values_vars[1],
        trace_oods_values_vars[2],
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
        random_coeff_2_var,
    ];

    let (pack_column_line_coeff1_hash, pack_column_line_coeff1) =
        zip_elements(&mut dsl, &list_column_line_coeff1)?;

    cache.insert("column_line_coeff1", pack_column_line_coeff1);

    dsl.set_program_output("hash", pack_column_line_coeff1_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
