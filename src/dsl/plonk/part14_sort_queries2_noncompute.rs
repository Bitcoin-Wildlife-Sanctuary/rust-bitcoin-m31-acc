use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::plonk::hints::{Hints, LOG_N_ROWS};
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let shared_information_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("shared_information")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    let sort_queries1_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("sort_queries1").unwrap().hash.as_ref().to_vec()),
    )?;

    let mut folding_intermediate_hashes = vec![];
    for i in 1..=8 {
        folding_intermediate_hashes.push(
            dsl.alloc_input(
                "hash",
                Element::Str(
                    cache
                        .get(&format!("folding_intermediate_{}", i))
                        .unwrap()
                        .hash
                        .as_ref()
                        .to_vec(),
                ),
            )?,
        );
    }

    // unpack `shared_information_hash`
    let res = unzip_elements(
        &mut dsl,
        shared_information_hash,
        cache.get("shared_information").unwrap(),
    )?;
    assert_eq!(res.len(), 8 + LOG_N_ROWS as usize);

    let fri_fold_random_coeff_var = res[0];
    let last_layer_var = res[1];
    let prepared_oods_hash = res[2];
    let alphas_hash = res[3];
    let column_line_trace_hash = res[4];
    let column_line_interaction_hash = res[5];
    let column_line_interaction_shifted_and_constant_hash = res[6];
    let column_line_composition_hash = res[7];
    let folding_alphas_vars = [res[8], res[9], res[10], res[11], res[12]];

    // unpack `sort_queries1_hash`
    let res = unzip_elements(
        &mut dsl,
        sort_queries1_hash,
        cache.get("sort_queries1").unwrap(),
    )?;
    assert_eq!(res.len(), (8 + LOG_N_ROWS + 5) as usize);

    let mut res = res.into_iter();

    let mut queries = vec![];
    for _ in 0..8 {
        queries.push(res.next().unwrap());
    }

    let mut fri_tree_commitments_vars = vec![];
    for _ in 0..LOG_N_ROWS {
        fri_tree_commitments_vars.push(res.next().unwrap());
    }

    let trace_queried_results_hash = res.next().unwrap();
    let interaction_queried_results_hash = res.next().unwrap();
    let constant_queried_results_hash = res.next().unwrap();
    let composition_queried_results_hash = res.next().unwrap();
    let twiddles_hash = res.next().unwrap();

    assert!(res.next().is_none());

    // unwrap `trace_queried_results_hash`
    let trace_queried_results = unzip_elements(
        &mut dsl,
        trace_queried_results_hash,
        cache.get("trace_queried_results").unwrap(),
    )?;
    assert_eq!(trace_queried_results.len(), 64);

    // unwrap `interaction_queried_results_hash`
    let interaction_queried_results = unzip_elements(
        &mut dsl,
        interaction_queried_results_hash,
        cache.get("interaction_queried_results").unwrap(),
    )?;
    assert_eq!(interaction_queried_results.len(), 32);

    // unwrap `constant_queried_results_hash`
    let constant_queried_results = unzip_elements(
        &mut dsl,
        constant_queried_results_hash,
        cache.get("constant_queried_results").unwrap(),
    )?;
    assert_eq!(constant_queried_results.len(), 64);

    // unwrap `composition_queried_results_hash`
    let composition_queried_results = unzip_elements(
        &mut dsl,
        composition_queried_results_hash,
        cache.get("composition_queried_results").unwrap(),
    )?;
    assert_eq!(composition_queried_results.len(), 16);

    // unwrap `twiddles_hash`
    let twiddles = unzip_elements(&mut dsl, twiddles_hash, cache.get("twiddles").unwrap())?;
    assert_eq!(twiddles.len(), 56);

    let mut queries_hashes = vec![];
    for i in 0..8 {
        // unwrap `folding_intermediate_hashes[i]`
        let folding_intermediate_cur = unzip_elements(
            &mut dsl,
            folding_intermediate_hashes[i],
            cache
                .get(&format!("folding_intermediate_{}", i + 1))
                .unwrap(),
        )?;
        assert_eq!(folding_intermediate_cur.len(), 10);

        let swap_bit_var = dsl.execute("skip_one_and_extract_5_bits", &[queries[i]])?[0];
        let expected_entry_quotient = dsl.execute(
            "qm31_conditional_swap",
            &[
                folding_intermediate_cur[0],
                folding_intermediate_cur[1],
                swap_bit_var,
            ],
        )?[0];

        let mut list_query_post_folding_cur = vec![];
        list_query_post_folding_cur.extend_from_slice(&trace_queried_results[i * 8..(i + 1) * 8]);
        list_query_post_folding_cur
            .extend_from_slice(&interaction_queried_results[i * 4..(i + 1) * 4]);
        list_query_post_folding_cur
            .extend_from_slice(&constant_queried_results[i * 8..(i + 1) * 8]);
        list_query_post_folding_cur
            .extend_from_slice(&composition_queried_results[i * 2..(i + 1) * 2]);
        list_query_post_folding_cur.extend_from_slice(&twiddles[i * 7 + 5..(i + 1) * 7]);
        list_query_post_folding_cur.push(column_line_trace_hash);
        list_query_post_folding_cur.push(column_line_interaction_hash);
        list_query_post_folding_cur.push(column_line_interaction_shifted_and_constant_hash);
        list_query_post_folding_cur.push(column_line_composition_hash);
        list_query_post_folding_cur.push(prepared_oods_hash);
        list_query_post_folding_cur.push(fri_fold_random_coeff_var);
        list_query_post_folding_cur.push(alphas_hash);
        list_query_post_folding_cur.push(expected_entry_quotient);

        let (pack_query_post_folding_cur_hash, pack_query_post_folding_cur) =
            zip_elements(&mut dsl, &list_query_post_folding_cur)?;

        cache.insert(
            format!("query_post_folding_{}", i + 1).to_string(),
            pack_query_post_folding_cur,
        );

        let mut list_query_folding_cur = vec![];
        list_query_folding_cur.push(queries[i]);
        list_query_folding_cur.push(last_layer_var);
        list_query_folding_cur.extend_from_slice(&folding_intermediate_cur);
        list_query_folding_cur.extend_from_slice(&folding_alphas_vars);
        list_query_folding_cur.extend_from_slice(&twiddles[i * 7..(i + 1) * 7 - 2]);
        list_query_folding_cur.push(pack_query_post_folding_cur_hash);

        let (pack_query_folding_cur_hash, pack_query_folding_cur) =
            zip_elements(&mut dsl, &list_query_folding_cur)?;

        cache.insert(
            format!("query_folding_{}", i + 1).to_string(),
            pack_query_folding_cur,
        );

        queries_hashes.push(pack_query_folding_cur_hash);
    }

    let mut list_global_state = vec![shared_information_hash];
    list_global_state.extend_from_slice(&queries_hashes);

    let (pack_global_state_hash, pack_global_state) = zip_elements(&mut dsl, &list_global_state)?;

    cache.insert("global_state".to_string(), pack_global_state);

    dsl.set_program_output("hash", pack_global_state_hash)?;

    Ok(dsl)
}
