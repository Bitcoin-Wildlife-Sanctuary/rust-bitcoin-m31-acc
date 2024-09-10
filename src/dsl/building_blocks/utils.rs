use crate::algorithms::utils::OP_HINT;
use anyhow::Result;
use bitcoin_circle_stark::treepp::*;
use bitcoin_script::script;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{
    FunctionMetadata, FunctionOutput, FunctionWithOptionsMetadata,
};
use bitcoin_script_dsl::options::Options;
use itertools::Itertools;

fn check_0_or_1() -> Script {
    script! {
        OP_DUP 0 OP_GREATERTHANOREQUAL OP_VERIFY
        OP_DUP 1 OP_LESSTHANOREQUAL OP_VERIFY
    }
}

fn decompose_positions_to_5(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let mut cur = dsl.get_num(inputs[0])? as usize;

    let mut hints = vec![];

    hints.push(cur & 1);
    cur >>= 1;
    let mut res = vec![];
    res.push(cur);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur);
    hints.push(cur);

    let new_elements = res
        .iter()
        .map(|&x| MemoryEntry::new("position", Element::Num(x as i32)))
        .collect_vec();
    let new_hints = hints
        .iter()
        .map(|&x| MemoryEntry::new("position", Element::Num(x as i32)))
        .collect_vec();

    Ok(FunctionOutput {
        new_elements,
        new_hints,
    })
}

fn decompose_positions_to_5_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_HINT check_0_or_1
        OP_HINT check_0_or_1
        OP_HINT check_0_or_1
        OP_HINT check_0_or_1
        OP_HINT check_0_or_1
        OP_HINT

        OP_DUP OP_TOALTSTACK
        OP_DUP OP_ADD OP_ADD OP_DUP OP_TOALTSTACK
        OP_DUP OP_ADD OP_ADD OP_DUP OP_TOALTSTACK
        OP_DUP OP_ADD OP_ADD OP_DUP OP_TOALTSTACK
        OP_DUP OP_ADD OP_ADD OP_DUP OP_TOALTSTACK
        OP_DUP OP_ADD OP_ADD
        OP_EQUALVERIFY

        for _ in 0..5 {
            OP_FROMALTSTACK
        }
    })
}

fn skip_one_and_extract_5_bits(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let mut cur = dsl.get_num(inputs[0])? as usize;

    let mut hints = vec![];
    hints.push(cur & 1);
    cur >>= 1;

    let mut res = vec![];
    res.push(cur & 1);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur & 1);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur & 1);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur & 1);
    hints.push(cur & 1);
    cur >>= 1;
    res.push(cur & 1);
    hints.push(cur & 1);
    cur >>= 1;
    hints.push(cur);

    let new_elements = res
        .iter()
        .map(|&x| MemoryEntry::new("position", Element::Num(x as i32)))
        .collect_vec();
    let new_hints = hints
        .iter()
        .map(|&x| MemoryEntry::new("position", Element::Num(x as i32)))
        .collect_vec();

    Ok(FunctionOutput {
        new_elements,
        new_hints,
    })
}

fn skip_one_and_extract_5_bits_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_HINT check_0_or_1
        OP_HINT check_0_or_1 OP_DUP OP_TOALTSTACK
        OP_HINT check_0_or_1 OP_DUP OP_TOALTSTACK
        OP_HINT check_0_or_1 OP_DUP OP_TOALTSTACK
        OP_HINT check_0_or_1 OP_DUP OP_TOALTSTACK
        OP_HINT check_0_or_1 OP_DUP OP_TOALTSTACK
        OP_HINT

        OP_DUP OP_ADD OP_ADD
        OP_DUP OP_ADD OP_ADD
        OP_DUP OP_ADD OP_ADD
        OP_DUP OP_ADD OP_ADD
        OP_DUP OP_ADD OP_ADD
        OP_DUP OP_ADD OP_ADD
        OP_EQUALVERIFY

        for _ in 0..5 {
            OP_FROMALTSTACK
        }
        OP_SWAP
        OP_ROT
        3 OP_ROLL
        4 OP_ROLL
    })
}

fn select_among_eight(
    dsl: &mut DSL,
    inputs: &[usize],
    options: &Options,
) -> Result<FunctionOutput> {
    let pick = options.get_u32("pick")? as usize;
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(dsl.get_str(inputs[pick - 1])?.to_vec()),
        )],
        new_hints: vec![],
    })
}

fn select_among_eight_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let pick = options.get_u32("pick")? as usize;
    Ok(script! {
        { 8 - pick } OP_PICK OP_TOALTSTACK
        OP_2DROP OP_2DROP OP_2DROP OP_2DROP
        OP_FROMALTSTACK
    })
}

pub(crate) fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "decompose_positions_to_5",
        FunctionMetadata {
            trace_generator: decompose_positions_to_5,
            script_generator: decompose_positions_to_5_gadget,
            input: vec!["position"],
            output: vec!["position", "position", "position", "position", "position"],
        },
    )?;
    dsl.add_function(
        "select_among_eight",
        FunctionWithOptionsMetadata {
            trace_generator: select_among_eight,
            script_generator: select_among_eight_gadget,
            input: vec![
                "hash", "hash", "hash", "hash", "hash", "hash", "hash", "hash",
            ],
            output: vec!["hash"],
        },
    )?;
    dsl.add_function(
        "skip_one_and_extract_5_bits",
        FunctionMetadata {
            trace_generator: skip_one_and_extract_5_bits,
            script_generator: skip_one_and_extract_5_bits_gadget,
            input: vec!["position"],
            output: vec!["position", "position", "position", "position", "position"],
        },
    )
}

#[cfg(test)]
mod test {
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script::script;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::options::Options;
    use bitcoin_script_dsl::test_program;
    use itertools::Itertools;
    use rand::{Rng, RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::vcs::sha256_hash::Sha256Hash;

    #[test]
    fn test_decompose_positions_to_5() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let num = prng.gen_range(0..=1023);

        let expected = vec![num >> 2, num >> 3, num >> 4, num >> 5, num >> 6];

        let mut dsl = DSL::new();
        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let num_var = dsl.alloc_input("position", Element::Num(num)).unwrap();

        let res = dsl.execute("decompose_positions_to_5", &[num_var]).unwrap();
        for (&a, &b) in expected.iter().zip(res.iter()) {
            assert_eq!(a, dsl.get_num(b).unwrap());
            dsl.set_program_output("position", b).unwrap();
        }

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_select_among_eight() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let hashes = (0..8)
            .map(|_| {
                let mut bytes = [0u8; 32];
                prng.fill_bytes(&mut bytes);
                Sha256Hash::from(bytes.as_slice())
            })
            .collect_vec();

        let mut dsl = DSL::new();
        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let mut hashes_vars = vec![];
        for hash in hashes.iter() {
            hashes_vars.push(
                dsl.alloc_input("hash", Element::Str(hash.as_ref().to_vec()))
                    .unwrap(),
            );
        }

        for i in 1..=8 {
            let selected = dsl
                .execute_with_options(
                    "select_among_eight",
                    &hashes_vars,
                    &Options::new().with_u32("pick", i as u32),
                )
                .unwrap()[0];
            dsl.set_program_output("hash", selected).unwrap();
        }

        test_program(
            dsl,
            script! {
                for hash in hashes.iter() {
                    { hash.as_ref().to_vec() }
                }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_skip_one_and_extract_5_bits() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let mut dsl = DSL::new();
            load_data_types(&mut dsl).unwrap();
            load_functions(&mut dsl).unwrap();

            let test_position = prng.gen_range(0..=1023);

            let test_position_var = dsl
                .alloc_input("position", Element::Num(test_position))
                .unwrap();

            let bits_vars = dsl
                .execute("skip_one_and_extract_5_bits", &[test_position_var])
                .unwrap();

            let expected = [
                (test_position >> 1) & 1,
                (test_position >> 2) & 1,
                (test_position >> 3) & 1,
                (test_position >> 4) & 1,
                (test_position >> 5) & 1,
            ];

            for (&expected_bit, &bit_var) in expected.iter().zip(bits_vars.iter()) {
                assert_eq!(dsl.get_num(bit_var).unwrap(), expected_bit);
                dsl.set_program_output("position", bit_var).unwrap();
            }

            test_program(
                dsl,
                script! {
                    { expected.to_vec() }
                },
            )
            .unwrap()
        }
    }
}
