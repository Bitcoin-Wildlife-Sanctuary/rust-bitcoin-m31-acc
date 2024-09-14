use bitcoin_circle_stark::fri::QueriesWithHint;
use bitcoin_circle_stark::merkle_tree::MerkleTreeTwinProof;
use bitcoin_circle_stark::pow::PoWHint;
use itertools::Itertools;
use stwo_prover::core::air::accumulation::PointEvaluationAccumulator;
use stwo_prover::core::air::{Air, ComponentProvers};
use stwo_prover::core::air::{AirProver, Component};
use stwo_prover::core::channel::{Channel, Sha256Channel};
use stwo_prover::core::circle::{CirclePoint, Coset};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::{SecureField, QM31};
use stwo_prover::core::fields::secure_column::SECURE_EXTENSION_DEGREE;
use stwo_prover::core::fri::{
    get_opening_positions, CirclePolyDegreeBound, FriConfig, FriLayerVerifier,
    FriVerificationError, FOLD_STEP,
};
use stwo_prover::core::pcs::{CommitmentSchemeVerifier, PcsConfig, TreeVec};
use stwo_prover::core::poly::line::LineDomain;
use stwo_prover::core::prover::{
    InvalidOodsSampleStructure, StarkProof, VerificationError, LOG_BLOWUP_FACTOR,
    LOG_LAST_LAYER_DEGREE_BOUND, N_QUERIES, PROOF_OF_WORK_BITS,
};
use stwo_prover::core::queries::{Queries, SparseSubCircleDomain};
use stwo_prover::core::vcs::sha256_hash::{Sha256Hash, Sha256Hasher};
use stwo_prover::core::vcs::sha256_merkle::{Sha256MerkleChannel, Sha256MerkleHasher};
use stwo_prover::core::{ColumnVec, InteractionElements, LookupValues};
use stwo_prover::examples::fibonacci::air::FibonacciAir;
use stwo_prover::trace_generation::AirTraceGenerator;

#[derive(Clone)]
/// Hints for performing the Fiat-Shamir transform until finalizing the queries.
pub struct FiatShamirHints {
    /// Commitments from the proof.
    pub commitments: [Sha256Hash; 2],

    /// trace oods values.
    pub trace_oods_values: [SecureField; 3],

    /// composition odds raw values.
    pub composition_oods_values: [SecureField; 4],

    /// fri commits for deriving the folding parameter
    pub fri_commitments: Vec<Sha256Hash>,

    /// last layer poly (assuming only one element)
    pub last_layer: QM31,

    /// PoW hint
    pub pow_hint: PoWHint,

    /// Merkle proofs for the trace Merkle tree.
    pub merkle_proofs_traces: Vec<MerkleTreeTwinProof>,

    /// Merkle proofs for the composition Merkle tree.
    pub merkle_proofs_compositions: Vec<MerkleTreeTwinProof>,
}

/// Fiat Shamir hints along with fri inputs
pub struct FiatShamirOutput {
    /// log blowup factor
    pub fri_log_blowup_factor: u32,

    /// log degree bound of column
    pub max_column_log_degree_bound: u32,

    /// log sizes of commitment scheme columns
    pub commitment_scheme_column_log_sizes: TreeVec<ColumnVec<u32>>,

    /// trace sample points and odds points
    pub sampled_points: TreeVec<Vec<Vec<CirclePoint<QM31>>>>,

    /// sample values
    pub sample_values: Vec<Vec<Vec<QM31>>>,

    /// random coefficient
    pub random_coeff: QM31,

    /// alpha
    pub circle_poly_alpha: QM31,

    /// folding alphas
    pub folding_alphas: Vec<QM31>,

    /// query subcircle domain.
    pub query_subcircle_domain: SparseSubCircleDomain,

    /// queries' parent indices.
    pub queries_parents: Vec<usize>,

    /// queried values on the leaves on the left.
    pub queried_values_left: Vec<Vec<M31>>,

    /// queried values on the leaves on the right.
    pub queried_values_right: Vec<Vec<M31>>,

    /// last layer.
    pub last_layer: QM31,

    /// fri commits for deriving the folding parameter
    pub fri_commitments: Vec<Sha256Hash>,
}

/// Generate Fiat Shamir hints along with fri inputs
pub fn compute_fiat_shamir_hints(
    proof: StarkProof<Sha256MerkleHasher>,
    channel: &mut Sha256Channel,
    air: &FibonacciAir,
) -> Result<(FiatShamirOutput, FiatShamirHints), VerificationError> {
    let config = PcsConfig::default();
    // Read trace commitment.
    let mut commitment_scheme: CommitmentSchemeVerifier<Sha256MerkleChannel> =
        CommitmentSchemeVerifier::new(config);

    let air_prover = air.to_air_prover();
    let components = ComponentProvers(air_prover.component_provers());

    // TODO(spapini): Retrieve column_log_sizes from AirTraceVerifier, and remove the dependency on
    // Air.
    let column_log_sizes = components.components().column_log_sizes();
    commitment_scheme.commit(proof.commitments[0], &column_log_sizes[0], channel);

    if column_log_sizes.len() == 2 {
        commitment_scheme.commit(proof.commitments[1], &column_log_sizes[1], channel);
    }

    channel.mix_felts(
        &proof
            .lookup_values
            .0
            .values()
            .map(|v| SecureField::from(*v))
            .collect_vec(),
    );

    let random_coeff = channel.draw_felt();

    // Read composition polynomial commitment.
    commitment_scheme.commit(
        *proof.commitments.last().unwrap(),
        &[air.composition_log_degree_bound(); 4],
        channel,
    );

    // Draw OODS point.
    let oods_point = CirclePoint::<SecureField>::get_random_point(channel);

    // Get mask sample points relative to oods point.
    let trace_sample_points = components.components().mask_points(oods_point);
    let masked_points = trace_sample_points.clone();

    // TODO(spapini): Change when we support multiple interactions.
    let mut sampled_points = components.components().mask_points(oods_point);
    // Add the composition polynomial mask points.
    sampled_points.push(vec![vec![oods_point]; SECURE_EXTENSION_DEGREE]);

    // this step is just a reorganization of the data
    assert_eq!(sampled_points.0[0][0][0], masked_points[0][0][0]);
    assert_eq!(sampled_points.0[0][0][1], masked_points[0][0][1]);
    assert_eq!(sampled_points.0[0][0][2], masked_points[0][0][2]);

    assert_eq!(sampled_points.0[1][0][0], oods_point);
    assert_eq!(sampled_points.0[1][1][0], oods_point);
    assert_eq!(sampled_points.0[1][2][0], oods_point);
    assert_eq!(sampled_points.0[1][3][0], oods_point);

    // TODO(spapini): Save clone.
    let (trace_oods_values, composition_oods_value) =
        sampled_values_to_mask(air, proof.commitment_scheme_proof.sampled_values.clone())
            .map_err(|_| {
                VerificationError::InvalidStructure(
                    "Unexpected sampled_values structure".to_string(),
                )
            })
            .unwrap();

    let mut evaluation_accumulator = PointEvaluationAccumulator::new(random_coeff);
    air.component.evaluate_constraint_quotients_at_point(
        oods_point,
        &trace_oods_values[0],
        &mut evaluation_accumulator,
        &InteractionElements::default(),
        &LookupValues::default(),
    );
    let oods_value = evaluation_accumulator.finalize();

    if composition_oods_value != oods_value {
        return Err(VerificationError::OodsNotMatching);
    }

    let sample_values = &proof.commitment_scheme_proof.sampled_values.0;

    channel.mix_felts(
        &proof
            .commitment_scheme_proof
            .sampled_values
            .clone()
            .flatten_cols(),
    );
    let random_coeff = channel.draw_felt();

    let bounds = commitment_scheme
        .column_log_sizes()
        .zip_cols(&sampled_points)
        .map_cols(|(log_size, sampled_points)| {
            vec![CirclePolyDegreeBound::new(log_size - LOG_BLOWUP_FACTOR); sampled_points.len()]
        })
        .flatten_cols()
        .into_iter()
        .sorted()
        .rev()
        .dedup()
        .collect_vec();

    // FRI commitment phase on OODS quotients.
    let fri_config = FriConfig::new(LOG_LAST_LAYER_DEGREE_BOUND, LOG_BLOWUP_FACTOR, N_QUERIES);

    // from fri-verifier
    let max_column_bound = bounds[0];
    let _ = max_column_bound.log_degree_bound + fri_config.log_blowup_factor;

    // Circle polynomials can all be folded with the same alpha.
    let circle_poly_alpha = channel.draw_felt();

    let mut inner_layers = Vec::new();
    let mut layer_bound = max_column_bound.fold_to_line();
    let mut layer_domain = LineDomain::new(Coset::half_odds(
        layer_bound.log_degree_bound + fri_config.log_blowup_factor,
    ));

    let mut fri_commitments = vec![];

    let mut folding_alphas = vec![];
    for (layer_index, proof) in proof
        .commitment_scheme_proof
        .fri_proof
        .inner_layers
        .into_iter()
        .enumerate()
    {
        channel.update_digest(Sha256Hasher::concat_and_hash(
            &proof.commitment,
            &channel.digest(),
        ));

        let folding_alpha = channel.draw_felt();
        folding_alphas.push(folding_alpha);

        fri_commitments.push(proof.commitment);

        inner_layers.push(FriLayerVerifier {
            degree_bound: layer_bound,
            domain: layer_domain,
            folding_alpha,
            layer_index,
            proof,
        });

        layer_bound = layer_bound
            .fold(FOLD_STEP)
            .ok_or(FriVerificationError::InvalidNumFriLayers)?;
        layer_domain = layer_domain.double();
    }

    if layer_bound.log_degree_bound != fri_config.log_last_layer_degree_bound {
        return Err(VerificationError::Fri(
            FriVerificationError::InvalidNumFriLayers,
        ));
    }

    let last_layer_poly = proof.commitment_scheme_proof.fri_proof.last_layer_poly;

    if last_layer_poly.len() > (1 << fri_config.log_last_layer_degree_bound) {
        return Err(VerificationError::Fri(
            FriVerificationError::LastLayerDegreeInvalid,
        ));
    }

    channel.mix_felts(&last_layer_poly);

    let pow_hint = PoWHint::new(
        channel.digest,
        proof.commitment_scheme_proof.proof_of_work,
        PROOF_OF_WORK_BITS,
    );

    // Verify proof of work.
    channel.mix_nonce(proof.commitment_scheme_proof.proof_of_work);
    if channel.trailing_zeros() < PROOF_OF_WORK_BITS {
        return Err(VerificationError::ProofOfWork);
    }

    let column_log_sizes = bounds
        .iter()
        .dedup()
        .map(|b| b.log_degree_bound + fri_config.log_blowup_factor)
        .collect_vec();

    let (queries, _) =
        Queries::generate_with_hints(channel, column_log_sizes[0], fri_config.n_queries);

    let fri_query_domains = get_opening_positions(&queries, &column_log_sizes);

    assert_eq!(fri_query_domains.len(), 1);
    let query_domain = fri_query_domains.first_key_value().unwrap();
    assert_eq!(
        *query_domain.0,
        max_column_bound.log_degree_bound + fri_config.log_blowup_factor
    );

    let queries_parents: Vec<usize> = query_domain
        .1
        .iter()
        .map(|subdomain| {
            assert_eq!(subdomain.log_size, 1);
            subdomain.coset_index
        })
        .collect();

    let merkle_proofs_traces = MerkleTreeTwinProof::from_stwo_proof(
        (max_column_bound.log_degree_bound + fri_config.log_blowup_factor) as usize,
        &queries_parents,
        &proof.commitment_scheme_proof.queried_values[0],
        &proof.commitment_scheme_proof.decommitments[0],
    );
    let merkle_proofs_compositions = MerkleTreeTwinProof::from_stwo_proof(
        (max_column_bound.log_degree_bound + fri_config.log_blowup_factor) as usize,
        &queries_parents,
        &proof.commitment_scheme_proof.queried_values[1],
        &proof.commitment_scheme_proof.decommitments[1],
    );

    for (&query, twin_proof) in queries_parents.iter().zip(merkle_proofs_traces.iter()) {
        assert!(twin_proof.verify(
            &proof.commitments[0],
            (max_column_bound.log_degree_bound + fri_config.log_blowup_factor) as usize,
            query << 1
        ));
    }

    for (&query, twin_proof) in queries_parents
        .iter()
        .zip(merkle_proofs_compositions.iter())
    {
        assert!(twin_proof.verify(
            &proof.commitments[1],
            (max_column_bound.log_degree_bound + fri_config.log_blowup_factor) as usize,
            query << 1
        ));
    }

    let mut queried_values_left = vec![];
    let mut queried_values_right = vec![];
    for (trace, composition) in merkle_proofs_traces
        .iter()
        .zip(merkle_proofs_compositions.iter())
    {
        let mut left_vec = vec![];
        let mut right_vec = vec![];
        for (&left, &right) in trace
            .left
            .iter()
            .zip(trace.right.iter())
            .chain(composition.left.iter().zip(composition.right.iter()))
        {
            left_vec.push(left);
            right_vec.push(right);
        }
        queried_values_left.push(left_vec);
        queried_values_right.push(right_vec);
    }

    let fiat_shamir_hints = FiatShamirHints {
        commitments: [proof.commitments[0], proof.commitments[1]],
        trace_oods_values: [
            sample_values[0][0][0],
            sample_values[0][0][1],
            sample_values[0][0][2],
        ],
        composition_oods_values: [
            sample_values[1][0][0],
            sample_values[1][1][0],
            sample_values[1][2][0],
            sample_values[1][3][0],
        ],
        fri_commitments: fri_commitments.clone(),
        last_layer: last_layer_poly.to_vec()[0],
        pow_hint,
        merkle_proofs_traces,
        merkle_proofs_compositions,
    };

    let fiat_shamir_output = FiatShamirOutput {
        fri_log_blowup_factor: fri_config.log_blowup_factor,
        max_column_log_degree_bound: max_column_bound.log_degree_bound,
        commitment_scheme_column_log_sizes: commitment_scheme.column_log_sizes(),
        sampled_points,
        sample_values: sample_values.to_vec(),
        random_coeff,
        circle_poly_alpha,
        folding_alphas,
        query_subcircle_domain: query_domain.1.clone(),
        queries_parents,
        queried_values_left,
        queried_values_right,
        last_layer: last_layer_poly.to_vec()[0],
        fri_commitments,
    };

    Ok((fiat_shamir_output, fiat_shamir_hints))
}

#[allow(clippy::type_complexity)]
fn sampled_values_to_mask(
    air: &impl Air,
    sampled_values: TreeVec<ColumnVec<Vec<SecureField>>>,
) -> Result<(Vec<TreeVec<Vec<Vec<SecureField>>>>, SecureField), InvalidOodsSampleStructure> {
    let mut sampled_values = sampled_values.as_ref();
    let composition_values = sampled_values.pop().ok_or(InvalidOodsSampleStructure)?;

    let mut sample_iters = sampled_values.map(|tree_value| tree_value.iter());
    let trace_oods_values = air
        .components()
        .iter()
        .map(|component| {
            component
                .mask_points(CirclePoint::zero())
                .zip(sample_iters.as_mut())
                .map(|(mask_per_tree, tree_iter)| {
                    tree_iter.take(mask_per_tree.len()).cloned().collect_vec()
                })
        })
        .collect_vec();

    let composition_oods_value = SecureField::from_partial_evals(
        composition_values
            .iter()
            .flatten()
            .cloned()
            .collect_vec()
            .try_into()
            .map_err(|_| InvalidOodsSampleStructure)?,
    );

    Ok((trace_oods_values, composition_oods_value))
}
