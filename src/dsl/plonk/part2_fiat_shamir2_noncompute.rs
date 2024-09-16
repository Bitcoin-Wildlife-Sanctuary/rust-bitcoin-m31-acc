use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_circle_stark::precomputed_merkle_tree::{
    get_precomputed_merkle_tree_roots, PRECOMPUTED_MERKLE_TREE_ROOTS,
};
use bitcoin_script_dsl::dsl::{Element, DSL};
use bitcoin_script_dsl::options::Options;
use itertools::Itertools;
use std::collections::HashMap;

pub fn generate_dsl(hints: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir2_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("fiat_shamir2").unwrap().hash.as_ref().to_vec()),
    )?;

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

    // unzip `fiat_shamir2_hash`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir2_hash,
        cache.get("fiat_shamir2").unwrap(),
    )?;
    assert_eq!(res.len(), 13);

    let mut res = res.into_iter();

    let constant_commitment_var = res.next().unwrap();
    let composition_commitment_var = res.next().unwrap();
    let pack_trace_queried_results_hash = res.next().unwrap();
    let pack_interaction_queried_results_hash = res.next().unwrap();
    let pack_fri_hash = res.next().unwrap();
    let mut queries = vec![];
    for _ in 0..8 {
        queries.push(res.next().unwrap());
    }

    assert!(res.next().is_none());

    // Step 1: query the constant and composition commitments on the queries
    let mut constant_a_wire_queried_results = vec![];
    let mut constant_b_wire_queried_results = vec![];
    let mut constant_c_wire_queried_results = vec![];
    let mut constant_op_queried_results = vec![];
    for (&query, proof) in queries
        .iter()
        .zip(hints.fiat_shamir_hints.merkle_proofs_constants.iter())
    {
        let res = dsl.execute_with_options(
            "merkle_twin_tree_4",
            &[constant_commitment_var, query],
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
        constant_a_wire_queried_results.push((res[0], res[4]));
        constant_b_wire_queried_results.push((res[1], res[5]));
        constant_c_wire_queried_results.push((res[2], res[6]));
        constant_op_queried_results.push((res[3], res[7]));
    }

    let mut composition_queried_results = vec![];
    for (&query, proof) in queries
        .iter()
        .zip(hints.fiat_shamir_hints.merkle_proofs_compositions.iter())
    {
        let res = dsl.execute_with_options(
            "merkle_twin_tree_4",
            &[composition_commitment_var, query],
            &Options::new()
                .with_multi_u32("left", proof.left.iter().map(|x| x.0).collect_vec())
                .with_multi_u32("right", proof.right.iter().map(|x| x.0).collect_vec())
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

        composition_queried_results.push((left, right));
    }

    let mut list_constant_queried_results = vec![];
    for constant_a_wire_queried_result in constant_a_wire_queried_results.iter() {
        list_constant_queried_results.push(constant_a_wire_queried_result.0);
        list_constant_queried_results.push(constant_a_wire_queried_result.1);
    }
    for constant_b_wire_queried_result in constant_b_wire_queried_results.iter() {
        list_constant_queried_results.push(constant_b_wire_queried_result.0);
        list_constant_queried_results.push(constant_b_wire_queried_result.1);
    }
    for constant_c_wire_queried_result in constant_c_wire_queried_results.iter() {
        list_constant_queried_results.push(constant_c_wire_queried_result.0);
        list_constant_queried_results.push(constant_c_wire_queried_result.1);
    }
    for constant_op_queried_result in constant_op_queried_results.iter() {
        list_constant_queried_results.push(constant_op_queried_result.0);
        list_constant_queried_results.push(constant_op_queried_result.1);
    }

    let (pack_constant_queried_results_hash, pack_constant_queried_results) =
        zip_elements(&mut dsl, &list_constant_queried_results)?;

    cache.insert(
        "constant_queried_results".to_string(),
        pack_constant_queried_results,
    );

    let mut list_composition_queried_results = vec![];
    for composition_queried_result in composition_queried_results {
        list_composition_queried_results.push(composition_queried_result.0);
        list_composition_queried_results.push(composition_queried_result.1);
    }

    let (pack_composition_queried_results_hash, pack_composition_queried_results) =
        zip_elements(&mut dsl, &list_composition_queried_results)?;

    cache.insert(
        "composition_queried_results".to_string(),
        pack_composition_queried_results,
    );

    // Step 2: handle the precomputed twiddle factors for each queries
    let mut twiddles_vars = vec![];

    let precomputed_merkle_tree_roots =
        PRECOMPUTED_MERKLE_TREE_ROOTS.get_or_init(get_precomputed_merkle_tree_roots);

    for (&query, pre_query_quotients_hint) in
        queries.iter().zip(hints.per_query_quotients_hints.iter())
    {
        let proof = &pre_query_quotients_hint.precomputed_merkle_proofs[0];

        let res = dsl.execute_with_options(
            "precomputed_merkle_tree_15",
            &[query],
            &Options::new()
                .with_binary(
                    "root_hash",
                    precomputed_merkle_tree_roots.get(&15).unwrap().to_vec(),
                )
                .with_u32("circle_point_x", proof.circle_point.x.0)
                .with_u32("circle_point_y", proof.circle_point.y.0)
                .with_multi_u32(
                    "twiddles",
                    proof.twiddles_elements.iter().map(|x| x.0).collect_vec(),
                )
                .with_multi_binary(
                    "siblings",
                    proof.siblings.iter().map(|x| x.to_vec()).collect_vec(),
                ),
        )?;

        let mut picked_res = vec![];
        picked_res.extend_from_slice(&res[res.len() - 8..res.len() - 3]);
        picked_res.extend_from_slice(&res[res.len() - 2..res.len()]);
        twiddles_vars.push(picked_res);
    }

    let mut list_twiddles = vec![];
    for twiddles_var in twiddles_vars.iter() {
        list_twiddles.extend_from_slice(&twiddles_var);
    }

    let (pack_twiddles_hash, pack_twiddles) = zip_elements(&mut dsl, &list_twiddles)?;

    cache.insert("twiddles".to_string(), pack_twiddles);

    let list_fiat_shamir2_result = vec![
        pack_trace_queried_results_hash,
        pack_interaction_queried_results_hash,
        pack_constant_queried_results_hash,
        pack_composition_queried_results_hash,
        pack_twiddles_hash,
        pack_fri_hash,
    ];

    let (pack_fiat_shamir2_result_hash, pack_fiat_shamir2_result) =
        zip_elements(&mut dsl, &list_fiat_shamir2_result)?;

    cache.insert("fiat_shamir2_result".to_string(), pack_fiat_shamir2_result);

    // fiat_shamir1_result
    // - z_var
    // - alpha_var
    // - composition_fold_random_coeff_var
    // - before_oods_channel_var
    // - line_batch_random_coeff_var
    // - fri_fold_random_coeff_var
    // - trace_oods_values_vars
    // - interaction_oods_values_vars
    // - constant_oods_values_vars
    // - composition_oods_raw_values_vars
    //
    // fiat_shamir2_result
    // - trace_queried_results_hash
    // - interaction_queried_results_hash
    // - constant_queried_results_hash
    // - composition_queried_results_hash
    // - twiddles_hash
    // - fri_hash

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", pack_fiat_shamir2_result_hash)?;

    Ok(dsl)
}
