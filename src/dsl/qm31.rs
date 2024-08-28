use crate::dsl::cm31::cm31_to_limbs_gadget;
use crate::qm31::{QM31Mult, QM31MultGadget};
use crate::treepp::*;
use crate::utils::convert_qm31_from_limbs;
use anyhow::{Error, Result};
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{FunctionMetadata, FunctionOutput};
use itertools::Itertools;
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
            input: vec!["table", "qm31_limbs", "qm31_limbs"],
            output: vec!["qm31"],
        },
    );
}

#[cfg(test)]
mod test {
    use crate::dsl::{load_data_types, load_functions};
    use crate::utils::convert_qm31_to_limbs;
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

        let expected = dsl
            .alloc_constant(
                "qm31_limbs",
                Element::ManyNum(convert_qm31_to_limbs(a_qm31).to_vec()),
            )
            .unwrap();
        let _ = dsl
            .execute("qm31_limbs_equalverify", &[res[0], expected])
            .unwrap();

        test_program(dsl).unwrap();
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

        test_program(dsl).unwrap()
    }
}
