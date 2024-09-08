use crate::algorithms::utils::OP_HINT;
use anyhow::Result;
use bitcoin_circle_stark::treepp::*;
use bitcoin_script::script;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{FunctionMetadata, FunctionOutput};
use itertools::Itertools;

fn check_0_or_1() -> Script {
    script! {
        OP_DUP 0 OP_GREATERTHANOREQUAL OP_VERIFY
        OP_DUP 1 OP_LESSTHANOREQUAL OP_VERIFY
    }
}

fn check_0_to_3() -> Script {
    script! {
        OP_DUP 0 OP_GREATERTHANOREQUAL OP_VERIFY
        OP_DUP 3 OP_LESSTHANOREQUAL OP_VERIFY
    }
}

fn decompose_positions_to_5(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let mut cur = dsl.get_num(inputs[0])? as usize;

    let mut hints = vec![];

    // remove the lower two bits
    hints.push(cur & 3);
    cur >>= 2;

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
        OP_HINT check_0_to_3
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
        OP_DUP OP_ADD OP_DUP OP_ADD OP_ADD
        OP_EQUALVERIFY

        for _ in 0..5 {
            OP_FROMALTSTACK
        }
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
    )
}

#[cfg(test)]
mod test {
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script::script;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

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
}
