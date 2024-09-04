use crate::algorithms::cm31::{CM31Mult, CM31MultGadget};
use crate::algorithms::decompose::DecomposeGadget;
use crate::dsl::building_blocks::m31::m31_to_limbs_gadget;
use crate::algorithms::utils::{check_limb_format, convert_cm31_from_limbs, convert_cm31_to_limbs, OP_HINT};
use anyhow::Error;
use anyhow::Result;
use bitcoin_circle_stark::treepp::*;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{FunctionMetadata, FunctionOutput};
use rust_bitcoin_m31::{
    cm31_add as raw_cm31_add, cm31_equalverify as raw_cm31_equalverify, cm31_sub as raw_cm31_sub,
    m31_add, push_cm31_one,
};
use stwo_prover::core::fields::cm31::CM31;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;

fn cm31_to_limbs(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let num = dsl.get_many_num(inputs[0])?;

    let limbs = vec![
        num[1] & 0xff,
        (num[1] >> 8) & 0xff,
        (num[1] >> 16) & 0xff,
        (num[1] >> 24) & 0xff,
        num[0] & 0xff,
        (num[0] >> 8) & 0xff,
        (num[0] >> 16) & 0xff,
        (num[0] >> 24) & 0xff,
    ];

    let new_entry = MemoryEntry::new("cm31_limbs", Element::ManyNum(limbs));

    Ok(FunctionOutput {
        new_elements: vec![new_entry.clone()],
        new_hints: vec![new_entry],
    })
}

pub(crate) fn cm31_to_limbs_gadget(_: &[usize]) -> Result<Script> {
    // Hint: eight limbs
    // Input: cm31
    // Output: eight limbs
    Ok(script! {
        { m31_to_limbs_gadget(&[])? }
        4 OP_ROLL
        { m31_to_limbs_gadget(&[])? }
    })
}

fn cm31_limbs_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
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

fn cm31_limbs_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        for i in (3..=8).rev() {
            { i } OP_ROLL OP_EQUALVERIFY
        }
        OP_ROT OP_EQUALVERIFY
        OP_EQUALVERIFY
    })
}

fn cm31_limbs_mul(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[1])?.to_vec();
    let b = dsl.get_many_num(inputs[2])?.to_vec();

    let hint = CM31Mult::compute_hint_from_limbs(&a[0..4], &a[4..8], &b[0..4], &b[4..8])?;

    let a_cm31 = convert_cm31_from_limbs(&a);
    let b_cm31 = convert_cm31_from_limbs(&b);

    let expected = a_cm31 * b_cm31;

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31",
            Element::ManyNum(reformat_cm31_to_dsl_element(expected)),
        )],
        new_hints: vec![
            MemoryEntry::new("m31", Element::Num(hint.q1)),
            MemoryEntry::new("m31", Element::Num(hint.q2)),
            MemoryEntry::new("m31", Element::Num(hint.q3)),
        ],
    })
}

fn cm31_limbs_mul_gadget(r: &[usize]) -> Result<Script> {
    Ok(CM31MultGadget::mult(r[0] - 512))
}

fn cm31_limbs_inverse(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[1])?;
    let a_val = convert_cm31_from_limbs(a);

    let inv = a_val.inverse();
    let inv_limbs = convert_cm31_to_limbs(inv);

    let hint =
        CM31Mult::compute_hint_from_limbs(&a[0..4], &a[4..8], &inv_limbs[0..4], &inv_limbs[4..8])?;

    let output_entry = MemoryEntry::new("cm31_limbs", Element::ManyNum(inv_limbs.to_vec()));

    Ok(FunctionOutput {
        new_elements: vec![output_entry.clone()],
        new_hints: vec![
            output_entry,
            MemoryEntry::new("m31", Element::Num(hint.q1)),
            MemoryEntry::new("m31", Element::Num(hint.q2)),
            MemoryEntry::new("m31", Element::Num(hint.q3)),
        ],
    })
}

fn cm31_limbs_inverse_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..8 {
            OP_HINT check_limb_format OP_DUP OP_TOALTSTACK
        }
        { CM31MultGadget::mult(r[0] - 512) }
        push_cm31_one raw_cm31_equalverify

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP

        for _ in 0..4 {
            7 OP_ROLL
        }
    })
}

fn cm31_recompose(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = convert_cm31_from_limbs(dsl.get_many_num(inputs[0])?);

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31",
            Element::ManyNum(reformat_cm31_to_dsl_element(a)),
        )],
        new_hints: vec![],
    })
}

fn cm31_recompose_gadget(_: &[usize]) -> Result<Script> {
    Ok(DecomposeGadget::recompose_cm31())
}

fn cm31_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
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

fn cm31_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_cm31_equalverify())
}

fn cm31_add(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();
    let b = dsl.get_many_num(inputs[1])?.to_vec();

    let a_cm31 = CM31::from_u32_unchecked(a[1] as u32, a[0] as u32);
    let b_cm31 = CM31::from_u32_unchecked(b[1] as u32, b[0] as u32);

    let res = a_cm31 + b_cm31;

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31",
            Element::ManyNum(reformat_cm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn cm31_add_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_cm31_add())
}

fn cm31_add_m31(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();
    let b = dsl.get_num(inputs[1])?;

    let a_cm31 = CM31::from_u32_unchecked(a[1] as u32, a[0] as u32);
    let b_m31 = M31::from_u32_unchecked(b as u32);

    let res = a_cm31 + b_m31;

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31",
            Element::ManyNum(reformat_cm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn cm31_add_m31_gadget(_: &[usize]) -> Result<Script> {
    Ok(m31_add())
}

fn cm31_sub(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();
    let b = dsl.get_many_num(inputs[1])?.to_vec();

    let a_cm31 = CM31::from_u32_unchecked(a[1] as u32, a[0] as u32);
    let b_cm31 = CM31::from_u32_unchecked(b[1] as u32, b[0] as u32);

    let res = a_cm31 - b_cm31;

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31",
            Element::ManyNum(reformat_cm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn cm31_sub_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_cm31_sub())
}

fn cm31_limbs_real(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "m31_limbs",
            Element::ManyNum(a[0..4].to_vec()),
        )],
        new_hints: vec![],
    })
}

fn cm31_limbs_real_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..4 {
            { r[0] } OP_PICK
        }
    })
}

fn cm31_limbs_imag(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "m31_limbs",
            Element::ManyNum(a[4..8].to_vec()),
        )],
        new_hints: vec![],
    })
}

fn cm31_limbs_imag_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..4 {
            { r[0] - 4 } OP_PICK
        }
    })
}

fn cm31_real(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("m31", Element::Num(a[1]))],
        new_hints: vec![],
    })
}

fn cm31_real_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        { r[0] - 1 } OP_PICK
    })
}

fn cm31_imag(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("m31", Element::Num(a[0]))],
        new_hints: vec![],
    })
}

fn cm31_imag_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        { r[0] } OP_PICK
    })
}

fn cm31_from_real_and_imag(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_num(inputs[0])?;
    let b = dsl.get_num(inputs[1])?;

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("cm31", Element::ManyNum(vec![b, a]))],
        new_hints: vec![],
    })
}

fn cm31_from_real_and_imag_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_SWAP
    })
}

pub fn cm31_mul_m31_limbs(
    dsl: &mut DSL,
    table: usize,
    cm31: usize,
    m31_limbs: usize,
) -> Result<usize> {
    let real = dsl.execute("cm31_real", &[cm31])?[0];
    let imag = dsl.execute("cm31_imag", &[cm31])?[0];

    let real_limbs = dsl.execute("m31_to_limbs", &[real])?[0];
    let real_res = dsl.execute("m31_limbs_mul", &[table, real_limbs, m31_limbs])?[0];

    let imag_limbs = dsl.execute("m31_to_limbs", &[imag])?[0];
    let imag_res = dsl.execute("m31_limbs_mul", &[table, imag_limbs, m31_limbs])?[0];

    Ok(dsl.execute("cm31_from_real_and_imag", &[real_res, imag_res])?[0])
}

pub(crate) fn reformat_cm31_to_dsl_element(v: CM31) -> Vec<i32> {
    vec![v.1 .0 as i32, v.0 .0 as i32]
}

pub(crate) fn reformat_cm31_from_dsl_element(v: &[i32]) -> CM31 {
    CM31::from_u32_unchecked(v[1] as u32, v[0] as u32)
}

pub(crate) fn load_functions(dsl: &mut DSL) {
    dsl.add_function(
        "cm31_to_limbs",
        FunctionMetadata {
            trace_generator: cm31_to_limbs,
            script_generator: cm31_to_limbs_gadget,
            input: vec!["cm31"],
            output: vec!["cm31_limbs"],
        },
    );
    dsl.add_function(
        "cm31_limbs_equalverify",
        FunctionMetadata {
            trace_generator: cm31_limbs_equalverify,
            script_generator: cm31_limbs_equalverify_gadget,
            input: vec!["cm31_limbs", "cm31_limbs"],
            output: vec![],
        },
    );
    dsl.add_function(
        "cm31_limbs_mul",
        FunctionMetadata {
            trace_generator: cm31_limbs_mul,
            script_generator: cm31_limbs_mul_gadget,
            input: vec!["&table", "cm31_limbs", "cm31_limbs"],
            output: vec!["cm31"],
        },
    );
    dsl.add_function(
        "cm31_limbs_inverse",
        FunctionMetadata {
            trace_generator: cm31_limbs_inverse,
            script_generator: cm31_limbs_inverse_gadget,
            input: vec!["&table", "cm31_limbs"],
            output: vec!["cm31_limbs"],
        },
    );
    dsl.add_function(
        "cm31_recompose",
        FunctionMetadata {
            trace_generator: cm31_recompose,
            script_generator: cm31_recompose_gadget,
            input: vec!["cm31_limbs"],
            output: vec!["cm31"],
        },
    );
    dsl.add_function(
        "cm31_equalverify",
        FunctionMetadata {
            trace_generator: cm31_equalverify,
            script_generator: cm31_equalverify_gadget,
            input: vec!["cm31", "cm31"],
            output: vec![],
        },
    );
    dsl.add_function(
        "cm31_add",
        FunctionMetadata {
            trace_generator: cm31_add,
            script_generator: cm31_add_gadget,
            input: vec!["cm31", "cm31"],
            output: vec!["cm31"],
        },
    );
    dsl.add_function(
        "cm31_add_m31",
        FunctionMetadata {
            trace_generator: cm31_add_m31,
            script_generator: cm31_add_m31_gadget,
            input: vec!["cm31", "m31"],
            output: vec!["cm31"],
        },
    );
    dsl.add_function(
        "cm31_sub",
        FunctionMetadata {
            trace_generator: cm31_sub,
            script_generator: cm31_sub_gadget,
            input: vec!["cm31", "cm31"],
            output: vec!["cm31"],
        },
    );
    dsl.add_function(
        "cm31_limbs_real",
        FunctionMetadata {
            trace_generator: cm31_limbs_real,
            script_generator: cm31_limbs_real_gadget,
            input: vec!["&cm31_limbs"],
            output: vec!["m31_limbs"],
        },
    );
    dsl.add_function(
        "cm31_limbs_imag",
        FunctionMetadata {
            trace_generator: cm31_limbs_imag,
            script_generator: cm31_limbs_imag_gadget,
            input: vec!["&cm31_limbs"],
            output: vec!["m31_limbs"],
        },
    );
    dsl.add_function(
        "cm31_real",
        FunctionMetadata {
            trace_generator: cm31_real,
            script_generator: cm31_real_gadget,
            input: vec!["&cm31"],
            output: vec!["m31"],
        },
    );
    dsl.add_function(
        "cm31_imag",
        FunctionMetadata {
            trace_generator: cm31_imag,
            script_generator: cm31_imag_gadget,
            input: vec!["&cm31"],
            output: vec!["m31"],
        },
    );
    dsl.add_function(
        "cm31_from_real_and_imag",
        FunctionMetadata {
            trace_generator: cm31_from_real_and_imag,
            script_generator: cm31_from_real_and_imag_gadget,
            input: vec!["m31", "m31"],
            output: vec!["cm31"],
        },
    )
}

#[cfg(test)]
mod test {
    use crate::dsl::building_blocks::cm31::{cm31_mul_m31_limbs, reformat_cm31_to_dsl_element};
    use crate::dsl::{load_data_types, load_functions};
    use crate::algorithms::utils::{convert_cm31_to_limbs, convert_m31_to_limbs};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{Rng, RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::fields::cm31::CM31;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::fields::FieldExpOps;

    #[test]
    fn test_cm31_to_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_real = prng.gen_range(0u32..((1 << 31) - 1));
        let a_imag = prng.gen_range(0u32..((1 << 31) - 1));

        let a_cm31 = CM31::from_u32_unchecked(a_real, a_imag);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_constant("cm31", Element::ManyNum(vec![a_imag as i32, a_real as i32]))
            .unwrap();
        let res = dsl.execute("cm31_to_limbs", &[a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_cm31_to_limbs(a_cm31)
        );

        dsl.set_program_output("cm31_limbs", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { convert_cm31_to_limbs(a_cm31).to_vec() }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_cm31_limbs_mul() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_real = prng.gen_range(0u32..((1 << 31) - 1));
        let a_imag = prng.gen_range(0u32..((1 << 31) - 1));
        let b_real = prng.gen_range(0u32..((1 << 31) - 1));
        let b_imag = prng.gen_range(0u32..((1 << 31) - 1));

        let a_cm31 = CM31::from_u32_unchecked(a_real, a_imag);
        let b_cm31 = CM31::from_u32_unchecked(b_real, b_imag);

        let expected = a_cm31 * b_cm31;

        let a_limbs = convert_cm31_to_limbs(a_cm31);
        let b_limbs = convert_cm31_to_limbs(b_cm31);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input("cm31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();
        let b = dsl
            .alloc_input("cm31_limbs", Element::ManyNum(b_limbs.to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];
        let res = dsl.execute("cm31_limbs_mul", &[table, a, b]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            &[expected.1 .0 as i32, expected.0 .0 as i32]
        );

        dsl.set_program_output("cm31", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_cm31_limbs_inverse() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_real = prng.gen_range(0u32..((1 << 31) - 1));
        let a_imag = prng.gen_range(0u32..((1 << 31) - 1));

        let a_cm31 = CM31::from_u32_unchecked(a_real, a_imag);
        let a_limbs = convert_cm31_to_limbs(a_cm31);

        let inv = a_cm31.inverse();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input("cm31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = dsl.execute("cm31_limbs_inverse", &[table, a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_cm31_to_limbs(inv)
        );

        dsl.set_program_output("cm31_limbs", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { convert_cm31_to_limbs(inv).to_vec() }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_cm31_limbs_real_and_imag() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_real = prng.gen_range(0u32..((1 << 31) - 1));
        let a_imag = prng.gen_range(0u32..((1 << 31) - 1));

        let a_cm31 = CM31::from_u32_unchecked(a_real, a_imag);

        let a_real_m31 = a_cm31.0;
        let a_imag_m31 = a_cm31.1;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input(
                "cm31_limbs",
                Element::ManyNum(convert_cm31_to_limbs(a_cm31).to_vec()),
            )
            .unwrap();

        let a_real = dsl.execute("cm31_limbs_real", &[a]).unwrap()[0];
        let a_imag = dsl.execute("cm31_limbs_imag", &[a]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(a_real).unwrap(),
            convert_m31_to_limbs(a_real_m31.0)
        );
        assert_eq!(
            dsl.get_many_num(a_imag).unwrap(),
            convert_m31_to_limbs(a_imag_m31.0)
        );

        dsl.set_program_output("m31_limbs", a_real).unwrap();
        dsl.set_program_output("m31_limbs", a_imag).unwrap();

        test_program(
            dsl,
            script! {
                { convert_m31_to_limbs(a_real_m31.0).to_vec() }
                { convert_m31_to_limbs(a_imag_m31.0).to_vec() }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_cm31_real_and_imag() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_real = prng.gen_range(0u32..((1 << 31) - 1));
        let a_imag = prng.gen_range(0u32..((1 << 31) - 1));

        let a_cm31 = CM31::from_u32_unchecked(a_real, a_imag);

        let a_real_m31 = a_cm31.0;
        let a_imag_m31 = a_cm31.1;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input(
                "cm31",
                Element::ManyNum(reformat_cm31_to_dsl_element(a_cm31).to_vec()),
            )
            .unwrap();

        let a_real = dsl.execute("cm31_real", &[a]).unwrap()[0];
        let a_imag = dsl.execute("cm31_imag", &[a]).unwrap()[0];

        assert_eq!(dsl.get_num(a_real).unwrap(), a_real_m31.0 as i32);
        assert_eq!(dsl.get_num(a_imag).unwrap(), a_imag_m31.0 as i32);

        dsl.set_program_output("m31", a_real).unwrap();
        dsl.set_program_output("m31", a_imag).unwrap();

        test_program(
            dsl,
            script! {
                { a_real_m31 }
                { a_imag_m31 }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_cm31_from_real_and_imag() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a_real = prng.gen_range(0u32..((1 << 31) - 1));
        let a_imag = prng.gen_range(0u32..((1 << 31) - 1));

        let a_cm31 = CM31::from_u32_unchecked(a_real, a_imag);

        let a_real_m31 = a_cm31.0;
        let a_imag_m31 = a_cm31.1;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input("m31", Element::Num(a_real_m31.0 as i32))
            .unwrap();

        let b = dsl
            .alloc_input("m31", Element::Num(a_imag_m31.0 as i32))
            .unwrap();

        let cm31 = dsl.execute("cm31_from_real_and_imag", &[a, b]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(cm31).unwrap(),
            reformat_cm31_to_dsl_element(a_cm31)
        );

        dsl.set_program_output("cm31", cm31).unwrap();

        test_program(
            dsl,
            script! {
                { a_cm31 }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_cm31_mul_m31_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let real = prng.gen_range(0u32..((1 << 31) - 1));
        let imag = prng.gen_range(0u32..((1 << 31) - 1));

        let cm31 = CM31::from_u32_unchecked(real, imag);
        let m31 = M31::reduce(prng.next_u64());

        let expected = cm31 * m31;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let cm31_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(cm31)))
            .unwrap();
        let m31_limbs_var = dsl
            .alloc_input(
                "m31_limbs",
                Element::ManyNum(convert_m31_to_limbs(m31.0).to_vec()),
            )
            .unwrap();
        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = cm31_mul_m31_limbs(&mut dsl, table, cm31_var, m31_limbs_var).unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_cm31_to_dsl_element(expected)
        );

        dsl.set_program_output("cm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap()
    }
}
