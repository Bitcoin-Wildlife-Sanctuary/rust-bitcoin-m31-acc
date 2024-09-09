use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use bitcoin_script_dsl::options::Options;
use itertools::Itertools;
use std::collections::HashMap;

pub fn generate_dsl(hints: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
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
    let query1_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("query1").unwrap().hash.as_ref().to_vec()),
    )?;
    let query2_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("query2").unwrap().hash.as_ref().to_vec()),
    )?;
    let query3_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("query3").unwrap().hash.as_ref().to_vec()),
    )?;
    let unsorted_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("unsorted").unwrap().hash.as_ref().to_vec()),
    )?;

    // unzip `unsorted`
    let res = unzip_elements(&mut dsl, unsorted_hash, cache.get("unsorted").unwrap())?;
    assert_eq!(res.len(), 65);

    let mut res_iter = res.iter();

    let mut fri_tree_commitments_vars = vec![];

    for _ in 0..5 {
        fri_tree_commitments_vars.push(*res_iter.next().unwrap());
    }

    let mut queries = vec![];
    let mut trace_queried_results = vec![];
    let mut composition_queried_results = vec![];
    let mut twiddles_vars = vec![];

    for _ in 0..5 {
        queries.push(*res_iter.next().unwrap());
        trace_queried_results.push((*res_iter.next().unwrap(), *res_iter.next().unwrap()));
        composition_queried_results.push((*res_iter.next().unwrap(), *res_iter.next().unwrap()));

        let mut tmp = vec![];
        for _ in 0..7 {
            tmp.push(*res_iter.next().unwrap());
        }
        twiddles_vars.push(tmp);
    }

    assert!(res_iter.next().is_none());

    let mut folding_intermediate_results_vars = vec![];
    for (&query, fold_hints) in queries
        .iter()
        .zip(hints.per_query_fold_hints.iter().skip(3))
        .take(5)
    {
        let queries = dsl.execute("decompose_positions_to_5", &[query])?;
        let mut tmp = vec![];
        for ((&commitment, proof), &query) in fri_tree_commitments_vars
            .iter()
            .zip(fold_hints.twin_proofs.iter())
            .zip(queries.iter())
        {
            let res = dsl.execute_with_options(
                "merkle_twin_tree_4",
                &[commitment, query],
                &Options::new()
                    .with_multi_u32(
                        "left",
                        vec![
                            proof.left[0].0,
                            proof.left[1].0,
                            proof.left[2].0,
                            proof.left[3].0,
                        ],
                    )
                    .with_multi_u32(
                        "right",
                        vec![
                            proof.right[0].0,
                            proof.right[1].0,
                            proof.right[2].0,
                            proof.right[3].0,
                        ],
                    )
                    .with_multi_binary(
                        "path",
                        proof
                            .path
                            .siblings
                            .iter()
                            .map(|x| x.as_ref().to_vec())
                            .collect_vec(),
                    ),
            )?;

            let left_first = dsl.execute("cm31_from_real_and_imag", &[res[0], res[1]])?[0];
            let left_second = dsl.execute("cm31_from_real_and_imag", &[res[2], res[3]])?[0];
            let left = dsl.execute("qm31_from_first_and_second", &[left_first, left_second])?[0];

            let right_first = dsl.execute("cm31_from_real_and_imag", &[res[4], res[5]])?[0];
            let right_second = dsl.execute("cm31_from_real_and_imag", &[res[6], res[7]])?[0];
            let right = dsl.execute("qm31_from_first_and_second", &[right_first, right_second])?[0];

            tmp.push((left, right));
        }
        folding_intermediate_results_vars.push(tmp);
    }

    let mut list_query_specific = vec![vec![]; 5];
    for i in 0..5 {
        list_query_specific[i].push(queries[i]);
        list_query_specific[i].push(trace_queried_results[i].0);
        list_query_specific[i].push(trace_queried_results[i].1);
        list_query_specific[i].push(composition_queried_results[i].0);
        list_query_specific[i].push(composition_queried_results[i].1);
        list_query_specific[i].extend_from_slice(&twiddles_vars[i]);
        for elem in folding_intermediate_results_vars[i].iter() {
            list_query_specific[i].push(elem.0);
            list_query_specific[i].push(elem.1);
        }
    }

    let (pack_query4_hash, pack_query4) = zip_elements(&mut dsl, &list_query_specific[0])?;
    let (pack_query5_hash, pack_query5) = zip_elements(&mut dsl, &list_query_specific[1])?;
    let (pack_query6_hash, pack_query6) = zip_elements(&mut dsl, &list_query_specific[2])?;
    let (pack_query7_hash, pack_query7) = zip_elements(&mut dsl, &list_query_specific[3])?;
    let (pack_query8_hash, pack_query8) = zip_elements(&mut dsl, &list_query_specific[4])?;

    cache.insert("query4".to_string(), pack_query4);
    cache.insert("query5".to_string(), pack_query5);
    cache.insert("query6".to_string(), pack_query6);
    cache.insert("query7".to_string(), pack_query7);
    cache.insert("query8".to_string(), pack_query8);

    let mut list_global_state = vec![];
    list_global_state.push(shared_information_hash);
    list_global_state.push(query1_hash);
    list_global_state.push(query2_hash);
    list_global_state.push(query3_hash);
    list_global_state.push(pack_query4_hash);
    list_global_state.push(pack_query5_hash);
    list_global_state.push(pack_query6_hash);
    list_global_state.push(pack_query7_hash);
    list_global_state.push(pack_query8_hash);

    let (pack_global_state_hash, pack_global_state) = zip_elements(&mut dsl, &list_global_state)?;
    cache.insert("global_state".to_string(), pack_global_state);

    dsl.set_program_output("hash", pack_global_state_hash)?;

    Ok(dsl)
}
