use crate::dsl::modules::prepare::column_line_coeffs;
use crate::dsl::plonk::hints::{Hints, LOG_N_ROWS};
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
    let column_line_coeffs3_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("column_line_coeffs3")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `column_line_coeffs3_hash`
    let res = unzip_elements(
        &mut dsl,
        column_line_coeffs3_hash,
        cache.get("column_line_coeffs3").unwrap(),
    )?;
    assert_eq!(res.len(), 12);
    let mut res = res.into_iter();

    let oods_point_y_var = res.next().unwrap();
    let _ = res.next().unwrap();

    let mut composition_oods_raw_values_vars = vec![];
    for _ in 0..4 {
        composition_oods_raw_values_vars.push(res.next().unwrap());
    }

    let fiat_shamir_result_hash = res.next().unwrap();
    let prepared_oods_hash = res.next().unwrap();
    let alphas_hash = res.next().unwrap();
    let column_line_trace_hash = res.next().unwrap();
    let column_line_interaction_hash = res.next().unwrap();
    let column_line_interaction_shifted_and_constant_hash = res.next().unwrap();

    assert!(res.next().is_none());

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute composition
    let column_line_compositions_vars = column_line_coeffs(
        &mut dsl,
        table,
        oods_point_y_var,
        &composition_oods_raw_values_vars,
    )?;

    // Step 3: start sorting the information
    let mut list_column_line_compositions = vec![];
    for column_line_composition_var in column_line_compositions_vars {
        list_column_line_compositions.push(column_line_composition_var.0);
        list_column_line_compositions.push(column_line_composition_var.1);
    }

    let (pack_column_line_composition_hash, pack_column_line_composition) =
        zip_elements(&mut dsl, &list_column_line_compositions)?;

    cache.insert(
        "column_line_composition".to_string(),
        pack_column_line_composition,
    );

    // Step 4: prepare information sufficient for opening the FRI commitments

    // unzip `fiat_shamir_result_hash`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir_result_hash,
        cache.get("fiat_shamir_result").unwrap(),
    )?;
    assert_eq!(res.len(), 16);
    let mut res = res.into_iter();

    let fri_fold_random_coeff_var = res.next().unwrap();
    let last_layer_var = res.next().unwrap();
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

    // unpack `fri_hash`
    let res = unzip_elements(&mut dsl, fri_hash, cache.get("fri").unwrap())?;
    assert_eq!(res.len(), (LOG_N_ROWS * 2) as usize);

    let mut res = res.into_iter();

    let mut fri_tree_commitments_vars = vec![];
    for _ in 0..LOG_N_ROWS {
        fri_tree_commitments_vars.push(res.next().unwrap());
    }

    let mut folding_alphas_vars = vec![];
    for _ in 0..LOG_N_ROWS {
        folding_alphas_vars.push(res.next().unwrap());
    }

    assert!(res.next().is_none());

    let mut list_shared_information = vec![
        fri_fold_random_coeff_var,
        last_layer_var,
        prepared_oods_hash,
        alphas_hash,
        column_line_trace_hash,
        column_line_interaction_hash,
        column_line_interaction_shifted_and_constant_hash,
        pack_column_line_composition_hash,
    ];
    list_shared_information.extend_from_slice(&folding_alphas_vars);

    let (pack_shared_information_hash, pack_shared_information) =
        zip_elements(&mut dsl, &list_shared_information)?;

    cache.insert("shared_information".to_string(), pack_shared_information);

    let mut list_sort_queries1 = vec![];
    list_sort_queries1.extend_from_slice(&queries);
    list_sort_queries1.extend_from_slice(&fri_tree_commitments_vars);
    list_sort_queries1.push(trace_queried_results_hash);
    list_sort_queries1.push(interaction_queried_results_hash);
    list_sort_queries1.push(constant_queried_results_hash);
    list_sort_queries1.push(composition_queried_results_hash);
    list_sort_queries1.push(twiddles_hash);

    let (pack_sort_queries1_hash, pack_sort_queries1) =
        zip_elements(&mut dsl, &list_sort_queries1)?;

    cache.insert("sort_queries1".to_string(), pack_sort_queries1);

    dsl.set_program_output("hash", pack_shared_information_hash)?;
    dsl.set_program_output("hash", pack_sort_queries1_hash)?;

    Ok(dsl)
}
