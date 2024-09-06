use fibonacci_example::fiat_shamir::{
    compute_fiat_shamir_hints, FiatShamirHints, FiatShamirOutput,
};
use fibonacci_example::fold::{compute_fold_hints, PerQueryFoldHints};
use fibonacci_example::prepare::{compute_prepare_hints, PrepareHints};
use fibonacci_example::quotients::{compute_quotients_hints, PerQueryQuotientHint};
use fibonacci_example::FIB_LOG_SIZE;
use stwo_prover::core::channel::Sha256Channel;
use stwo_prover::core::fields::m31::{BaseField, M31};
use stwo_prover::core::fields::IntoSlice;
use stwo_prover::core::pcs::PcsConfig;
use stwo_prover::core::vcs::sha256_hash::Sha256Hasher;
use stwo_prover::core::vcs::sha256_merkle::Sha256MerkleChannel;
use stwo_prover::examples::fibonacci::Fibonacci;
use stwo_prover::trace_generation::commit_and_prove;

pub struct Hints {
    pub fiat_shamir_output: FiatShamirOutput,
    pub fiat_shamir_hints: FiatShamirHints,
    pub prepare_hints: PrepareHints,
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

        let (prepare_output, prepare_hints) =
            compute_prepare_hints(&fiat_shamir_output, &proof).unwrap();

        let (quotients_output, per_query_quotients_hints) =
            compute_quotients_hints(&fiat_shamir_output, &prepare_output);

        let per_query_fold_hints = compute_fold_hints(
            &proof.commitment_scheme_proof.fri_proof,
            &fiat_shamir_output,
            &prepare_output,
            &quotients_output,
        );

        Self {
            fiat_shamir_output,
            fiat_shamir_hints,
            prepare_hints,
            per_query_quotients_hints,
            per_query_fold_hints,
        }
    }
}
