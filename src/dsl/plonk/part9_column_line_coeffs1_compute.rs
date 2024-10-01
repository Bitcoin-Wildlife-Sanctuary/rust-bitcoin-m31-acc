use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::modules::prepare::column_line_coeffs;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
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

    let prepared_oods_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("prepared_oods").unwrap().hash.as_ref().to_vec()),
    )?;

    let alphas_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("alphas").unwrap().hash.as_ref().to_vec()),
    )?;

    // unzip `prepared_oods_hash`
    let res = unzip_elements(
        &mut dsl,
        prepared_oods_hash,
        cache.get("prepared_oods").unwrap(),
    )?;
    assert_eq!(res.len(), 6);

    let oods_point_y_var = res[0];
    let oods_shifted_by_1_y_var = res[1];
    let _ = (res[2], res[3]);
    let _ = (res[4], res[5]);

    // unzip `fiat_shamir1_result_hash`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir1_result_hash,
        cache.get("fiat_shamir1_result").unwrap(),
    )?;
    assert_eq!(res.len(), 32);

    let mut res = res.into_iter();

    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let fri_fold_random_coeff_var = res.next().unwrap();
    let _ = res.next().unwrap();
    let last_layer_var = res.next().unwrap();

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

    // unzip `fiat_shamir2_result_hash`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir2_result_hash,
        cache.get("fiat_shamir2_result").unwrap(),
    )?;
    assert_eq!(res.len(), 14);

    let mut res = res.into_iter();

    let trace_queried_results_hash = res.next().unwrap();
    let interaction_queried_results_hash = res.next().unwrap();
    let constant_queried_results_hash = res.next().unwrap();
    let composition_queried_results_hash = res.next().unwrap();
    let twiddles_hash = res.next().unwrap();
    let fri_hash = res.next().unwrap();

    let mut queries = vec![];
    for _ in 0..8 {
        queries.push(res.next().unwrap());
    }

    assert!(res.next().is_none());

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute column line coeff for trace
    let column_line_traces_vars =
        column_line_coeffs(&mut dsl, table, oods_point_y_var, &trace_oods_values_vars)?;

    // Step 3: assemble the carrying state for the rest of the column line coeffs
    let mut list_fiat_shamir_result = vec![
        fri_fold_random_coeff_var,
        last_layer_var,
        trace_queried_results_hash,
        interaction_queried_results_hash,
        constant_queried_results_hash,
        composition_queried_results_hash,
        twiddles_hash,
        fri_hash,
    ];
    list_fiat_shamir_result.extend_from_slice(&queries);

    let (pack_fiat_shamir_result_hash, pack_fiat_shamir_result) =
        zip_elements(&mut dsl, &list_fiat_shamir_result)?;

    cache.insert("fiat_shamir_result".to_string(), pack_fiat_shamir_result);

    let mut list_column_line_trace = vec![];
    for column_line_trace_var in column_line_traces_vars.iter() {
        list_column_line_trace.push(column_line_trace_var.0);
        list_column_line_trace.push(column_line_trace_var.1);
    }

    let (pack_column_line_trace_hash, pack_column_line_trace) =
        zip_elements(&mut dsl, &list_column_line_trace)?;

    cache.insert("column_line_trace".to_string(), pack_column_line_trace);

    let mut list_column_line_coeffs1 = vec![];
    list_column_line_coeffs1.push(oods_point_y_var);
    list_column_line_coeffs1.push(oods_shifted_by_1_y_var);

    list_column_line_coeffs1.extend_from_slice(&interaction_oods_values_vars);
    list_column_line_coeffs1.extend_from_slice(&constant_oods_values_vars);
    list_column_line_coeffs1.extend_from_slice(&composition_oods_raw_values_vars);

    list_column_line_coeffs1.push(pack_fiat_shamir_result_hash);
    list_column_line_coeffs1.push(prepared_oods_hash);
    list_column_line_coeffs1.push(alphas_hash);
    list_column_line_coeffs1.push(pack_column_line_trace_hash);

    let (pack_column_line_coeffs1_hash, pack_column_line_coeffs1) =
        zip_elements(&mut dsl, &list_column_line_coeffs1)?;

    cache.insert("column_line_coeffs1".to_string(), pack_column_line_coeffs1);

    dsl.set_program_output("hash", pack_column_line_coeffs1_hash)?;

    Ok(dsl)
}
