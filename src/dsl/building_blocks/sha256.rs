use crate::dsl::building_blocks::qm31::{
    reformat_qm31_from_dsl_element, reformat_qm31_to_dsl_element,
};
use crate::dsl::utils::draw_hints_to_memory_entries;
use anyhow::Result;
use bitcoin_circle_stark::channel::{ChannelWithHint, Sha256ChannelGadget};
use bitcoin_circle_stark::treepp::*;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{
    FunctionMetadata, FunctionOutput, FunctionWithOptionsMetadata,
};
use bitcoin_script_dsl::options::Options;
use stwo_prover::core::channel::{Channel, Sha256Channel};
use stwo_prover::core::vcs::sha256_hash::{Sha256Hash, Sha256Hasher};

pub fn mix_digest(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let old_channel_digest = dsl.get_str(inputs[0])?.to_vec();
    let digest = dsl.get_str(inputs[1])?;

    let new_digest = Sha256Hasher::concat_and_hash(
        &Sha256Hash::from(digest),
        &Sha256Hash::from(old_channel_digest),
    );

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(new_digest.as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

pub fn mix_digest_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_SWAP
        { Sha256ChannelGadget::mix_digest() }
    })
}

pub fn draw_felt(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let old_channel_digest = dsl.get_str(inputs[0])?;

    let mut channel = Sha256Channel::default();
    channel.update_digest(Sha256Hash::from(old_channel_digest));
    let (felt, hint) = channel.draw_felt_and_hints();

    Ok(FunctionOutput {
        new_elements: vec![
            MemoryEntry::new("hash", Element::Str(channel.digest().as_ref().to_vec())),
            MemoryEntry::new("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(felt))),
        ],
        new_hints: draw_hints_to_memory_entries(hint),
    })
}

pub fn draw_felt_gadget(_: &[usize]) -> Result<Script> {
    Ok(Sha256ChannelGadget::draw_felt_with_hint())
}

pub fn mix_felt(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let old_channel_digest = dsl.get_str(inputs[0])?.to_vec();
    let felt = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[1])?);

    let mut channel = Sha256Channel::default();
    channel.update_digest(Sha256Hash::from(old_channel_digest));
    channel.mix_felts(&[felt]);

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(channel.digest().as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

pub fn mix_felt_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        4 OP_ROLL
        { Sha256ChannelGadget::mix_felt() }
    })
}

fn draw_8_numbers(dsl: &mut DSL, inputs: &[usize], options: &Options) -> Result<FunctionOutput> {
    let logn = options.get_u32("logn")? as usize;

    let old_channel_digest = dsl.get_str(inputs[0])?.to_vec();

    let mut channel = Sha256Channel::default();
    channel.update_digest(Sha256Hash::from(old_channel_digest));

    let (queries, hints) = channel.draw_queries_and_hints(8, logn);
    assert_eq!(hints.0.len(), 8);
    assert!(hints.1.is_empty());

    let mut new_elements = vec![];
    new_elements.push(MemoryEntry::new(
        "hash",
        Element::Str(channel.digest().as_ref().to_vec()),
    ));
    for query in queries {
        new_elements.push(MemoryEntry::new("position", Element::Num(query as i32)));
    }

    let new_hints = draw_hints_to_memory_entries(hints);

    Ok(FunctionOutput {
        new_elements,
        new_hints,
    })
}

fn draw_8_numbers_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let logn = options.get_u32("logn")? as usize;

    Ok(script! {
        { Sha256ChannelGadget::draw_numbers_with_hint(8, logn) }
        for _ in 0..8 {
            8 OP_ROLL
        }
    })
}

pub(crate) fn load_functions(dsl: &mut DSL) {
    dsl.add_function(
        "mix_digest",
        FunctionMetadata {
            trace_generator: mix_digest,
            script_generator: mix_digest_gadget,
            input: vec!["hash", "hash"],
            output: vec!["hash"],
        },
    );
    dsl.add_function(
        "draw_felt",
        FunctionMetadata {
            trace_generator: draw_felt,
            script_generator: draw_felt_gadget,
            input: vec!["hash"],
            output: vec!["hash", "qm31"],
        },
    );
    dsl.add_function(
        "mix_felt",
        FunctionMetadata {
            trace_generator: mix_felt,
            script_generator: mix_felt_gadget,
            input: vec!["hash", "qm31"],
            output: vec!["hash"],
        },
    );
    dsl.add_function(
        "draw_8_numbers",
        FunctionWithOptionsMetadata {
            trace_generator: draw_8_numbers,
            script_generator: draw_8_numbers_gadget,
            input: vec!["hash"],
            output: vec![
                "hash", "position", "position", "position", "position", "position", "position",
                "position", "position",
            ],
        },
    );
}

#[cfg(test)]
mod test {
    use crate::dsl::building_blocks::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::channel::ChannelWithHint;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::options::Options;
    use bitcoin_script_dsl::test_program;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::channel::{Channel, Sha256Channel};
    use stwo_prover::core::vcs::sha256_hash::{Sha256Hash, Sha256Hasher};

    #[test]
    fn test_mix_digest() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut init_state = [0u8; 32];
        init_state.iter_mut().for_each(|v| *v = prng.gen());
        let init_state = Sha256Hash::from(init_state.to_vec());

        let mut elem = [0u8; 32];
        elem.iter_mut().for_each(|v| *v = prng.gen());
        let elem = Sha256Hash::from(elem.to_vec());

        let new_hash = Sha256Hasher::concat_and_hash(&elem, &init_state);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let old_channel_digest = dsl
            .alloc_input("hash", Element::Str(init_state.as_ref().to_vec()))
            .unwrap();

        let elem = dsl
            .alloc_input("hash", Element::Str(elem.as_ref().to_vec()))
            .unwrap();

        let res = dsl
            .execute("mix_digest", &[old_channel_digest, elem])
            .unwrap()[0];

        dsl.set_program_output("hash", res).unwrap();

        test_program(
            dsl,
            script! {
                { new_hash }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_draw_felt() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut init_state = [0u8; 32];
        init_state.iter_mut().for_each(|v| *v = prng.gen());
        let init_state = Sha256Hash::from(init_state.to_vec());

        let mut channel = Sha256Channel::default();
        channel.update_digest(init_state);
        let b = channel.draw_felt();
        let c = channel.digest;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let old_channel_digest = dsl
            .alloc_input("hash", Element::Str(init_state.as_ref().to_vec()))
            .unwrap();

        let res = dsl.execute("draw_felt", &[old_channel_digest]).unwrap();

        dsl.set_program_output("hash", res[0]).unwrap();
        dsl.set_program_output("qm31", res[1]).unwrap();

        test_program(
            dsl,
            script! {
                { c }
                { b }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_mix_felt() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut init_state = [0u8; 32];
        init_state.iter_mut().for_each(|v| *v = prng.gen());
        let init_state = Sha256Hash::from(init_state.to_vec());

        let mut channel = Sha256Channel::default();
        channel.update_digest(init_state);

        let felt = get_rand_qm31(&mut prng);

        channel.mix_felts(&[felt]);

        let after = channel.digest().as_ref().to_vec();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let old_channel_digest = dsl
            .alloc_input("hash", Element::Str(init_state.as_ref().to_vec()))
            .unwrap();
        let felt_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(felt)))
            .unwrap();

        let res = dsl
            .execute("mix_felt", &[old_channel_digest, felt_var])
            .unwrap()[0];

        assert_eq!(dsl.get_str(res).unwrap(), after);

        dsl.set_program_output("hash", res).unwrap();

        test_program(
            dsl,
            script! {
                { after }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_draw_8_numbers() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut init_state = [0u8; 32];
        init_state.iter_mut().for_each(|v| *v = prng.gen());
        let init_state = Sha256Hash::from(init_state.to_vec());

        let mut channel = Sha256Channel::default();
        channel.update_digest(init_state);

        let (numbers, _) = channel.draw_queries_and_hints(8, 12);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let old_channel_digest = dsl
            .alloc_input("hash", Element::Str(init_state.as_ref().to_vec()))
            .unwrap();
        let res = dsl
            .execute_with_options(
                "draw_8_numbers",
                &[old_channel_digest],
                &Options::new().with_u32("logn", 12),
            )
            .unwrap();

        assert_eq!(dsl.get_str(res[0]).unwrap(), channel.digest().as_ref());
        assert_eq!(res.len(), numbers.len() + 1);

        for (&idx, &number) in res.iter().skip(1).zip(numbers.iter()) {
            assert_eq!(dsl.get_num(idx).unwrap(), number as i32);
        }

        dsl.set_program_output("hash", res[0]).unwrap();
        for &idx in res.iter().skip(1) {
            dsl.set_program_output("position", idx).unwrap();
        }

        test_program(
            dsl,
            script! {
                { channel.digest.as_ref().to_vec() }
                for number in numbers {
                    { number }
                }
            },
        )
        .unwrap()
    }
}
