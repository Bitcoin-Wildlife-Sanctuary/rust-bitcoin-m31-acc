use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::framework::options::Options;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;

pub fn generate_dsl(hints: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let prepared_oods2_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("prepared_oods2").unwrap().hash.as_ref().to_vec()),
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

    // unzip `prepared_oods2`
    let res = unzip_elements(
        &mut dsl,
        prepared_oods2_hash,
        cache.get("prepared_oods2").unwrap(),
    )?;
    assert_eq!(res.len(), 28);

    let mut list_shared_information = vec![];
    list_shared_information.extend_from_slice(&res);

    // unzip `after_fiat_shamir`
    let res = unzip_elements(
        &mut dsl,
        after_fiat_shamir_hash,
        cache.get("after_fiat_shamir").unwrap(),
    )?;
    assert_eq!(res.len(), 108);

    let mut res_iter = res.iter();

    let circle_poly_alpha_var = *res_iter.next().unwrap();
    let last_layer_var = *res_iter.next().unwrap();

    let mut fri_tree_commitments_vars = vec![];
    for _ in 0..5 {
        fri_tree_commitments_vars.push(*res_iter.next().unwrap());
    }

    let mut folding_alphas_vars = vec![];
    for _ in 0..5 {
        folding_alphas_vars.push(*res_iter.next().unwrap());
    }

    let mut queries = vec![];
    for _ in 0..8 {
        queries.push(*res_iter.next().unwrap());
    }

    let mut trace_queried_results = vec![];
    for _ in 0..8 {
        trace_queried_results.push((*res_iter.next().unwrap(), *res_iter.next().unwrap()));
    }

    let mut composition_queried_results = vec![];
    for _ in 0..8 {
        composition_queried_results.push((*res_iter.next().unwrap(), *res_iter.next().unwrap()));
    }

    let mut twiddles_vars = vec![];
    for _ in 0..8 {
        let mut tmp = vec![];
        for _ in 0..7 {
            tmp.push(*res_iter.next().unwrap());
        }
        twiddles_vars.push(tmp);
    }

    assert!(res_iter.next().is_none());

    list_shared_information.push(circle_poly_alpha_var);
    list_shared_information.push(last_layer_var);
    list_shared_information.extend_from_slice(&folding_alphas_vars);

    let (pack_shared_information_hash, pack_shared_information) =
        zip_elements(&mut dsl, &list_shared_information)?;

    let mut folding_intermediate_results_vars = vec![];
    for (&query, fold_hints) in queries
        .iter()
        .zip(hints.per_query_fold_hints.iter())
        .take(3)
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

    let mut list_query_specific = vec![vec![]; 3];
    for i in 0..3 {
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

    let mut list_unsorted = vec![];
    list_unsorted.extend_from_slice(fri_tree_commitments_vars.as_slice());
    for i in 3..8 {
        list_unsorted.push(queries[i]);
        list_unsorted.push(trace_queried_results[i].0);
        list_unsorted.push(trace_queried_results[i].1);
        list_unsorted.push(composition_queried_results[i].0);
        list_unsorted.push(composition_queried_results[i].1);
        list_unsorted.extend_from_slice(&twiddles_vars[i]);
    }

    let (pack_query1_hash, pack_query1) = zip_elements(&mut dsl, &list_query_specific[0])?;
    let (pack_query2_hash, pack_query2) = zip_elements(&mut dsl, &list_query_specific[1])?;
    let (pack_query3_hash, pack_query3) = zip_elements(&mut dsl, &list_query_specific[2])?;
    let (pack_unsorted_hash, pack_unsorted) = zip_elements(&mut dsl, &list_unsorted)?;

    cache.insert("shared_information".to_string(), pack_shared_information);
    cache.insert("query1".to_string(), pack_query1);
    cache.insert("query2".to_string(), pack_query2);
    cache.insert("query3".to_string(), pack_query3);
    cache.insert("unsorted".to_string(), pack_unsorted);

    dsl.set_program_output("hash", pack_shared_information_hash)?;
    dsl.set_program_output("hash", pack_query1_hash)?;
    dsl.set_program_output("hash", pack_query2_hash)?;
    dsl.set_program_output("hash", pack_query3_hash)?;
    dsl.set_program_output("hash", pack_unsorted_hash)?;

    Ok(dsl)
}
