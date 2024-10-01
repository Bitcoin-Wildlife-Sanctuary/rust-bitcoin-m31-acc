use crate::algorithms::m31::{M31Mult, M31MultGadget};
use crate::algorithms::utils::{
    check_limb_format, convert_m31_from_limbs, convert_m31_to_limbs, OP_256MUL, OP_HINT,
};
use crate::dsl::framework::dsl::{Element, MemoryEntry, DSL};
use crate::dsl::framework::functions::{FunctionMetadata, FunctionOutput};
use anyhow::{Error, Result};
use bitcoin_circle_stark::treepp::*;
use rust_bitcoin_m31::push_m31_one;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;

pub fn m31_to_limbs(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let num = dsl.get_num(inputs[0])?;

    let limbs = vec![
        num & 0xff,
        (num >> 8) & 0xff,
        (num >> 16) & 0xff,
        (num >> 24) & 0xff,
    ];

    let new_entry = MemoryEntry::new("m31_limbs", Element::ManyNum(limbs));

    Ok(FunctionOutput {
        new_elements: vec![new_entry.clone()],
        new_hints: vec![new_entry],
    })
}

pub fn m31_to_limbs_gadget(_: &[usize]) -> Result<Script> {
    // Hint: four limbs
    // Input: m31
    // Output: four limbs
    Ok(script! {
        OP_HINT check_limb_format OP_DUP OP_TOALTSTACK
        OP_HINT check_limb_format OP_DUP OP_TOALTSTACK
        OP_HINT check_limb_format OP_DUP OP_TOALTSTACK
        OP_HINT check_limb_format OP_DUP OP_TOALTSTACK

        OP_256MUL
        OP_ADD
        OP_256MUL
        OP_ADD
        OP_256MUL
        OP_ADD

        OP_EQUALVERIFY

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP
    })
}

pub fn m31_limbs_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();
    let b = dsl.get_many_num(inputs[1])?.to_vec();

    if a != b {
        Err(Error::msg("Equalverify fails"))
    } else {
        Ok(FunctionOutput {
            new_elements: vec![],
            new_hints: vec![],
        })
    }
}

pub fn m31_limbs_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        4 OP_ROLL OP_EQUALVERIFY
        3 OP_ROLL OP_EQUALVERIFY
        OP_ROT OP_EQUALVERIFY
        OP_EQUALVERIFY
    })
}

pub fn m31_limbs_mul(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[1])?.to_vec();
    let b = dsl.get_many_num(inputs[2])?.to_vec();

    let c_limbs = M31Mult::compute_c_limbs_from_limbs(&a, &b)?;
    let q = M31Mult::compute_q(&c_limbs)?;

    let a_val = convert_m31_from_limbs(&a);
    let b_val = convert_m31_from_limbs(&b);

    let expected = (a_val as u64) * (b_val as u64) % ((1 << 31) - 1);

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("m31", Element::Num(expected as i32))],
        new_hints: vec![MemoryEntry::new("m31", Element::Num(q))],
    })
}

pub fn m31_limbs_mul_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        { M31MultGadget::compute_c_limbs(r[0] - 512) }
        { M31MultGadget::reduce() }
    })
}

pub fn m31_limbs_inverse(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[1])?;
    let a_val = convert_m31_from_limbs(a);

    let a_m31 = M31::from_u32_unchecked(a_val);

    let inv = a_m31.inverse().0;

    let c_limbs = M31Mult::compute_c_limbs(a_val, inv)?;
    let q = M31Mult::compute_q(&c_limbs)?;

    let output_entry = MemoryEntry::new(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(inv).to_vec()),
    );

    Ok(FunctionOutput {
        new_elements: vec![output_entry.clone()],
        new_hints: vec![output_entry, MemoryEntry::new("m31", Element::Num(q))],
    })
}

pub fn m31_limbs_inverse_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..4 {
            OP_HINT check_limb_format OP_DUP OP_TOALTSTACK
        }
        { M31MultGadget::compute_c_limbs(r[0] - 512) }
        { M31MultGadget::reduce() }
        push_m31_one OP_EQUALVERIFY

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP
    })
}

pub fn m31_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_num(inputs[0])?;
    let b = dsl.get_num(inputs[1])?;

    if a != b {
        Err(Error::msg("Equalverify fails"))
    } else {
        Ok(FunctionOutput {
            new_elements: vec![],
            new_hints: vec![],
        })
    }
}

pub fn m31_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_EQUALVERIFY
    })
}

pub(crate) fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "m31_to_limbs",
        FunctionMetadata {
            trace_generator: m31_to_limbs,
            script_generator: m31_to_limbs_gadget,
            input: vec!["m31"],
            output: vec!["m31_limbs"],
        },
    )?;
    dsl.add_function(
        "m31_limbs_equalverify",
        FunctionMetadata {
            trace_generator: m31_limbs_equalverify,
            script_generator: m31_limbs_equalverify_gadget,
            input: vec!["m31_limbs", "m31_limbs"],
            output: vec![],
        },
    )?;
    dsl.add_function(
        "m31_limbs_mul",
        FunctionMetadata {
            trace_generator: m31_limbs_mul,
            script_generator: m31_limbs_mul_gadget,
            input: vec!["&table", "m31_limbs", "m31_limbs"],
            output: vec!["m31"],
        },
    )?;
    dsl.add_function(
        "m31_limbs_inverse",
        FunctionMetadata {
            trace_generator: m31_limbs_inverse,
            script_generator: m31_limbs_inverse_gadget,
            input: vec!["&table", "m31_limbs"],
            output: vec!["m31_limbs"],
        },
    )?;
    dsl.add_function(
        "m31_equalverify",
        FunctionMetadata {
            trace_generator: m31_equalverify,
            script_generator: m31_equalverify_gadget,
            input: vec!["m31", "m31"],
            output: vec![],
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::algorithms::utils::convert_m31_to_limbs;
    use crate::dsl::framework::dsl::{Element, DSL};
    use crate::dsl::framework::test_program;
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script::script;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::fields::FieldExpOps;

    #[test]
    fn test_m31_to_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_val = prng.gen_range(0..((1i64 << 31) - 1)) as i32;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a = dsl.alloc_constant("m31", Element::Num(a_val)).unwrap();
        let res = dsl.execute("m31_to_limbs", &[a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_m31_to_limbs(a_val as u32)
        );

        dsl.set_program_output("m31_limbs", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { convert_m31_to_limbs(a_val as u32).to_vec() }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_m31_limbs_mul() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = prng.gen_range(0u32..((1 << 31) - 1));
        let b = prng.gen_range(0u32..((1 << 31) - 1));
        let r = (a as u64) * (b as u64) % ((1 << 31) - 1);

        let a_limbs = convert_m31_to_limbs(a);
        let b_limbs = convert_m31_to_limbs(b);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a = dsl
            .alloc_input("m31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();
        let b = dsl
            .alloc_input("m31_limbs", Element::ManyNum(b_limbs.to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];
        let res = dsl.execute("m31_limbs_mul", &[table, a, b]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(dsl.get_num(res[0]).unwrap(), r as i32);

        dsl.set_program_output("m31", res[0]).unwrap();

        test_program(
            dsl,
            script! {{ r as u32 }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_m31_limbs_inverse() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = prng.gen_range(0u32..((1 << 31) - 1));
        let a_limbs = convert_m31_to_limbs(a);
        let inv = M31::from_u32_unchecked(a).inverse();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a = dsl
            .alloc_input("m31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];
        let res = dsl.execute("m31_limbs_inverse", &[table, a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_m31_to_limbs(inv.0)
        );

        dsl.set_program_output("m31_limbs", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { convert_m31_to_limbs(inv.0).to_vec() }
            },
        )
        .unwrap();
    }
}
