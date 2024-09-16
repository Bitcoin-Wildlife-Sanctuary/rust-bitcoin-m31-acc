use crate::dsl::plonk::hints::fiat_shamir::FiatShamirOutput;
use bitcoin_circle_stark::precomputed_merkle_tree::PrecomputedMerkleTree;
use stwo_prover::core::prover::{StarkProof, VerificationError};
use stwo_prover::core::vcs::sha256_merkle::Sha256MerkleHasher;

/// Prepare Output
pub struct PrepareOutput {
    /// Precomputed Merkle tree for point and twiddles.
    pub precomputed_merkle_tree: PrecomputedMerkleTree,
}

/// prepare output for quotients and verifier hints
pub fn compute_prepare_hints(
    fs_output: &FiatShamirOutput,
    _: &StarkProof<Sha256MerkleHasher>,
) -> Result<PrepareOutput, VerificationError> {
    let precomputed_merkle_tree = PrecomputedMerkleTree::new(
        (fs_output.max_column_log_degree_bound + fs_output.fri_log_blowup_factor - 1) as usize,
    );

    Ok(PrepareOutput {
        precomputed_merkle_tree,
    })
}
