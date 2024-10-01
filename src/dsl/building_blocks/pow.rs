use crate::algorithms::utils::OP_HINT;
use crate::dsl::framework::dsl::{Element, MemoryEntry, DSL};
use crate::dsl::framework::functions::{FunctionOutput, FunctionWithOptionsMetadata};
use crate::dsl::framework::options::Options;
use anyhow::{Error, Result};
use bitcoin_circle_stark::pow::{PoWHint, PowGadget};
use bitcoin_circle_stark::treepp::*;
use stwo_prover::core::channel::{Channel, Sha256Channel};
use stwo_prover::core::vcs::sha256_hash::Sha256Hash;

fn verify_pow(dsl: &mut DSL, inputs: &[usize], options: &Options) -> Result<FunctionOutput> {
    let old_channel_digest = dsl.get_str(inputs[0])?;

    let n_bits = options.get_u32("n_bits")?;
    let nonce = options.get_u64("nonce")?;

    let pow_hint = PoWHint::new(Sha256Hash::from(old_channel_digest), nonce, n_bits);

    let mut channel = Sha256Channel::default();
    channel.update_digest(Sha256Hash::from(old_channel_digest));
    channel.mix_nonce(nonce);
    if channel.trailing_zeros() < n_bits {
        return Err(Error::msg("The proof of work requirement is not satisfied"));
    }

    let new_hints = vec![
        MemoryEntry::new(
            "internal",
            Element::Str(pow_hint.nonce.to_le_bytes().to_vec()),
        ),
        MemoryEntry::new("internal", Element::Str(pow_hint.prefix)),
        MemoryEntry::new(
            "internal",
            Element::Str(vec![pow_hint.msb.unwrap_or_default()]),
        ),
        // if msb is not required, still push a stack element to make sure that the max stack consumption
        // is data-independent
    ];

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(channel.digest().as_ref().to_vec()),
        )],
        new_hints,
    })
}

fn verify_pow_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let n_bits = options.get_u32("n_bits")?;

    // NOTE: nonce should not be assumed to be a constant in the script.
    Ok(script! {
        { PowGadget::verify_pow(n_bits) }
        if n_bits % 8 == 0 {
            OP_HINT OP_DROP
        }
        // drop the dummy msb element if it is not needed
    })
}

pub(crate) fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "verify_pow",
        FunctionWithOptionsMetadata {
            trace_generator: verify_pow,
            script_generator: verify_pow_gadget,
            input: vec!["hash"],
            output: vec!["hash"],
        },
    )
}

#[cfg(test)]
mod test {
    use crate::dsl::framework::dsl::{Element, DSL};
    use crate::dsl::framework::options::Options;
    use crate::dsl::framework::test_program;
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::channel::{Channel, Sha256Channel};

    fn grind_find_nonce(channel_digest: Vec<u8>, n_bits: u32) -> u64 {
        let mut nonce = 0u64;

        let mut channel = Sha256Channel::default();
        channel.update_digest(channel_digest.into());

        loop {
            let mut channel = channel.clone();
            channel.mix_nonce(nonce);
            if channel.trailing_zeros() >= n_bits {
                return nonce;
            }
            nonce += 1;
        }
    }

    #[test]
    fn test_verify_pow() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut init_state = vec![0u8; 32];
        prng.fill_bytes(&mut init_state);

        let nonce = grind_find_nonce(init_state.clone(), 10);

        let mut channel = Sha256Channel::default();
        channel.update_digest(init_state.clone().into());
        channel.mix_nonce(nonce);

        let expected = channel.digest();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let old_channel_digest = dsl.alloc_input("hash", Element::Str(init_state)).unwrap();

        let new_channel_digest = dsl
            .execute_with_options(
                "verify_pow",
                &[old_channel_digest],
                &Options::new()
                    .with_u64("nonce", nonce)
                    .with_u32("n_bits", 10),
            )
            .unwrap()[0];

        assert_eq!(dsl.get_str(new_channel_digest).unwrap(), expected.as_ref());

        dsl.set_program_output("hash", new_channel_digest).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap();
    }
}
