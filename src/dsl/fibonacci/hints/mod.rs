use crate::dsl::fibonacci::hints::fiat_shamir::{compute_fiat_shamir_hints, FiatShamirHints};
use crate::dsl::fibonacci::hints::fold::{compute_fold_hints, PerQueryFoldHints};
use crate::dsl::fibonacci::hints::prepare::compute_prepare_hints;
use crate::dsl::fibonacci::hints::quotients::{compute_quotients_hints, PerQueryQuotientHint};
use stwo_prover::core::channel::Sha256Channel;
use stwo_prover::core::fields::m31::{BaseField, M31};
use stwo_prover::core::fields::IntoSlice;
use stwo_prover::core::pcs::PcsConfig;
use stwo_prover::core::vcs::sha256_hash::Sha256Hasher;
use stwo_prover::core::vcs::sha256_merkle::Sha256MerkleChannel;
use stwo_prover::examples::fibonacci::Fibonacci;
use stwo_prover::trace_generation::commit_and_prove;

/// The Fibonacci log size in this test.
pub const FIB_LOG_SIZE: u32 = 5;

mod fiat_shamir;
mod fold;
mod prepare;
mod quotients;

pub struct Hints {
    pub fiat_shamir_hints: FiatShamirHints,
    pub per_query_quotients_hints: Vec<PerQueryQuotientHint>,
    pub per_query_fold_hints: Vec<PerQueryFoldHints>,
}

impl Hints {
    pub fn instance() -> Self {
        let config = PcsConfig::default();

        let fib = Fibonacci::new(FIB_LOG_SIZE, M31::reduce(443693538));

        let trace = fib.get_trace();
        let channel = &mut Sha256Channel::default();
        channel.update_digest(Sha256Hasher::hash(BaseField::into_slice(&[fib
            .air
            .component
            .claim])));
        let proof =
            commit_and_prove::<_, Sha256MerkleChannel>(&fib.air, channel, vec![trace], config)
                .unwrap();

        let channel = &mut Sha256Channel::default();
        channel.update_digest(Sha256Hasher::hash(BaseField::into_slice(&[fib
            .air
            .component
            .claim])));
        let (fiat_shamir_output, fiat_shamir_hints) =
            compute_fiat_shamir_hints(proof.clone(), channel, &fib.air).unwrap();

        let prepare_output = compute_prepare_hints(&fiat_shamir_output, &proof).unwrap();

        let (quotients_output, per_query_quotients_hints) =
            compute_quotients_hints(&fiat_shamir_output, &prepare_output);

        let per_query_fold_hints = compute_fold_hints(
            &proof.commitment_scheme_proof.fri_proof,
            &fiat_shamir_output,
            &prepare_output,
            &quotients_output,
        );

        Self {
            fiat_shamir_hints,
            per_query_quotients_hints,
            per_query_fold_hints,
        }
    }
}
