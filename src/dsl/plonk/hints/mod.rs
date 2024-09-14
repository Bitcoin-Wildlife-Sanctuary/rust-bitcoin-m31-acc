use crate::dsl::plonk::hints::fiat_shamir::FiatShamirHints;
use stwo_prover::core::channel::Sha256Channel;
use stwo_prover::core::pcs::PcsConfig;
use stwo_prover::examples::plonk::prove_fibonacci_plonk;

pub const LOG_N_ROWS: u32 = 5;

mod fiat_shamir;

pub struct Hints {
    pub fiat_shamir_hints: FiatShamirHints,
}

impl Hints {
    pub fn instance() -> Self {
        let config = PcsConfig::default();

        let (plonk_component, proof) = prove_fibonacci_plonk(LOG_N_ROWS, config);

        let mut channel = Sha256Channel::default();

        let fiat_shamir_hints = fiat_shamir::compute_fiat_shamir_hints(
            proof.clone(),
            &mut channel,
            &plonk_component,
            config,
        )
        .unwrap();

        Hints { fiat_shamir_hints }
    }
}
