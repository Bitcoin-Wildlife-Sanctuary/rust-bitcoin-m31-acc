use crate::dsl::cm31::cm31_to_limbs_gadget;
use crate::qm31::{QM31Mult, QM31MultGadget};
use crate::utils::convert_qm31_from_limbs;
use anyhow::{Error, Result};
use bitcoin_circle_stark::treepp::*;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{FunctionMetadata, FunctionOutput};
use itertools::Itertools;
use rust_bitcoin_m31::qm31_equalverify as raw_qm31_equalverify;
use stwo_prover::core::fields::qm31::QM31;

pub fn qm31_to_limbs(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let num = dsl.get_many_num(inputs[0])?;

    let limbs = num
        .iter()
        .rev()
        .map(|x| {
            [
                x & 0xff,
                (x >> 8) & 0xff,
                (x >> 16) & 0xff,
                (x >> 24) & 0xff,
            ]
        })
        .flatten()
        .collect_vec();

    let new_entry = MemoryEntry::new("qm31_limbs", Element::ManyNum(limbs));

    Ok(FunctionOutput {
        new_elements: vec![new_entry.clone()],
        new_hints: vec![new_entry],
    })
}

pub fn qm31_to_limbs_gadget(_: &[usize]) -> Result<Script> {
    // Hint: 16 limbs
    // Input: qm31
    // Output: 16 limbs
    Ok(script! {
        { cm31_to_limbs_gadget(&[])? }
        9 OP_ROLL
        9 OP_ROLL
        { cm31_to_limbs_gadget(&[])? }
    })
}

pub fn qm31_limbs_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
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

pub fn qm31_limbs_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        for i in (3..=16).rev() {
            { i } OP_ROLL OP_EQUALVERIFY
        }
        OP_ROT OP_EQUALVERIFY
        OP_EQUALVERIFY
    })
}

pub fn qm31_limbs_mul(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[1])?.to_vec();
    let b = dsl.get_many_num(inputs[2])?.to_vec();

    let hint = QM31Mult::compute_hint_from_limbs(&a[0..8], &a[8..16], &b[0..8], &b[8..16])?;

    let a_qm31 = convert_qm31_from_limbs(&a);
    let b_qm31 = convert_qm31_from_limbs(&b);

    let expected = a_qm31 * b_qm31;

    let new_hints = [hint.h1, hint.h2, hint.h3]
        .iter()
        .flat_map(|x| {
            vec![
                MemoryEntry::new("qm31", Element::Num(x.q1)),
                MemoryEntry::new("qm31", Element::Num(x.q2)),
                MemoryEntry::new("qm31", Element::Num(x.q3)),
            ]
        })
        .collect_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(expected)),
        )],
        new_hints,
    })
}

pub fn qm31_limbs_mul_gadget(r: &[usize]) -> Result<Script> {
    Ok(QM31MultGadget::mult(r[0] - 512))
}

pub fn qm31_limbs_get_first(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31_limbs",
            Element::ManyNum(a[0..8].to_vec()),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_limbs_get_first_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..8 {
            { r[0] } OP_PICK
        }
    })
}

pub fn qm31_limbs_get_second(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31_limbs",
            Element::ManyNum(a[8..16].to_vec()),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_limbs_get_second_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..8 {
            { r[0] - 8 } OP_PICK
        }
    })
}

pub fn qm31_equalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
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

pub fn qm31_equalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        raw_qm31_equalverify
    })
}

pub fn qm31_first(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("cm31", Element::ManyNum(vec![a[2], a[3]]))],
        new_hints: vec![],
    })
}

pub fn qm31_first_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..2 {
            { r[0] - 2 } OP_PICK
        }
    })
}

pub fn qm31_second(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("cm31", Element::ManyNum(vec![a[0], a[1]]))],
        new_hints: vec![],
    })
}

pub fn qm31_second_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..2 {
            { r[0] } OP_PICK
        }
    })
}

pub(crate) fn reformat_qm31_to_dsl_element(v: QM31) -> Vec<i32> {
    vec![
        v.1 .1 .0 as i32,
        v.1 .0 .0 as i32,
        v.0 .1 .0 as i32,
        v.0 .0 .0 as i32,
    ]
}

pub(crate) fn load_functions(dsl: &mut DSL) {
    dsl.add_function(
        "qm31_to_limbs",
        FunctionMetadata {
            trace_generator: qm31_to_limbs,
            script_generator: qm31_to_limbs_gadget,
            input: vec!["qm31"],
            output: vec!["qm31_limbs"],
        },
    );
    dsl.add_function(
        "qm31_limbs_equalverify",
        FunctionMetadata {
            trace_generator: qm31_limbs_equalverify,
            script_generator: qm31_limbs_equalverify_gadget,
            input: vec!["qm31_limbs", "qm31_limbs"],
            output: vec![],
        },
    );
    dsl.add_function(
        "qm31_limbs_mul",
        FunctionMetadata {
            trace_generator: qm31_limbs_mul,
            script_generator: qm31_limbs_mul_gadget,
            input: vec!["&table", "qm31_limbs", "qm31_limbs"],
            output: vec!["qm31"],
        },
    );
    dsl.add_function(
        "qm31_limbs_first",
        FunctionMetadata {
            trace_generator: qm31_limbs_get_first,
            script_generator: qm31_limbs_get_first_gadget,
            input: vec!["&qm31_limbs"],
            output: vec!["cm31_limbs"],
        },
    );
    dsl.add_function(
        "qm31_limbs_second",
        FunctionMetadata {
            trace_generator: qm31_limbs_get_second,
            script_generator: qm31_limbs_get_second_gadget,
            input: vec!["&qm31_limbs"],
            output: vec!["cm31_limbs"],
        },
    );
    dsl.add_function(
        "qm31_equalverify",
        FunctionMetadata {
            trace_generator: qm31_equalverify,
            script_generator: qm31_equalverify_gadget,
            input: vec!["qm31", "qm31"],
            output: vec![],
        },
    );
    dsl.add_function(
        "qm31_first",
        FunctionMetadata {
            trace_generator: qm31_first,
            script_generator: qm31_first_gadget,
            input: vec!["&qm31"],
            output: vec!["cm31"],
        },
    );
    dsl.add_function(
        "qm31_second",
        FunctionMetadata {
            trace_generator: qm31_second,
            script_generator: qm31_second_gadget,
            input: vec!["&qm31"],
            output: vec!["cm31"],
        },
    )
}

#[cfg(test)]
mod test {
    use crate::dsl::cm31::reformat_cm31_to_dsl_element;
    use crate::dsl::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::{load_data_types, load_functions};
    use crate::utils::{convert_cm31_to_limbs, convert_qm31_to_limbs};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use itertools::Itertools;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::fields::qm31::QM31;

    #[test]
    fn test_qm31_to_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_constant(
                "qm31",
                Element::ManyNum(vec![a[3] as i32, a[2] as i32, a[1] as i32, a[0] as i32]),
            )
            .unwrap();
        let res = dsl.execute("qm31_to_limbs", &[a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_qm31_to_limbs(a_qm31)
        );

        dsl.set_program_output("qm31_limbs", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { convert_qm31_to_limbs(a_qm31).to_vec() }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_qm31_limbs_mul() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();
        let b = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);
        let b_qm31 = QM31::from_u32_unchecked(b[0], b[1], b[2], b[3]);

        let expected = a_qm31 * b_qm31;

        let a_limbs = convert_qm31_to_limbs(a_qm31);
        let b_limbs = convert_qm31_to_limbs(b_qm31);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input("qm31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();
        let b = dsl
            .alloc_input("qm31_limbs", Element::ManyNum(b_limbs.to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];
        let res = dsl.execute("qm31_limbs_mul", &[table, a, b]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            &[
                expected.1 .1 .0 as i32,
                expected.1 .0 .0 as i32,
                expected.0 .1 .0 as i32,
                expected.0 .0 .0 as i32
            ]
        );

        dsl.set_program_output("qm31", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_qm31_limbs_get_first_and_second() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);

        let a_first_cm31 = a_qm31.0;
        let a_second_cm31 = a_qm31.1;

        let a_limbs = convert_qm31_to_limbs(a_qm31);
        let a_first_limbs = convert_cm31_to_limbs(a_first_cm31);
        let a_second_limbs = convert_cm31_to_limbs(a_second_cm31);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input("qm31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();

        let a_first = dsl.execute("qm31_limbs_first", &[a]).unwrap()[0];
        let a_second = dsl.execute("qm31_limbs_second", &[a]).unwrap()[0];

        assert_eq!(dsl.get_many_num(a_first).unwrap(), a_first_limbs);
        assert_eq!(dsl.get_many_num(a_second).unwrap(), a_second_limbs);

        dsl.set_program_output("cm31_limbs", a_first).unwrap();
        dsl.set_program_output("cm31_limbs", a_second).unwrap();

        test_program(
            dsl,
            script! {
                { a_first_limbs.to_vec() }
                { a_second_limbs.to_vec() }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_get_first_and_second() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);

        let a_first_cm31 = a_qm31.0;
        let a_second_cm31 = a_qm31.1;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(a_qm31)),
            )
            .unwrap();

        let a_first = dsl.execute("qm31_first", &[a]).unwrap()[0];
        let a_second = dsl.execute("qm31_second", &[a]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(a_first).unwrap(),
            reformat_cm31_to_dsl_element(a_first_cm31)
        );
        assert_eq!(
            dsl.get_many_num(a_second).unwrap(),
            reformat_cm31_to_dsl_element(a_second_cm31)
        );

        dsl.set_program_output("cm31", a_first).unwrap();
        dsl.set_program_output("cm31", a_second).unwrap();

        test_program(
            dsl,
            script! {
                { a_first_cm31 }
                { a_second_cm31 }
            },
        )
        .unwrap()
    }
}
