use anyhow::{Error, Result};
use bitcoin_circle_stark::merkle_tree::{
    MerkleTreePath, MerkleTreePathGadget, MerkleTreeTwinGadget, MerkleTreeTwinProof,
};
use bitcoin_circle_stark::precomputed_merkle_tree::{
    PrecomputedMerkleTree, PrecomputedMerkleTreeGadget, PrecomputedMerkleTreeProof,
};
use bitcoin_circle_stark::treepp::*;
use bitcoin_circle_stark::utils::limb_to_be_bits_toaltstack_except_lowest_1bit;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::{FunctionOutput, FunctionWithOptionsMetadata};
use bitcoin_script_dsl::options::Options;
use itertools::Itertools;
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::vcs::sha256_hash::Sha256Hash;

fn query_and_verify_merkle_twin_tree<const N: usize>(
    dsl: &mut DSL,
    inputs: &[usize],
    options: &Options,
) -> Result<FunctionOutput> {
    let root_hash = dsl.get_str(inputs[0])?.to_vec();
    let pos = dsl.get_num(inputs[1])?;

    let left = options.get_multi_u32("left")?;
    let right = options.get_multi_u32("right")?;
    let path = options.get_multi_binary("path")?;

    if left.len() != right.len() || left.len() != N {
        return Err(Error::msg("The number of elements on the leaf mismatches"));
    }

    // verify the Merkle tree externally first
    let proof = MerkleTreeTwinProof {
        left: left
            .iter()
            .map(|&x| M31::from_u32_unchecked(x))
            .collect_vec(),
        right: right
            .iter()
            .map(|&x| M31::from_u32_unchecked(x))
            .collect_vec(),
        path: MerkleTreePath {
            siblings: path
                .iter()
                .map(|x| Sha256Hash::from(x.as_slice()))
                .collect_vec(),
        },
    };

    let mut pos = pos;
    if pos % 2 == 1 {
        pos -= 1;
    }

    let proof_is_valid = proof.verify(&Sha256Hash::from(root_hash), path.len() + 1, pos as usize);
    if !proof_is_valid {
        return Err(Error::msg("Merkle tree proof is invalid"));
    }

    let mut new_elements = vec![];
    let mut new_hints = vec![];

    for &elem in left.iter().chain(right.iter()) {
        let memory_entry = MemoryEntry::new("m31", Element::Num(elem as i32));
        new_elements.push(memory_entry.clone());
        new_hints.push(memory_entry);
    }

    for elem in path.iter() {
        new_hints.push(MemoryEntry::new("internal", Element::Str(elem.clone())));
    }

    Ok(FunctionOutput {
        new_elements,
        new_hints,
    })
}

fn query_and_verify_merkle_twin_tree_gadget<const N: usize>(
    _: &[usize],
    options: &Options,
) -> Result<Script> {
    let path_len = options.get_multi_binary("path")?.len();
    let logn = path_len + 1;

    Ok(MerkleTreeTwinGadget::query_and_verify(N, logn))
}

fn query_and_verify_raw_merkle_tree(
    dsl: &mut DSL,
    inputs: &[usize],
    options: &Options,
) -> Result<FunctionOutput> {
    let root_hash = dsl.get_str(inputs[0])?.to_vec();
    let leaf_hash = dsl.get_str(inputs[1])?.to_vec();
    let raw_position = dsl.get_num(inputs[2])? as usize;

    let path = options.get_multi_binary("path")?;
    let logn = options.get_u32("logn")?;
    let shift = options.get_u32("shift")? as usize;

    if shift < 1 {
        return Err(Error::msg(
            "Raw Merkle tree always assume at least a shift of 1",
        ));
    }

    if path.len() != logn as usize - shift {
        return Err(Error::msg(
            "Merkle tree proof seems to have an incorrect length",
        ));
    }

    let position = raw_position >> shift;

    let proof = MerkleTreePath {
        siblings: path
            .iter()
            .map(|x| Sha256Hash::from(x.as_slice()))
            .collect_vec(),
    };
    let depth = path.len();

    let proof_is_valid = proof.verify(
        &Sha256Hash::from(root_hash.as_slice()),
        depth,
        Sha256Hash::from(leaf_hash.as_slice()),
        position,
    );
    if !proof_is_valid {
        return Err(Error::msg("Merkle tree proof is invalid"));
    }

    Ok(FunctionOutput {
        new_elements: vec![],
        new_hints: path
            .iter()
            .map(|x| MemoryEntry::new("internal", Element::Str(x.clone())))
            .collect_vec(),
    })
}

fn query_and_verify_raw_merkle_tree_gadget(_: &[usize], options: &Options) -> Result<Script> {
    let logn = options.get_u32("logn")?;
    let shift = options.get_u32("shift")? as usize;
    Ok(script! {
        // push the root hash to the altstack
        2 OP_ROLL OP_TOALTSTACK

        // perform a bit decomposition
        { limb_to_be_bits_toaltstack_except_lowest_1bit(logn) }
        if shift > 1 {
            for _ in 1..shift {
                OP_FROMALTSTACK OP_DROP
            }
        }

        { MerkleTreePathGadget::verify(logn as usize - shift) }
    })
}

fn query_and_verify_precomputed_merkle_tree<const N: usize>(
    dsl: &mut DSL,
    inputs: &[usize],
    options: &Options,
) -> Result<FunctionOutput> {
    let root_hash = options.get_binary("root_hash")?;

    let pos = dsl.get_num(inputs[0])? as usize;
    let circle_point_x = options.get_u32("circle_point_x")?;
    let circle_point_y = options.get_u32("circle_point_y")?;
    let twiddles_elements = options.get_multi_u32("twiddles")?;
    let siblings = options.get_multi_binary("siblings")?;

    if twiddles_elements.len() != N {
        return Err(Error::msg(
            "The number of twiddles elements does not match the function signature",
        ));
    }

    if siblings.len() != N {
        return Err(Error::msg(
            "The number of siblings elements does not match the function signature",
        ));
    }

    let proof = PrecomputedMerkleTreeProof {
        circle_point: CirclePoint {
            x: M31::from(circle_point_x),
            y: M31::from(circle_point_y),
        },
        twiddles_elements: twiddles_elements
            .iter()
            .map(|&x| M31::from(x))
            .collect_vec(),
        siblings: siblings
            .iter()
            .map(|x| TryInto::<[u8; 32]>::try_into(x.clone()).unwrap())
            .collect_vec(),
    };

    let proof_is_valid = PrecomputedMerkleTree::verify(
        TryInto::<[u8; 32]>::try_into(root_hash).unwrap(),
        N,
        &proof,
        pos,
    );
    if !proof_is_valid {
        return Err(Error::msg("Merkle tree proof is invalid"));
    }

    let circle_point_x_entry = MemoryEntry::new("m31", Element::Num(circle_point_x as i32));
    let circle_point_y_entry = MemoryEntry::new("m31", Element::Num(circle_point_y as i32));

    let mut new_elements = vec![];
    for &twiddles_element in twiddles_elements.iter() {
        new_elements.push(MemoryEntry::new(
            "m31",
            Element::Num(twiddles_element as i32),
        ));
    }
    new_elements.push(circle_point_x_entry.clone());
    new_elements.push(circle_point_y_entry.clone());

    let mut new_hints = vec![];
    new_hints.push(circle_point_x_entry);
    new_hints.push(circle_point_y_entry);

    new_hints.push(MemoryEntry::new(
        "m31",
        Element::Num(*twiddles_elements.last().unwrap() as i32),
    ));

    for (element, sibling) in proof
        .twiddles_elements
        .iter()
        .rev()
        .skip(1)
        .zip(proof.siblings.iter())
    {
        new_hints.push(MemoryEntry::new("m31", Element::Num(element.0 as i32)));
        new_hints.push(MemoryEntry::new("internal", Element::Str(sibling.to_vec())));
    }

    new_hints.push(MemoryEntry::new(
        "internal",
        Element::Str(siblings.last().unwrap().to_vec()),
    ));

    Ok(FunctionOutput {
        new_elements,
        new_hints,
    })
}

fn query_and_verify_precomputed_merkle_tree_gadget<const N: usize>(
    _: &[usize],
    options: &Options,
) -> Result<Script> {
    let root_hash = TryInto::<[u8; 32]>::try_into(options.get_binary("root_hash")?)?;

    Ok(script! {
        { PrecomputedMerkleTreeGadget::query_and_verify(root_hash, N + 1) }
    })
}

pub(crate) fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "merkle_twin_tree_1",
        FunctionWithOptionsMetadata {
            trace_generator: query_and_verify_merkle_twin_tree::<1>,
            script_generator: query_and_verify_merkle_twin_tree_gadget::<1>,
            input: vec!["hash", "position"],
            output: vec!["m31", "m31"],
        },
    )?;
    dsl.add_function(
        "merkle_twin_tree_4",
        FunctionWithOptionsMetadata {
            trace_generator: query_and_verify_merkle_twin_tree::<4>,
            script_generator: query_and_verify_merkle_twin_tree_gadget::<4>,
            input: vec!["hash", "position"],
            output: vec!["m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31"],
        },
    )?;
    dsl.add_function(
        "raw_merkle_tree",
        FunctionWithOptionsMetadata {
            trace_generator: query_and_verify_raw_merkle_tree,
            script_generator: query_and_verify_raw_merkle_tree_gadget,
            input: vec!["hash", "hash", "position"],
            output: vec![],
        },
    )?;
    dsl.add_function(
        "precomputed_merkle_tree_14",
        FunctionWithOptionsMetadata {
            trace_generator: query_and_verify_precomputed_merkle_tree::<14>,
            script_generator: query_and_verify_precomputed_merkle_tree_gadget::<14>,
            input: vec!["position"],
            output: vec![
                "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31",
                "m31", "m31", "m31", "m31",
            ],
        },
    )?;
    dsl.add_function(
        "precomputed_merkle_tree_15",
        FunctionWithOptionsMetadata {
            trace_generator: query_and_verify_precomputed_merkle_tree::<15>,
            script_generator: query_and_verify_precomputed_merkle_tree_gadget::<15>,
            input: vec!["position"],
            output: vec![
                "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31", "m31",
                "m31", "m31", "m31", "m31", "m31",
            ],
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::merkle_tree::{MerkleTree, MerkleTreePath, MerkleTreeTwinProof};
    use bitcoin_circle_stark::precomputed_merkle_tree::PrecomputedMerkleTree;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::options::Options;
    use bitcoin_script_dsl::test_program;
    use itertools::Itertools;
    use rand::{Rng, RngCore, SeedableRng};
    use rand_chacha::{ChaCha20Rng, ChaCha8Rng};
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::vcs::ops::MerkleHasher;
    use stwo_prover::core::vcs::sha256_merkle::Sha256MerkleHasher;

    #[test]
    fn test_merkle_twin_tree_4() {
        let mut prng = ChaCha8Rng::seed_from_u64(0);

        let logn = 5;

        let mut last_layer = vec![];
        for _ in 0..(1 << logn) {
            let a = get_rand_qm31(&mut prng);
            last_layer.push(a.to_m31_array().to_vec());
        }

        let merkle_tree = MerkleTree::new(last_layer.clone());

        let mut pos: u32 = prng.gen();
        pos &= (1 << logn) - 1;
        if pos % 2 == 1 {
            pos -= 1;
        }

        let proof = MerkleTreeTwinProof::query(&merkle_tree, pos as usize);
        assert!(proof.verify(&merkle_tree.root_hash, logn, pos as usize));

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let root_hash_var = dsl
            .alloc_input(
                "hash",
                Element::Str(merkle_tree.root_hash.as_ref().to_vec()),
            )
            .unwrap();
        let pos_var = dsl
            .alloc_input("position", Element::Num(pos as i32))
            .unwrap();

        let res = dsl
            .execute_with_options(
                "merkle_twin_tree_4",
                &[root_hash_var, pos_var],
                &Options::new()
                    .with_multi_u32("left", proof.left.iter().map(|x| x.0).collect_vec())
                    .with_multi_u32("right", proof.right.iter().map(|x| x.0).collect_vec())
                    .with_multi_binary(
                        "path",
                        proof
                            .path
                            .siblings
                            .iter()
                            .map(|x| x.as_ref().to_vec())
                            .collect_vec(),
                    ),
            )
            .unwrap();

        for (&idx, expected) in res.iter().zip(proof.left.iter().chain(proof.right.iter())) {
            assert_eq!(dsl.get_num(idx).unwrap(), expected.0 as i32,);
        }

        for &idx in res.iter() {
            dsl.set_program_output("m31", idx).unwrap();
        }

        test_program(
            dsl,
            script! {
                for elem in proof.left.iter() {
                    { *elem }
                }
                for elem in proof.right.iter() {
                    { *elem }
                }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_merkle_twin_tree_1() {
        let mut prng = ChaCha8Rng::seed_from_u64(0);

        let logn = 5;

        let mut last_layer = vec![];
        for _ in 0..(1 << logn) {
            let a = M31::reduce(prng.next_u64());
            last_layer.push(vec![a]);
        }

        let merkle_tree = MerkleTree::new(last_layer.clone());

        let mut pos: u32 = prng.gen();
        pos &= (1 << logn) - 1;
        if pos % 2 == 1 {
            pos -= 1;
        }

        let proof = MerkleTreeTwinProof::query(&merkle_tree, pos as usize);
        assert!(proof.verify(&merkle_tree.root_hash, logn, pos as usize));

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let root_hash_var = dsl
            .alloc_input(
                "hash",
                Element::Str(merkle_tree.root_hash.as_ref().to_vec()),
            )
            .unwrap();
        let pos_var = dsl
            .alloc_input("position", Element::Num(pos as i32))
            .unwrap();

        let res = dsl
            .execute_with_options(
                "merkle_twin_tree_1",
                &[root_hash_var, pos_var],
                &Options::new()
                    .with_multi_u32("left", vec![proof.left[0].0])
                    .with_multi_u32("right", vec![proof.right[0].0])
                    .with_multi_binary(
                        "path",
                        proof
                            .path
                            .siblings
                            .iter()
                            .map(|x| x.as_ref().to_vec())
                            .collect_vec(),
                    ),
            )
            .unwrap();

        assert_eq!(dsl.get_num(res[0]).unwrap(), proof.left[0].0 as i32,);
        assert_eq!(dsl.get_num(res[1]).unwrap(), proof.right[0].0 as i32,);

        dsl.set_program_output("m31", res[0]).unwrap();
        dsl.set_program_output("m31", res[1]).unwrap();

        test_program(
            dsl,
            script! {
                { proof.left[0] }
                { proof.right[0] }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_merkle_tree() {
        let mut prng = ChaCha8Rng::seed_from_u64(0);

        let mut last_layer = vec![];
        for _ in 0..1 << 12 {
            let a = get_rand_qm31(&mut prng);
            last_layer.push(a.to_m31_array().to_vec());
        }

        let merkle_tree = MerkleTree::new(last_layer.clone());

        let proof = MerkleTreePath::query(&merkle_tree, 100);

        let last_layer_hash = {
            let left = Sha256MerkleHasher::hash_node(None, last_layer[100].as_slice());
            let right = Sha256MerkleHasher::hash_node(None, last_layer[101].as_slice());
            Sha256MerkleHasher::hash_node(Some((left, right)), &[])
        };
        assert!(proof.verify(
            &merkle_tree.root_hash,
            proof.siblings.len(),
            last_layer_hash,
            100 >> 1
        ));

        let mut dsl = DSL::new();
        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let root_hash_var = dsl
            .alloc_input(
                "hash",
                Element::Str(merkle_tree.root_hash.as_ref().to_vec()),
            )
            .unwrap();
        let leaf_hash_var = dsl
            .alloc_input("hash", Element::Str(last_layer_hash.as_ref().to_vec()))
            .unwrap();
        let pos_var = dsl.alloc_input("position", Element::Num(100)).unwrap();

        let _ = dsl
            .execute_with_options(
                "raw_merkle_tree",
                &[root_hash_var, leaf_hash_var, pos_var],
                &Options::new()
                    .with_multi_binary(
                        "path",
                        proof
                            .siblings
                            .iter()
                            .map(|x| x.as_ref().to_vec())
                            .collect_vec(),
                    )
                    .with_u32("logn", 12)
                    .with_u32("shift", 1),
            )
            .unwrap();

        test_program(dsl, script! {}).unwrap();

        let mut dsl = DSL::new();
        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let root_hash_var = dsl
            .alloc_input(
                "hash",
                Element::Str(merkle_tree.root_hash.as_ref().to_vec()),
            )
            .unwrap();
        let leaf_hash_var = dsl
            .alloc_input("hash", Element::Str(last_layer_hash.as_ref().to_vec()))
            .unwrap();
        let pos_var = dsl.alloc_input("position", Element::Num(100 << 1)).unwrap();

        let _ = dsl
            .execute_with_options(
                "raw_merkle_tree",
                &[root_hash_var, leaf_hash_var, pos_var],
                &Options::new()
                    .with_multi_binary(
                        "path",
                        proof
                            .siblings
                            .iter()
                            .map(|x| x.as_ref().to_vec())
                            .collect_vec(),
                    )
                    .with_u32("logn", 13)
                    .with_u32("shift", 2),
            )
            .unwrap();

        test_program(dsl, script! {}).unwrap();
    }

    #[test]
    fn test_precomputed_merkle_tree() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let logn = 15;
        let n_layers = logn - 1;
        let tree = PrecomputedMerkleTree::new(n_layers);

        let mut pos: u32 = prng.gen();
        pos &= (1 << logn) - 1;

        let proof = tree.query(pos as usize);
        assert!(PrecomputedMerkleTree::verify(
            tree.root_hash,
            n_layers,
            &proof,
            pos as usize
        ));

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let pos_var = dsl
            .alloc_input("position", Element::Num(pos as i32))
            .unwrap();
        let res = dsl
            .execute_with_options(
                "precomputed_merkle_tree_14",
                &[pos_var],
                &Options::new()
                    .with_binary("root_hash", tree.root_hash.to_vec())
                    .with_u32("circle_point_x", proof.circle_point.x.0)
                    .with_u32("circle_point_y", proof.circle_point.y.0)
                    .with_multi_u32(
                        "twiddles",
                        proof.twiddles_elements.iter().map(|x| x.0).collect_vec(),
                    )
                    .with_multi_binary(
                        "siblings",
                        proof.siblings.iter().map(|x| x.to_vec()).collect_vec(),
                    ),
            )
            .unwrap();

        for (&res_entry, expected) in res.iter().zip(proof.twiddles_elements.iter()) {
            assert_eq!(dsl.get_num(res_entry).unwrap(), expected.0 as i32);
        }
        assert_eq!(
            dsl.get_num(res[n_layers]).unwrap(),
            proof.circle_point.x.0 as i32
        );
        assert_eq!(
            dsl.get_num(res[n_layers + 1]).unwrap(),
            proof.circle_point.y.0 as i32
        );

        for &res_entry in res.iter() {
            dsl.set_program_output("m31", res_entry).unwrap();
        }

        test_program(
            dsl,
            script! {
                for twiddles_element in proof.twiddles_elements.iter() {
                    { twiddles_element.0 }
                }
                { proof.circle_point.x }
                { proof.circle_point.y }
            },
        )
        .unwrap();
    }
}
