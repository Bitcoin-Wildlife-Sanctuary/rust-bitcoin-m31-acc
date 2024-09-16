use crate::dsl::building_blocks::point::add_constant_m31_point;
use crate::dsl::fibonacci::hints::FIB_LOG_SIZE;
use crate::dsl::modules::prepare::prepare_pair_vanishing;
use crate::dsl::plonk::hints::Hints;
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

    let constraint_oods_point_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("constraint_oods_point")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `constraint_oods_point_hash`
    let res = unzip_elements(
        &mut dsl,
        constraint_oods_point_hash,
        cache.get("constraint_oods_point").unwrap(),
    )?;
    assert_eq!(res.len(), 2);

    let oods_point_x_var = res[0];
    let oods_point_y_var = res[1];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: shift the oods point
    let domain = CanonicCoset::new(FIB_LOG_SIZE);
    let shift_0 = domain.at(0);
    let shift_1 = domain.at(1);

    let (oods_shifted_by_0_x_var, oods_shifted_by_0_y_var) =
        add_constant_m31_point(&mut dsl, table, oods_point_x_var, oods_point_y_var, shift_0)?;
    let (oods_shifted_by_1_x_var, oods_shifted_by_1_y_var) =
        add_constant_m31_point(&mut dsl, table, oods_point_x_var, oods_point_y_var, shift_1)?;

    // Step 3: prepare the shifted points
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

    let mut list_oods_with_shifted01 = vec![
        oods_point_x_var,
        oods_point_y_var,
        oods_shifted_by_0_x_var,
        oods_shifted_by_0_y_var,
        oods_shifted_by_1_x_var,
        oods_shifted_by_1_y_var,
    ];
    list_oods_with_shifted01.push(prepared_oods_shifted_by_0_var.0);
    list_oods_with_shifted01.push(prepared_oods_shifted_by_0_var.1);
    list_oods_with_shifted01.push(prepared_oods_shifted_by_1_var.0);
    list_oods_with_shifted01.push(prepared_oods_shifted_by_1_var.1);

    let (pack_oods_with_shifted_01_hash, pack_oods_with_shifted_01) =
        zip_elements(&mut dsl, &list_oods_with_shifted01)?;

    cache.insert(
        "oods_with_shifted_01".to_string(),
        pack_oods_with_shifted_01,
    );

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", fiat_shamir2_result_hash)?;
    dsl.set_program_output("hash", pack_oods_with_shifted_01_hash)?;

    Ok(dsl)
}
