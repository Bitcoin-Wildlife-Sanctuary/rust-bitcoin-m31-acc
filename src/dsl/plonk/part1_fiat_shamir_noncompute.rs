use crate::dsl::building_blocks::point::get_random_point_skipped;
use crate::dsl::building_blocks::qm31::{
    reformat_qm31_from_dsl_element, reformat_qm31_to_dsl_element,
};
use crate::dsl::fibonacci::hints::FIB_LOG_SIZE;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::Zipper;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use bitcoin_script_dsl::options::Options;
use itertools::Itertools;
use std::collections::HashMap;
use stwo_prover::core::channel::Sha256Channel;
use stwo_prover::core::prover::{LOG_BLOWUP_FACTOR, PROOF_OF_WORK_BITS};

pub fn generate_dsl(hints: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    let channel = &mut Sha256Channel::default();

    // Step 1: mix the channel with the trace commitment
    let mut channel_var =
        dsl.alloc_constant("hash", Element::Str(channel.digest().as_ref().to_vec()))?;
    let trace_commitment_var = dsl.alloc_hint(
        "hash",
        Element::Str(hints.fiat_shamir_hints.commitments[0].as_ref().to_vec()),
    )?;

    channel_var = dsl.execute("mix_digest", &[channel_var, trace_commitment_var])?[0];

    // Step 2: derive the z and alpha
    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let z_var = res[1];

    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let alpha_var = res[1];

    // Step 3: mix the channel with the interaction commitment and constant commitment
    let interaction_commitment_var = dsl.alloc_hint(
        "hash",
        Element::Str(hints.fiat_shamir_hints.commitments[1].as_ref().to_vec()),
    )?;

    let constant_commitment_var = dsl.alloc_hint(
        "hash",
        Element::Str(hints.fiat_shamir_hints.commitments[2].as_ref().to_vec()),
    )?;

    channel_var = dsl.execute("mix_digest", &[channel_var, interaction_commitment_var])?[0];
    channel_var = dsl.execute("mix_digest", &[channel_var, constant_commitment_var])?[0];

    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let composition_fold_random_coeff_var = res[1];

    // Step 4: mix the channel with composition commitment
    let composition_commitment_var = dsl.alloc_hint(
        "hash",
        Element::Str(hints.fiat_shamir_hints.commitments[3].as_ref().to_vec()),
    )?;
    channel_var = dsl.execute("mix_digest", &[channel_var, composition_commitment_var])?[0];

    // Step 5: save a copy of the channel before drawing the OODS point draw (for deferred computation)
    let before_oods_channel_var = channel_var;
    channel_var = get_random_point_skipped(&mut dsl, channel_var)?;

    // Step 6: mix the channel with the trace, interaction, constant, composition values
    let mut trace_oods_values_vars = vec![];
    assert_eq!(hints.fiat_shamir_hints.trace_oods_values.len(), 4);
    for &trace_oods_value in hints.fiat_shamir_hints.trace_oods_values.iter() {
        trace_oods_values_vars.push(dsl.alloc_hint(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(trace_oods_value)),
        )?);
    }

    let mut interaction_oods_values_vars = vec![];
    assert_eq!(hints.fiat_shamir_hints.interaction_oods_values.len(), 12);
    for &interaction_oods_value in hints.fiat_shamir_hints.interaction_oods_values.iter() {
        interaction_oods_values_vars.push(dsl.alloc_hint(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(interaction_oods_value)),
        )?);
    }

    let mut constant_oods_values_vars = vec![];
    assert_eq!(hints.fiat_shamir_hints.constant_oods_values.len(), 4);
    for &constant_oods_value in hints.fiat_shamir_hints.constant_oods_values.iter() {
        constant_oods_values_vars.push(dsl.alloc_hint(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(constant_oods_value)),
        )?);
    }

    let mut composition_oods_raw_values_vars = vec![];
    assert_eq!(hints.fiat_shamir_hints.constant_oods_values.len(), 4);
    for &composition_oods_raw_value in hints.fiat_shamir_hints.composition_oods_values.iter() {
        composition_oods_raw_values_vars.push(dsl.alloc_hint(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(composition_oods_raw_value)),
        )?);
    }

    for &trace_oods_value_var in trace_oods_values_vars.iter() {
        channel_var = dsl.execute("mix_felt", &[channel_var, trace_oods_value_var])?[0];
    }
    for &interaction_oods_value_var in interaction_oods_values_vars.iter() {
        channel_var = dsl.execute("mix_felt", &[channel_var, interaction_oods_value_var])?[0];
    }
    for &constant_oods_value_var in constant_oods_values_vars.iter() {
        channel_var = dsl.execute("mix_felt", &[channel_var, constant_oods_value_var])?[0];
    }
    for &composition_oods_raw_value_var in composition_oods_raw_values_vars.iter() {
        channel_var = dsl.execute("mix_felt", &[channel_var, composition_oods_raw_value_var])?[0];
    }

    // Step 7: derive line_batch_random_coeff and fri_fold_random_coeff
    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let line_batch_random_coeff_var = res[1];

    let res = dsl.execute("draw_felt", &[channel_var])?;
    channel_var = res[0];
    let fri_fold_random_coeff_var = res[1];

    // Step 8: get the FRI trees' commitments, mix them with the channel one by one, and obtain the folding alphas
    let mut fri_tree_commitments_vars = vec![];
    let mut folding_alphas_vars = vec![];
    for fri_tree_commitment in hints.fiat_shamir_hints.fri_layer_commitments.iter() {
        let fri_tree_commitment_var =
            dsl.alloc_hint("hash", Element::Str(fri_tree_commitment.as_ref().to_vec()))?;
        fri_tree_commitments_vars.push(fri_tree_commitment_var);
        channel_var = dsl.execute("mix_digest", &[channel_var, fri_tree_commitment_var])?[0];
        let res = dsl.execute("draw_felt", &[channel_var])?;
        channel_var = res[0];
        let folding_alpha_var = res[1];
        folding_alphas_vars.push(folding_alpha_var);
    }

    // Step 9: get the last layer and mix it with the channel
    let last_layer_var = dsl.alloc_hint(
        "qm31",
        Element::ManyNum(reformat_qm31_to_dsl_element(
            hints.fiat_shamir_hints.last_layer,
        )),
    )?;
    channel_var = dsl.execute("mix_felt", &[channel_var, last_layer_var])?[0];

    // Step 10: check proof of work
    channel_var = dsl.execute_with_options(
        "verify_pow",
        &[channel_var],
        &Options::new()
            .with_u32("n_bits", PROOF_OF_WORK_BITS)
            .with_u64("nonce", hints.fiat_shamir_hints.pow_hint.nonce),
    )?[0];

    // Step 11: draw all the queries
    let mut queries = dsl.execute_with_options(
        "draw_8_numbers",
        &[channel_var],
        &Options::new().with_u32("logn", FIB_LOG_SIZE + LOG_BLOWUP_FACTOR + 1),
    )?;
    let _ = queries.remove(0);
    // at this moment, the channel is no longer needed.

    // Step 12: query the trace commitment on the queries
    let mut trace_mult_queried_results = vec![];
    let mut trace_a_val_queried_results = vec![];
    let mut trace_b_val_queried_results = vec![];
    let mut trace_c_val_queried_results = vec![];
    for (&query, proof) in queries
        .iter()
        .zip(hints.fiat_shamir_hints.merkle_proofs_traces.iter())
    {
        let res = dsl.execute_with_options(
            "merkle_twin_tree_4",
            &[trace_commitment_var, query],
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
        trace_mult_queried_results.push((res[0], res[4]));
        trace_a_val_queried_results.push((res[1], res[5]));
        trace_b_val_queried_results.push((res[2], res[6]));
        trace_c_val_queried_results.push((res[3], res[7]));
    }

    Ok(dsl)
}