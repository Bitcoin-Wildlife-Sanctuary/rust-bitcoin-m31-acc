use anyhow::{Error, Result};
use bitcoin::script::write_scriptint;
use bitcoin_circle_stark::treepp::*;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{
    FunctionMetadata, FunctionOutput, FunctionWithOptionsMetadata,
};
use bitcoin_script_dsl::options::Options;
use stwo_prover::core::vcs::sha256_hash::{Sha256Hash, Sha256Hasher};

fn new_zip(_: &mut DSL, _: &[usize], options: &Options) -> Result<FunctionOutput> {
    let len = options.get_u32("len")?;
    let init_hash = Sha256Hasher::hash(&len.to_le_bytes());

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(init_hash.as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

fn new_zip_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let len = options.get_u32("len")?;
    let init_hash = Sha256Hasher::hash(&len.to_le_bytes());

    Ok(script! {
        { init_hash.as_ref().to_vec() }
    })
}

fn zip_num(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let mut hash = Sha256Hash::from(dsl.get_str(inputs[0])?);
    let num = dsl.get_num(inputs[1])?;

    let hashed_num = Sha256Hasher::hash(&convert_num_to_bytes(num));
    hash = Sha256Hasher::concat_and_hash(&Sha256Hash::from(hash), &hashed_num);

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(hash.as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

fn zip_num_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_SHA256
        OP_CAT
        OP_SHA256
    })
}

fn zip_str(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let mut hash = Sha256Hash::from(dsl.get_str(inputs[0])?);
    let str = dsl.get_str(inputs[1])?;

    let hashed_str = Sha256Hasher::hash(str);
    hash = Sha256Hasher::concat_and_hash(&Sha256Hash::from(hash), &hashed_str);

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(hash.as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

fn zip_str_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_SHA256
        OP_CAT
        OP_SHA256
    })
}

fn zip_many_num(dsl: &mut DSL, inputs: &[usize], options: &Options) -> Result<FunctionOutput> {
    let mut hash = Sha256Hash::from(dsl.get_str(inputs[0])?);
    let many_num = dsl.get_many_num(inputs[1])?;

    if many_num.len() != options.get_u32("len")? as usize {
        return Err(Error::msg(
            "The provided length of stack elements does not match with the element in the memory",
        ));
    }

    for &num in many_num.iter().rev() {
        let hashed_num = Sha256Hasher::hash(&convert_num_to_bytes(num));
        hash = Sha256Hasher::concat_and_hash(&Sha256Hash::from(hash), &hashed_num);
    }

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(hash.as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

fn zip_many_num_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let len = options.get_u32("len")?;

    Ok(script! {
        // note: given that the input is supplied from outside and can be dynamic,
        // one must be careful about how to use the hash functions to achieve
        // collision-resistance in the malicious setting.

        { len } OP_ROLL // move the current hash to the bottom
        for _ in 0..len { // hash the elements one by one
            OP_SWAP
            OP_SHA256
            OP_CAT
            OP_SHA256
        }
    })
}

fn zip_many_str(dsl: &mut DSL, inputs: &[usize], options: &Options) -> Result<FunctionOutput> {
    let mut hash = Sha256Hash::from(dsl.get_str(inputs[0])?);
    let many_str = dsl.get_many_str(inputs[1])?;

    if many_str.len() != options.get_u32("len")? as usize {
        return Err(Error::msg(
            "The provided length of stack elements does not match with the element in the memory",
        ));
    }

    for str in many_str.iter().rev() {
        let hashed_str = Sha256Hasher::hash(str);
        hash = Sha256Hasher::concat_and_hash(&Sha256Hash::from(hash), &hashed_str);
    }

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new(
            "hash",
            Element::Str(hash.as_ref().to_vec()),
        )],
        new_hints: vec![],
    })
}

fn zip_many_str_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let len = options.get_u32("len")?;

    Ok(script! {
        for _ in 0..len {
            OP_SHA256
            OP_CAT
            OP_SHA256
        }
    })
}

pub(crate) fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "new_zip",
        FunctionWithOptionsMetadata {
            trace_generator: new_zip,
            script_generator: new_zip_gadget,
            input: vec![],
            output: vec!["hash"],
        },
    )?;
    dsl.add_function(
        "hashequalverify",
        FunctionMetadata {
            trace_generator: hashequalverify,
            script_generator: hashequalverify_gadget,
            input: vec!["hash", "hash"],
            output: vec![],
        },
    )?;
    dsl.add_function(
        "zip_num",
        FunctionMetadata {
            trace_generator: zip_num,
            script_generator: zip_num_gadget,
            input: vec!["hash", "any"],
            output: vec!["hash"],
        },
    )?;
    dsl.add_function(
        "zip_str",
        FunctionMetadata {
            trace_generator: zip_str,
            script_generator: zip_str_gadget,
            input: vec!["hash", "any"],
            output: vec!["hash"],
        },
    )?;
    dsl.add_function(
        "zip_many_num",
        FunctionWithOptionsMetadata {
            trace_generator: zip_many_num,
            script_generator: zip_many_num_gadget,
            input: vec!["hash", "any"],
            output: vec!["hash"],
        },
    )?;
    dsl.add_function(
        "zip_many_str",
        FunctionWithOptionsMetadata {
            trace_generator: zip_many_str,
            script_generator: zip_many_str_gadget,
            input: vec!["hash", "any"],
            output: vec!["hash"],
        },
    )?;

    Ok(())
}

fn convert_num_to_bytes(v: i32) -> Vec<u8> {
    let mut buffer = [0u8; 8];
    let len = write_scriptint(&mut buffer, v as i64);
    buffer[0..len].to_vec()
}

pub fn zip_elements(dsl: &mut DSL, list: &[usize]) -> Result<(usize, Vec<MemoryEntry>)> {
    let mut zipper = dsl.execute_with_options(
        "new_zip",
        &[],
        &Options::new().with_u32("len", list.len() as u32),
    )?[0];

    let mut zipped_entries = vec![];

    for &idx in list.iter() {
        let entry = dsl.memory.get(&idx);
        if entry.is_none() {
            return Err(Error::msg("The entry does not exist in the memory"));
        }
        let entry = entry.unwrap();
        zipped_entries.push(entry.clone());

        match &entry.data {
            Element::Num(_) => {
                zipper = dsl.execute("zip_num", &[zipper, idx])?[0];
            }
            Element::ManyNum(v) => {
                zipper = dsl.execute_with_options(
                    "zip_many_num",
                    &[zipper, idx],
                    &Options::new().with_u32("len", v.len() as u32),
                )?[0];
            }
            Element::Str(_) => {
                zipper = dsl.execute("zip_str", &[zipper, idx])?[0];
            }
            Element::ManyStr(v) => {
                zipper = dsl.execute_with_options(
                    "zip_many_str",
                    &[zipper, idx],
                    &Options::new().with_u32("len", v.len() as u32),
                )?[0];
            }
        }
    }

    Ok((zipper, zipped_entries))
}

pub fn unzip_elements(
    dsl: &mut DSL,
    expected_zipper: usize,
    memory_entries: &[MemoryEntry],
) -> Result<Vec<usize>> {
    let mut result = vec![];
    for memory_entry in memory_entries.iter() {
        result.push(dsl.alloc_hint(memory_entry.data_type.clone(), memory_entry.data.clone())?);
    }
    let (zipper, _) = zip_elements(dsl, &result)?;
    let _ = dsl.execute("hashequalverify", &[expected_zipper, zipper])?;

    Ok(result)
}

pub fn hashequalverify(dsl: &mut DSL, inputs: &[usize]) -> Result<FunctionOutput> {
    let a = dsl.get_str(inputs[0])?.to_vec();
    let b = dsl.get_str(inputs[1])?;

    if a != b {
        Err(Error::msg("The two hashes do not match"))
    } else {
        Ok(FunctionOutput {
            new_elements: vec![],
            new_hints: vec![],
        })
    }
}

pub fn hashequalverify_gadget(_: &[usize]) -> Result<Script> {
    Ok(script! {
        OP_EQUALVERIFY
    })
}

#[cfg(test)]
mod test {
    use crate::algorithms::utils::convert_qm31_to_limbs;
    use crate::dsl::building_blocks::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::tools::{unzip_elements, zip_elements};
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::vcs::sha256_hash::Sha256Hash;

    #[test]
    fn test_zip_and_unzip() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let test_m31 = M31::reduce(prng.next_u64());
        let test_qm31 = get_rand_qm31(&mut prng);
        let test_hash = Sha256Hash::from(prng.get_seed().as_slice());
        let test_qm31_limbs = convert_qm31_to_limbs(get_rand_qm31(&mut prng));

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let test_m31_var = dsl
            .alloc_input("m31", Element::Num(test_m31.0 as i32))
            .unwrap();
        let test_qm31_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(test_qm31)),
            )
            .unwrap();
        let test_hash_var = dsl
            .alloc_input("hash", Element::Str(test_hash.as_ref().to_vec()))
            .unwrap();
        let test_qm31_limbs_var = dsl
            .alloc_input("qm31_limbs", Element::ManyNum(test_qm31_limbs.to_vec()))
            .unwrap();

        let (hash_var, zipped_entries) = zip_elements(
            &mut dsl,
            &[
                test_m31_var,
                test_qm31_var,
                test_hash_var,
                test_qm31_limbs_var,
            ],
        )
        .unwrap();

        let hash = dsl.get_str(hash_var).unwrap().to_vec();

        dsl.set_program_output("hash", hash_var).unwrap();
        test_program(
            dsl,
            script! {
                { hash.clone() }
            },
        )
        .unwrap();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let expected_hash = dsl.alloc_input("hash", Element::Str(hash.clone())).unwrap();
        let res = unzip_elements(&mut dsl, expected_hash, &zipped_entries).unwrap();

        dsl.set_program_output("m31", res[0]).unwrap();
        dsl.set_program_output("qm31", res[1]).unwrap();
        dsl.set_program_output("hash", res[2]).unwrap();
        dsl.set_program_output("qm31_limbs", res[3]).unwrap();

        test_program(
            dsl,
            script! {
                { test_m31 }
                { test_qm31 }
                { test_hash }
                { test_qm31_limbs.to_vec() }
            },
        )
        .unwrap()
    }
}
