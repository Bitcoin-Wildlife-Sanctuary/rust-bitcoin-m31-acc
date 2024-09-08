use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<&str, Zipper>) -> Result<DSL> {
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
    assert_eq!(res.len(), 92);

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
        for _ in 0..5 {
            tmp.push(*res_iter.next().unwrap());
        }
        twiddles_vars.push(tmp);
    }

    assert!(res_iter.next().is_none());

    list_shared_information.push(circle_poly_alpha_var);
    list_shared_information.push(last_layer_var);
    list_shared_information.extend_from_slice(&fri_tree_commitments_vars);
    list_shared_information.extend_from_slice(&folding_alphas_vars);

    let (pack_shared_information_hash, pack_shared_information) =
        zip_elements(&mut dsl, &list_shared_information)?;

    /*let mut folding_intermediate_results_vars = vec![];
    for (&query, fold_hints) in queries.iter().zip(hints.per_query_fold_hints.iter()) {
        let mut tmp = vec![];
        for (&commitment_var, proof) in fri_tree_commitments_vars.iter().zip(fold_hints.twin_proofs.iter()) {
            tmp.push((res[0], res[1]));
        }
        folding_intermediate_results_vars.push(tmp);
    }*/

    cache.insert("shared_information", pack_shared_information);
    dsl.set_program_output("hash", pack_shared_information_hash)?;

    Ok(dsl)
}
