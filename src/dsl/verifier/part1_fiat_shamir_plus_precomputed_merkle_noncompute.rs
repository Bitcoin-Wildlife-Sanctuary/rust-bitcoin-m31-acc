use crate::dsl::building_blocks::point::get_random_point_skipped;
use crate::dsl::building_blocks::qm31::reformat_qm31_to_dsl_element;
use crate::dsl::load_data_types;
use crate::dsl::load_functions;
use crate::dsl::tools::{zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use anyhow::Result;
use bitcoin_circle_stark::precomputed_merkle_tree::{
    get_precomputed_merkle_tree_roots, PRECOMPUTED_MERKLE_TREE_ROOTS,
};
use bitcoin_script_dsl::dsl::{Element, DSL};
use bitcoin_script_dsl::options::Options;
use fibonacci_example::FIB_LOG_SIZE;
use itertools::Itertools;
use std::collections::HashMap;
use stwo_prover::core::channel::Sha256Channel;
use stwo_prover::core::fields::m31::{BaseField, M31};
use stwo_prover::core::fields::IntoSlice;
use stwo_prover::core::prover::{LOG_BLOWUP_FACTOR, PROOF_OF_WORK_BITS};
use stwo_prover::core::vcs::sha256_hash::Sha256Hasher;

pub fn generate_dsl(hints: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    let claim = M31::reduce(443693538);
    let channel = &mut Sha256Channel::default();
    channel.update_digest(Sha256Hasher::hash(BaseField::into_slice(&[claim])));

    // Step 1: mix the channel with the trace commitment
    let mut channel_var =
        dsl.alloc_constant("hash", Element::Str(channel.digest().as_ref().to_vec()))?;
    let first_commitment_var = dsl.alloc_hint(
        "hash",
        Element::Str(hints.fiat_shamir_hints.commitments[0].as_ref().to_vec()),
    )?;

    channel_var = dsl.execute("mix_digest", &[channel_var, first_commitment_var])?[0];

    // Step 2: draw the random coeff 1, which is used to assemble the composition polynomial (for deferred computation)
    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let random_coeff_1_var = res[1];

    // Step 3: mix the channel with the composition commitment
    let second_commitment_var = dsl.alloc_hint(
        "hash",
        Element::Str(hints.fiat_shamir_hints.commitments[1].as_ref().to_vec()),
    )?;
    channel_var = dsl.execute("mix_digest", &[channel_var, second_commitment_var])?[0];

    // Step 4: save a copy of the channel before drawing the OODS point draw (for deferred computation)
    let before_oods_channel_var = channel_var;
    channel_var = get_random_point_skipped(&mut dsl, channel_var)?;

    // Step 5: mix the channel with the trace values and composition values
    let mut trace_oods_values_vars = vec![];
    for &trace_oods_value in hints.fiat_shamir_hints.trace_oods_values.iter() {
        trace_oods_values_vars.push(dsl.alloc_hint(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(trace_oods_value)),
        )?);
    }
    let mut composition_oods_raw_values_vars = vec![];
    for &composition_oods_raw_value in hints.fiat_shamir_hints.composition_oods_values.iter() {
        composition_oods_raw_values_vars.push(dsl.alloc_hint(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(composition_oods_raw_value)),
        )?);
    }

    channel_var = dsl.execute("mix_felt", &[channel_var, trace_oods_values_vars[0]])?[0];
    channel_var = dsl.execute("mix_felt", &[channel_var, trace_oods_values_vars[1]])?[0];
    channel_var = dsl.execute("mix_felt", &[channel_var, trace_oods_values_vars[2]])?[0];
    channel_var = dsl.execute(
        "mix_felt",
        &[channel_var, composition_oods_raw_values_vars[0]],
    )?[0];
    channel_var = dsl.execute(
        "mix_felt",
        &[channel_var, composition_oods_raw_values_vars[1]],
    )?[0];
    channel_var = dsl.execute(
        "mix_felt",
        &[channel_var, composition_oods_raw_values_vars[2]],
    )?[0];
    channel_var = dsl.execute(
        "mix_felt",
        &[channel_var, composition_oods_raw_values_vars[3]],
    )?[0];

    // Step 6: draw the random coeff 2 (used for FRI) and circle poly alpha
    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let random_coeff_2_var = res[1];

    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let circle_poly_alpha_var = res[1];

    // Step 7: get the FRI trees' commitments, mix them with the channel one by one, and obtain the folding alphas
    let mut fri_tree_commitments_vars = vec![];
    let mut folding_alphas_vars = vec![];
    for (fri_tree_commitment, _) in hints
        .fiat_shamir_hints
        .fri_commitment_and_folding_hints
        .iter()
    {
        let fri_tree_commitment_var =
            dsl.alloc_hint("hash", Element::Str(fri_tree_commitment.as_ref().to_vec()))?;
        fri_tree_commitments_vars.push(fri_tree_commitment_var);
        channel_var = dsl.execute("mix_digest", &[channel_var, fri_tree_commitment_var])?[0];
        let res = dsl.execute("draw_felt", &[channel_var])?;
        channel_var = res[0];
        let folding_alpha_var = res[1];
        folding_alphas_vars.push(folding_alpha_var);
    }

    // Step 8: get the last layer and mix it with the channel
    let last_layer_var = dsl.alloc_hint(
        "qm31",
        Element::ManyNum(reformat_qm31_to_dsl_element(
            hints.fiat_shamir_hints.last_layer,
        )),
    )?;
    channel_var = dsl.execute("mix_felt", &[channel_var, last_layer_var])?[0];

    // Step 9: check proof of work
    channel_var = dsl.execute_with_options(
        "verify_pow",
        &[channel_var],
        &Options::new()
            .with_u32("n_bits", PROOF_OF_WORK_BITS)
            .with_u64("nonce", hints.fiat_shamir_hints.pow_hint.nonce),
    )?[0];

    // Step 10: draw all the queries
    let mut queries = dsl.execute_with_options(
        "draw_8_numbers",
        &[channel_var],
        &Options::new().with_u32("logn", FIB_LOG_SIZE + LOG_BLOWUP_FACTOR + 1),
    )?;
    let _ = queries.remove(0);
    // at this moment, the channel is no longer needed.

    // Step 11: query the trace commitment on the queries
    let mut trace_queried_results = vec![];
    for (&query, proof) in queries
        .iter()
        .zip(hints.fiat_shamir_hints.merkle_proofs_traces.iter())
    {
        let res = dsl.execute_with_options(
            "merkle_twin_tree_1",
            &[first_commitment_var, query],
            &Options::new()
                .with_multi_u32("left", vec![proof.left[0].0])
                .with_multi_u32("right", vec![proof.right[0].0])
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
        trace_queried_results.push((res[0], res[1]));
    }

    // Step 12: query the composition commitment on the queries
    let mut composition_queried_results = vec![];
    for (&query, proof) in queries
        .iter()
        .zip(hints.fiat_shamir_hints.merkle_proofs_compositions.iter())
    {
        let res = dsl.execute_with_options(
            "merkle_twin_tree_4",
            &[second_commitment_var, query],
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
        composition_queried_results.push((res[0], res[1]));
    }

    // Step 13: handle the precomputed twiddle factors for each queries
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

    // results of interest:
    // - random_coeff_1_var (for deferred check)
    //
    // - trace_oods_values_vars (for deferred check)
    // - composition_oods_raw_values_vars (for deferred check)
    //
    // - random_coeff_2_var
    // - circle_poly_alpha_var
    //
    // - fri_tree_commitments_vars
    // - folding alphas
    //
    // - last_layer_var
    // - before_oods_channel_var
    //
    // - queries
    // - trace_queried_results
    // - composition_queried_results
    // - twiddles_vars
    // (these four are for per-query)

    // decision:
    // - random_coeff_1_var (for deferred check)
    // - trace_oods_values_vars (for deferred check)
    // - composition_oods_raw_values_vars (for deferred check)
    // - before_oods_channel_var
    // - random_coeff_2_var
    //
    // - circle_poly_alpha_var
    // - last_layer_var
    // - fri_tree_commitments_vars
    // - folding alphas
    // - queries
    // - trace_queried_results
    // - composition_queried_results
    // - twiddles_vars

    let list_fiat_shamir_verify1 = [
        random_coeff_1_var,
        trace_oods_values_vars[0],
        trace_oods_values_vars[1],
        trace_oods_values_vars[2],
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
        before_oods_channel_var,
        random_coeff_2_var,
    ];

    let (pack_fiat_shamir_verify1_hash, pack_fiat_shamir_verify1) =
        zip_elements(&mut dsl, &list_fiat_shamir_verify1)?;

    let mut list_after_fiat_shamir = vec![circle_poly_alpha_var, last_layer_var];

    for &fri_tree_commitments_var in fri_tree_commitments_vars.iter() {
        list_after_fiat_shamir.push(fri_tree_commitments_var);
    }
    for &folding_alphas_var in folding_alphas_vars.iter() {
        list_after_fiat_shamir.push(folding_alphas_var);
    }

    for &query in queries.iter() {
        list_after_fiat_shamir.push(query);
    }
    for &trace_queried_result in trace_queried_results.iter() {
        list_after_fiat_shamir.push(trace_queried_result.0);
        list_after_fiat_shamir.push(trace_queried_result.1);
    }
    for &composition_queried_result in composition_queried_results.iter() {
        list_after_fiat_shamir.push(composition_queried_result.0);
        list_after_fiat_shamir.push(composition_queried_result.1);
    }
    for twiddles_vars_per_query in twiddles_vars.iter() {
        list_after_fiat_shamir.extend_from_slice(twiddles_vars_per_query);
    }

    let (pack_after_fiat_shamir_hash, pack_after_fiat_shamir) =
        zip_elements(&mut dsl, &list_after_fiat_shamir)?;

    cache.insert("fiat_shamir_verify1".to_string(), pack_fiat_shamir_verify1);
    cache.insert("after_fiat_shamir".to_string(), pack_after_fiat_shamir);

    dsl.set_program_output("hash", pack_fiat_shamir_verify1_hash)?;
    dsl.set_program_output("hash", pack_after_fiat_shamir_hash)?;

    Ok(dsl)
}
