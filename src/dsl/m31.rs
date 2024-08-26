use crate::treepp::*;
use crate::utils::{check_limb_format, OP_256MUL, OP_HINT};
use anyhow::{Error, Result};
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::FunctionOutput;

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

pub fn m31_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
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

pub fn m31_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        4 OP_ROLL OP_EQUALVERIFY
        3 OP_ROLL OP_EQUALVERIFY
        OP_ROT OP_EQUALVERIFY
        OP_EQUALVERIFY
    })
}

#[cfg(test)]
mod test {
    use crate::dsl::{load_data_types, load_functions};
    use crate::utils::convert_m31_to_limbs;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_m31_to_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_val = prng.gen_range(0..((1i64 << 31) - 1)) as i32;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl.alloc_constant("m31", Element::Num(a_val)).unwrap();
        let res = dsl.execute("m31_to_limbs", &[a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_m31_to_limbs(a_val as u32)
        );

        let expected = dsl
            .alloc_constant(
                "m31_limbs",
                Element::ManyNum(convert_m31_to_limbs(a_val as u32).to_vec()),
            )
            .unwrap();
        let _ = dsl.execute("m31_equalverify", &[res[0], expected]).unwrap();

        test_program(dsl).unwrap();
    }
}
