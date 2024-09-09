use crate::algorithms::qm31::{QM31Mult, QM31MultGadget};
use crate::algorithms::utils::{
    check_limb_format, convert_qm31_from_limbs, convert_qm31_to_limbs, OP_HINT,
};
use crate::dsl::building_blocks::cm31::{
    cm31_mul_m31_limbs, cm31_to_limbs_gadget, reformat_cm31_from_dsl_element,
};
use anyhow::{Error, Result};
use bitcoin_circle_stark::treepp::*;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{FunctionMetadata, FunctionOutput};
use itertools::Itertools;
use num_traits::{One, Zero};
use rust_bitcoin_m31::{
    cm31_add, m31_add, m31_add_n31, m31_sub, push_m31_one, push_n31_one, push_qm31_one,
    qm31_add as raw_qm31_add, qm31_equalverify as raw_qm31_equalverify, qm31_neg as raw_qm31_neg,
    qm31_shift_by_i as raw_qm31_shift_by_i, qm31_shift_by_ij as raw_qm31_shift_by_ij,
    qm31_shift_by_j as raw_qm31_shift_by_j, qm31_sub as raw_qm31_sub, qm31_swap,
};
use std::ops::{Add, Neg, Sub};
use stwo_prover::core::fields::cm31::CM31;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::QM31;
use stwo_prover::core::fields::FieldExpOps;

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

pub fn qm31_limbs_first(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31_limbs",
            Element::ManyNum(a[0..8].to_vec()),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_limbs_first_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..8 {
            { r[0] } OP_PICK
        }
    })
}

pub fn qm31_limbs_second(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "cm31_limbs",
            Element::ManyNum(a[8..16].to_vec()),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_limbs_second_gadget(r: &[usize]) -> Result<Script> {
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
    Ok(raw_qm31_equalverify())
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

pub fn qm31_1add(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(a.add(QM31::one()))),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_1add_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        push_n31_one
        m31_add_n31
    })
}

pub fn qm31_1sub(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(a.sub(QM31::one()))),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_1sub_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        push_m31_one
        m31_sub
    })
}

pub fn qm31_neg(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(a.neg())),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_neg_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_qm31_neg())
}

pub fn qm31_add(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    let b = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[1])?);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(a + b)),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_add_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_qm31_add())
}

pub fn qm31_sub(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    let b = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[1])?);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(a - b)),
        )],
        new_hints: vec![],
    })
}

pub fn qm31_sub_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_qm31_sub())
}

fn qm31_from_first_and_second(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();
    let b = dsl.get_many_num(inputs[1])?;

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(vec![b[0], b[1], a[0], a[1]]),
        )],
        new_hints: vec![],
    })
}

fn qm31_from_first_and_second_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_2SWAP
    })
}

fn qm31_add_cm31(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let qm31 = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    let cm31 = reformat_cm31_from_dsl_element(dsl.get_many_num(inputs[1])?);

    let res = qm31.add(QM31(cm31, CM31::zero()));

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn qm31_add_cm31_gadget(_: &[usize]) -> Result<Script> {
    Ok(cm31_add())
}

fn qm31_add_m31(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let qm31 = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);
    let m31 = dsl.get_num(inputs[1])?;

    let res = qm31.add(M31::from_u32_unchecked(m31 as u32));

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn qm31_add_m31_gadget(_: &[usize]) -> Result<Script> {
    Ok(m31_add())
}

pub fn qm31_mul_m31_limbs(
    dsl: &mut DSL,
    table: usize,
    qm31: usize,
    m31_limbs: usize,
) -> Result<usize> {
    let first = dsl.execute("qm31_first", &[qm31])?[0];
    let first_res = cm31_mul_m31_limbs(dsl, table, first, m31_limbs)?;

    let second = dsl.execute("qm31_second", &[qm31])?[0];
    let second_res = cm31_mul_m31_limbs(dsl, table, second, m31_limbs)?;

    let res = dsl.execute("qm31_from_first_and_second", &[first_res, second_res])?[0];
    Ok(res)
}

pub fn qm31_mul_cm31_limbs(
    dsl: &mut DSL,
    table: usize,
    qm31: usize,
    cm31_limbs: usize,
) -> Result<usize> {
    let first = dsl.execute("qm31_first", &[qm31])?[0];
    let second = dsl.execute("qm31_second", &[qm31])?[0];

    let first_limbs = dsl.execute("cm31_to_limbs", &[first])?[0];
    let first_res = dsl.execute("cm31_limbs_mul", &[table, first_limbs, cm31_limbs])?[0];

    let second_limbs = dsl.execute("cm31_to_limbs", &[second])?[0];
    let second_res = dsl.execute("cm31_limbs_mul", &[table, second_limbs, cm31_limbs])?[0];

    let res = dsl.execute("qm31_from_first_and_second", &[first_res, second_res])?[0];

    Ok(res)
}

fn qm31_shift_by_i(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let qm31 = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);

    let res = qm31 * QM31::from_u32_unchecked(0, 1, 0, 0);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn qm31_shift_by_i_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_qm31_shift_by_i())
}

fn qm31_shift_by_j(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let qm31 = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);

    let res = qm31 * QM31::from_u32_unchecked(0, 0, 1, 0);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn qm31_shift_by_j_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_qm31_shift_by_j())
}

fn qm31_shift_by_ij(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let qm31 = reformat_qm31_from_dsl_element(dsl.get_many_num(inputs[0])?);

    let res = qm31 * QM31::from_u32_unchecked(0, 0, 0, 1);
    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "qm31",
            Element::ManyNum(reformat_qm31_to_dsl_element(res)),
        )],
        new_hints: vec![],
    })
}

fn qm31_shift_by_ij_gadget(_: &[usize]) -> Result<Script> {
    Ok(raw_qm31_shift_by_ij())
}

fn qm31_limbs_inverse(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[1])?;
    let a_val = convert_qm31_from_limbs(a);

    let inv = a_val.inverse();
    let inv_limbs = convert_qm31_to_limbs(inv);

    let hint = QM31Mult::compute_hint_from_limbs(
        &a[0..8],
        &a[8..16],
        &inv_limbs[0..8],
        &inv_limbs[8..16],
    )?;

    let output_entry = MemoryEntry::new("qm31_limbs", Element::ManyNum(inv_limbs.to_vec()));

    let mut new_hints = vec![output_entry.clone()];
    new_hints.extend([hint.h1, hint.h2, hint.h3].iter().flat_map(|x| {
        vec![
            MemoryEntry::new("qm31", Element::Num(x.q1)),
            MemoryEntry::new("qm31", Element::Num(x.q2)),
            MemoryEntry::new("qm31", Element::Num(x.q3)),
        ]
    }));

    Ok(FunctionOutput {
        new_elements: vec![output_entry],
        new_hints,
    })
}

fn qm31_limbs_inverse_gadget(r: &[usize]) -> Result<Script> {
    Ok(script! {
        for _ in 0..16 {
            OP_HINT check_limb_format OP_DUP OP_TOALTSTACK
        }
        { QM31MultGadget::mult(r[0] - 512) }
        push_qm31_one raw_qm31_equalverify

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP

        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP
        OP_2SWAP

        for _ in 0..4 {
            7 OP_ROLL
        }
        for _ in 0..4 {
            11 OP_ROLL
        }
        for _ in 0..4 {
            15 OP_ROLL
        }
    })
}

fn qm31_conditional_swap(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_many_num(inputs[0])?.to_vec();
    let b = dsl.get_many_num(inputs[1])?.to_vec();
    let bit = dsl.get_num(inputs[2])?;

    if bit != 0 && bit != 1 {
        return Err(Error::msg("The swap bit is expected to be either 0 or 1"));
    }

    let (first, second) = if bit == 0 { (&a, &b) } else { (&b, &a) };

    let mut new_elements = vec![];
    new_elements.push(MemoryEntry::new("qm31", Element::ManyNum(first.to_vec())));
    new_elements.push(MemoryEntry::new("qm31", Element::ManyNum(second.to_vec())));

    Ok(FunctionOutput {
        new_elements,
        new_hints: vec![],
    })
}

fn qm31_conditional_swap_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_IF
            qm31_swap
        OP_ENDIF
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

pub(crate) fn reformat_qm31_from_dsl_element(v: &[i32]) -> QM31 {
    QM31::from_u32_unchecked(v[3] as u32, v[2] as u32, v[1] as u32, v[0] as u32)
}

pub(crate) fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "qm31_to_limbs",
        FunctionMetadata {
            trace_generator: qm31_to_limbs,
            script_generator: qm31_to_limbs_gadget,
            input: vec!["qm31"],
            output: vec!["qm31_limbs"],
        },
    )?;
    dsl.add_function(
        "qm31_limbs_equalverify",
        FunctionMetadata {
            trace_generator: qm31_limbs_equalverify,
            script_generator: qm31_limbs_equalverify_gadget,
            input: vec!["qm31_limbs", "qm31_limbs"],
            output: vec![],
        },
    )?;
    dsl.add_function(
        "qm31_limbs_mul",
        FunctionMetadata {
            trace_generator: qm31_limbs_mul,
            script_generator: qm31_limbs_mul_gadget,
            input: vec!["&table", "qm31_limbs", "qm31_limbs"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_limbs_first",
        FunctionMetadata {
            trace_generator: qm31_limbs_first,
            script_generator: qm31_limbs_first_gadget,
            input: vec!["&qm31_limbs"],
            output: vec!["cm31_limbs"],
        },
    )?;
    dsl.add_function(
        "qm31_limbs_second",
        FunctionMetadata {
            trace_generator: qm31_limbs_second,
            script_generator: qm31_limbs_second_gadget,
            input: vec!["&qm31_limbs"],
            output: vec!["cm31_limbs"],
        },
    )?;
    dsl.add_function(
        "qm31_equalverify",
        FunctionMetadata {
            trace_generator: qm31_equalverify,
            script_generator: qm31_equalverify_gadget,
            input: vec!["qm31", "qm31"],
            output: vec![],
        },
    )?;
    dsl.add_function(
        "qm31_first",
        FunctionMetadata {
            trace_generator: qm31_first,
            script_generator: qm31_first_gadget,
            input: vec!["&qm31"],
            output: vec!["cm31"],
        },
    )?;
    dsl.add_function(
        "qm31_second",
        FunctionMetadata {
            trace_generator: qm31_second,
            script_generator: qm31_second_gadget,
            input: vec!["&qm31"],
            output: vec!["cm31"],
        },
    )?;
    dsl.add_function(
        "qm31_1add",
        FunctionMetadata {
            trace_generator: qm31_1add,
            script_generator: qm31_1add_gadget,
            input: vec!["qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_1sub",
        FunctionMetadata {
            trace_generator: qm31_1sub,
            script_generator: qm31_1sub_gadget,
            input: vec!["qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_neg",
        FunctionMetadata {
            trace_generator: qm31_neg,
            script_generator: qm31_neg_gadget,
            input: vec!["qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_add",
        FunctionMetadata {
            trace_generator: qm31_add,
            script_generator: qm31_add_gadget,
            input: vec!["qm31", "qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_sub",
        FunctionMetadata {
            trace_generator: qm31_sub,
            script_generator: qm31_sub_gadget,
            input: vec!["qm31", "qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_from_first_and_second",
        FunctionMetadata {
            trace_generator: qm31_from_first_and_second,
            script_generator: qm31_from_first_and_second_gadget,
            input: vec!["cm31", "cm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_add_cm31",
        FunctionMetadata {
            trace_generator: qm31_add_cm31,
            script_generator: qm31_add_cm31_gadget,
            input: vec!["qm31", "cm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_add_m31",
        FunctionMetadata {
            trace_generator: qm31_add_m31,
            script_generator: qm31_add_m31_gadget,
            input: vec!["qm31", "m31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_shift_by_i",
        FunctionMetadata {
            trace_generator: qm31_shift_by_i,
            script_generator: qm31_shift_by_i_gadget,
            input: vec!["qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_shift_by_j",
        FunctionMetadata {
            trace_generator: qm31_shift_by_j,
            script_generator: qm31_shift_by_j_gadget,
            input: vec!["qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_shift_by_ij",
        FunctionMetadata {
            trace_generator: qm31_shift_by_ij,
            script_generator: qm31_shift_by_ij_gadget,
            input: vec!["qm31"],
            output: vec!["qm31"],
        },
    )?;
    dsl.add_function(
        "qm31_limbs_inverse",
        FunctionMetadata {
            trace_generator: qm31_limbs_inverse,
            script_generator: qm31_limbs_inverse_gadget,
            input: vec!["&table", "qm31_limbs"],
            output: vec!["qm31_limbs"],
        },
    )?;
    dsl.add_function(
        "qm31_conditional_swap",
        FunctionMetadata {
            trace_generator: qm31_conditional_swap,
            script_generator: qm31_conditional_swap_gadget,
            input: vec!["qm31", "qm31", "position"],
            output: vec!["qm31", "qm31"],
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::algorithms::utils::{
        convert_cm31_to_limbs, convert_m31_to_limbs, convert_qm31_to_limbs,
    };
    use crate::dsl::building_blocks::cm31::reformat_cm31_to_dsl_element;
    use crate::dsl::building_blocks::qm31::{
        qm31_mul_cm31_limbs, qm31_mul_m31_limbs, reformat_qm31_to_dsl_element,
    };
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use itertools::Itertools;
    use num_traits::{One, Zero};
    use rand::{Rng, RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use std::ops::{Add, Neg, Sub};
    use stwo_prover::core::fields::cm31::CM31;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::fields::qm31::QM31;
    use stwo_prover::core::fields::FieldExpOps;

    #[test]
    fn test_qm31_to_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a = dsl
            .alloc_constant(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(a_qm31)),
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

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

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
    fn test_qm31_limbs_first_and_second() {
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

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

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
    fn test_qm31_first_and_second() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);

        let a_first_cm31 = a_qm31.0;
        let a_second_cm31 = a_qm31.1;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

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

    #[test]
    fn test_qm31_1add_1sub_neg() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);
        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);

        let a_1add = a.add(QM31::one());
        let a_1sub = a.sub(QM31::one());
        let a_neg = a.neg();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(a)))
            .unwrap();

        let a_1add_var = dsl.execute("qm31_1add", &[a_var]).unwrap()[0];
        let a_1sub_var = dsl.execute("qm31_1sub", &[a_var]).unwrap()[0];
        let a_neg_var = dsl.execute("qm31_neg", &[a_var]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(a_1add_var).unwrap(),
            reformat_qm31_to_dsl_element(a_1add)
        );
        assert_eq!(
            dsl.get_many_num(a_1sub_var).unwrap(),
            reformat_qm31_to_dsl_element(a_1sub)
        );
        assert_eq!(
            dsl.get_many_num(a_neg_var).unwrap(),
            reformat_qm31_to_dsl_element(a_neg)
        );

        dsl.set_program_output("qm31", a_1add_var).unwrap();
        dsl.set_program_output("qm31", a_1sub_var).unwrap();
        dsl.set_program_output("qm31", a_neg_var).unwrap();

        test_program(
            dsl,
            script! {
                { a_1add }
                { a_1sub }
                { a_neg }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_add_sub() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);
        let a = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();
        let b = (0..4)
            .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
            .collect_vec();

        let a = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);
        let b = QM31::from_u32_unchecked(b[0], b[1], b[2], b[3]);

        let sum = a + b;
        let diff = a - b;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(a)))
            .unwrap();
        let b_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(b)))
            .unwrap();

        let res_var = dsl.execute("qm31_add", &[a_var, b_var]).unwrap()[0];
        assert_eq!(
            dsl.get_many_num(res_var).unwrap(),
            reformat_qm31_to_dsl_element(sum)
        );

        let res2_var = dsl.execute("qm31_sub", &[a_var, b_var]).unwrap()[0];
        assert_eq!(
            dsl.get_many_num(res2_var).unwrap(),
            reformat_qm31_to_dsl_element(diff)
        );

        dsl.set_program_output("qm31", res_var).unwrap();
        dsl.set_program_output("qm31", res2_var).unwrap();

        test_program(
            dsl,
            script! {
                { sum }
                { diff }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_mul_m31_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let qm31 = get_rand_qm31(&mut prng);
        let m31 = M31::reduce(prng.next_u64());

        let expected = qm31 * m31;

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let qm31_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(qm31)))
            .unwrap();
        let m31_limbs_var = dsl
            .alloc_input(
                "m31_limbs",
                Element::ManyNum(convert_m31_to_limbs(m31.0).to_vec()),
            )
            .unwrap();
        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = qm31_mul_m31_limbs(&mut dsl, table, qm31_var, m31_limbs_var).unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(expected)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_mul_cm31_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let qm31 = get_rand_qm31(&mut prng);
        let cm31 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));

        let expected = qm31.mul_cm31(cm31);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let qm31_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(qm31)))
            .unwrap();
        let cm31_limbs_var = dsl
            .alloc_input(
                "cm31_limbs",
                Element::ManyNum(convert_cm31_to_limbs(cm31).to_vec()),
            )
            .unwrap();
        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = qm31_mul_cm31_limbs(&mut dsl, table, qm31_var, cm31_limbs_var).unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(expected)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_add_cm31() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let qm31 = get_rand_qm31(&mut prng);
        let cm31 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));

        let expected = qm31.add(QM31(cm31, CM31::zero()));

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let qm31_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(qm31)))
            .unwrap();
        let cm31_var = dsl
            .alloc_input(
                "cm31",
                Element::ManyNum(reformat_cm31_to_dsl_element(cm31).to_vec()),
            )
            .unwrap();

        let res = dsl.execute("qm31_add_cm31", &[qm31_var, cm31_var]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(expected)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_add_m31() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let qm31 = get_rand_qm31(&mut prng);
        let m31 = M31::reduce(prng.next_u64());

        let expected = qm31.add(m31);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let qm31_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(qm31)))
            .unwrap();
        let m31_var = dsl.alloc_input("m31", Element::Num(m31.0 as i32)).unwrap();

        let res = dsl.execute("qm31_add_m31", &[qm31_var, m31_var]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(expected)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_shift_by_i_j_ij() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let qm31 = get_rand_qm31(&mut prng);
        let qm31_shift_by_i = qm31 * QM31::from_u32_unchecked(0, 1, 0, 0);
        let qm31_shift_by_j = qm31 * QM31::from_u32_unchecked(0, 0, 1, 0);
        let qm31_shift_by_ij = qm31 * QM31::from_u32_unchecked(0, 0, 0, 1);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let qm31_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(qm31)))
            .unwrap();

        let qm31_shift_by_i_var = dsl.execute("qm31_shift_by_i", &[qm31_var]).unwrap()[0];
        let qm31_shift_by_j_var = dsl.execute("qm31_shift_by_j", &[qm31_var]).unwrap()[0];
        let qm31_shift_by_ij_var = dsl.execute("qm31_shift_by_ij", &[qm31_var]).unwrap()[0];

        assert_eq!(
            dsl.get_many_num(qm31_shift_by_i_var).unwrap(),
            reformat_qm31_to_dsl_element(qm31_shift_by_i)
        );
        assert_eq!(
            dsl.get_many_num(qm31_shift_by_j_var).unwrap(),
            reformat_qm31_to_dsl_element(qm31_shift_by_j)
        );
        assert_eq!(
            dsl.get_many_num(qm31_shift_by_ij_var).unwrap(),
            reformat_qm31_to_dsl_element(qm31_shift_by_ij)
        );

        dsl.set_program_output("qm31", qm31_shift_by_i_var).unwrap();
        dsl.set_program_output("qm31", qm31_shift_by_j_var).unwrap();
        dsl.set_program_output("qm31", qm31_shift_by_ij_var)
            .unwrap();

        test_program(
            dsl,
            script! {
                { qm31_shift_by_i }
                { qm31_shift_by_j }
                { qm31_shift_by_ij }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_qm31_limbs_inverse() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = get_rand_qm31(&mut prng);
        let a_limbs = convert_qm31_to_limbs(a);

        let inv = a.inverse();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a = dsl
            .alloc_input("qm31_limbs", Element::ManyNum(a_limbs.to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = dsl.execute("qm31_limbs_inverse", &[table, a]).unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(
            dsl.get_many_num(res[0]).unwrap(),
            convert_qm31_to_limbs(inv)
        );

        dsl.set_program_output("qm31_limbs", res[0]).unwrap();

        test_program(
            dsl,
            script! {
                { convert_qm31_to_limbs(inv).to_vec() }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_qm31_conditional_swap() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = get_rand_qm31(&mut prng);
        let b = get_rand_qm31(&mut prng);

        let mut dsl = DSL::new();
        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let a_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(a)))
            .unwrap();
        let b_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(b)))
            .unwrap();

        let bit_0_var = dsl.alloc_input("position", Element::Num(0)).unwrap();
        let bit_1_var = dsl.alloc_input("position", Element::Num(1)).unwrap();

        let res1 = dsl
            .execute("qm31_conditional_swap", &[a_var, b_var, bit_0_var])
            .unwrap();
        let res2 = dsl
            .execute("qm31_conditional_swap", &[a_var, b_var, bit_1_var])
            .unwrap();

        assert_eq!(
            dsl.get_many_num(res1[0]).unwrap(),
            reformat_qm31_to_dsl_element(a)
        );
        assert_eq!(
            dsl.get_many_num(res1[1]).unwrap(),
            reformat_qm31_to_dsl_element(b)
        );
        assert_eq!(
            dsl.get_many_num(res2[0]).unwrap(),
            reformat_qm31_to_dsl_element(b)
        );
        assert_eq!(
            dsl.get_many_num(res2[1]).unwrap(),
            reformat_qm31_to_dsl_element(a)
        );

        dsl.set_program_output("qm31", res1[0]).unwrap();
        dsl.set_program_output("qm31", res1[1]).unwrap();
        dsl.set_program_output("qm31", res2[0]).unwrap();
        dsl.set_program_output("qm31", res2[1]).unwrap();

        test_program(
            dsl,
            script! {
                { a }
                { b }
                { b }
                { a }
            },
        )
        .unwrap();
    }
}
