use crate::dsl::plonk::hints::fiat_shamir::FiatShamirOutput;
use crate::dsl::plonk::hints::prepare::PrepareOutput;
use bitcoin_circle_stark::precomputed_merkle_tree::PrecomputedMerkleTreeProof;

#[derive(Default, Clone)]
/// Hint that repeats for each query.
pub struct PerQueryQuotientHint {
    /// Precomputed tree Merkle proofs.
    pub precomputed_merkle_proofs: Vec<PrecomputedMerkleTreeProof>,
}

/// Compute the quotients hints.
pub(crate) fn compute_quotients_hints(
    fs_output: &FiatShamirOutput,
    prepare_output: &PrepareOutput,
) -> Vec<PerQueryQuotientHint> {
    let mut hints = vec![];
    for (_, queries_parent) in fs_output.queries_parents.iter().enumerate() {
        let precomputed = prepare_output
            .precomputed_merkle_tree
            .query(queries_parent << 1);

        hints.push(PerQueryQuotientHint {
            precomputed_merkle_proofs: vec![precomputed.clone()],
        });
    }

    hints
}
