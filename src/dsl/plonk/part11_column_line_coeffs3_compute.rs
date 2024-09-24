use crate::dsl::modules::prepare::column_line_coeffs;
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
    let column_line_coeffs2_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("column_line_coeffs2")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `column_line_coeffs2_hash`
    let res = unzip_elements(
        &mut dsl,
        column_line_coeffs2_hash,
        cache.get("column_line_coeffs2").unwrap(),
    )?;
    assert_eq!(res.len(), 19);

    let mut res = res.into_iter();

    let oods_point_y_var = res.next().unwrap();
    let oods_shifted_by_1_y_var = res.next().unwrap();

    let mut interaction_oods_values_vars = vec![];
    for _ in 0..4 {
        interaction_oods_values_vars.push(res.next().unwrap());
    }

    let mut constant_oods_values_vars = vec![];
    for _ in 0..4 {
        constant_oods_values_vars.push(res.next().unwrap());
    }

    let mut composition_oods_raw_values_vars = vec![];
    for _ in 0..4 {
        composition_oods_raw_values_vars.push(res.next().unwrap());
    }

    let fiat_shamir_result_hash = res.next().unwrap();
    let prepared_oods_hash = res.next().unwrap();
    let alphas_hash = res.next().unwrap();
    let column_line_trace_hash = res.next().unwrap();
    let column_line_interaction_hash = res.next().unwrap();

    assert!(res.next().is_none());

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute interaction, focusing on the oods_shifted_by_1_y_var ones
    let column_line_interactions_shifted_vars = column_line_coeffs(
        &mut dsl,
        table,
        oods_shifted_by_1_y_var,
        &interaction_oods_values_vars,
    )?;

    // Step 3: compute the constant
    let column_line_constants_vars = column_line_coeffs(
        &mut dsl,
        table,
        oods_point_y_var,
        &constant_oods_values_vars,
    )?;

    // Step 4: assemble the carrying state of the rest of the column line coeffs
    let mut list_column_line_interaction_shifted_and_constant = vec![];

    for column_line_interaction_shifted_var in column_line_interactions_shifted_vars.iter() {
        list_column_line_interaction_shifted_and_constant
            .push(column_line_interaction_shifted_var.0);
        list_column_line_interaction_shifted_and_constant
            .push(column_line_interaction_shifted_var.1);
    }

    for column_line_constant_var in column_line_constants_vars.iter() {
        list_column_line_interaction_shifted_and_constant.push(column_line_constant_var.0);
        list_column_line_interaction_shifted_and_constant.push(column_line_constant_var.1);
    }

    let (
        pack_column_line_interaction_shifted_and_constant_hash,
        pack_column_line_interaction_shifted_and_constant,
    ) = zip_elements(&mut dsl, &list_column_line_interaction_shifted_and_constant)?;

    cache.insert(
        "column_line_interaction_shifted_and_constant".to_string(),
        pack_column_line_interaction_shifted_and_constant,
    );

    let mut list_column_line_coeffs3 = vec![];
    list_column_line_coeffs3.push(oods_point_y_var);
    list_column_line_coeffs3.push(oods_shifted_by_1_y_var);

    list_column_line_coeffs3.extend_from_slice(&composition_oods_raw_values_vars);

    list_column_line_coeffs3.push(fiat_shamir_result_hash);
    list_column_line_coeffs3.push(prepared_oods_hash);
    list_column_line_coeffs3.push(alphas_hash);
    list_column_line_coeffs3.push(column_line_trace_hash);
    list_column_line_coeffs3.push(column_line_interaction_hash);
    list_column_line_coeffs3.push(pack_column_line_interaction_shifted_and_constant_hash);

    let (pack_column_line_coeffs3_hash, pack_column_line_coeffs3) =
        zip_elements(&mut dsl, &list_column_line_coeffs3)?;

    cache.insert("column_line_coeffs3".to_string(), pack_column_line_coeffs3);

    dsl.set_program_output("hash", pack_column_line_coeffs3_hash)?;

    Ok(dsl)
}
