use crate::algorithms::utils::OP_HINT;
use crate::dsl::verifier::hints::Hints;
use anyhow::Result;
use bitcoin::script::write_scriptint;
use bitcoin_circle_stark::treepp::*;
use bitcoin_circle_stark::utils::hash;
use bitcoin_script_dsl::compiler::Compiler;
use bitcoin_script_dsl::dsl::Element;
use bitcoin_scriptexec::utils::scriptint_vec;
use covenants_gadgets::utils::stack_hash::StackHash;
use covenants_gadgets::CovenantProgram;
use sha2::digest::Update;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

pub type Witness = Vec<Vec<u8>>;

pub struct FibonacciVerifierProgram {}

#[derive(Clone)]
pub struct FibonacciVerifierInput {
    pub stack: Witness,
    pub hints: Witness,
}

impl From<FibonacciVerifierInput> for Script {
    fn from(input: FibonacciVerifierInput) -> Script {
        script! {
            for elem in input.stack {
                { elem }
            }
            for elem in input.hints {
                { elem }
            }
        }
    }
}

/// The state of the Fibonacci split program.
#[derive(Clone, Debug)]
pub struct FibonacciVerifierState {
    /// The program counter.
    pub pc: usize,
    /// The hash of the stack.
    pub stack_hash: Vec<u8>,
    /// The stack from the execution.
    pub stack: Vec<Vec<u8>>,
}

impl From<FibonacciVerifierState> for Script {
    fn from(v: FibonacciVerifierState) -> Self {
        script! {
            { v.pc }
            { v.stack_hash }
        }
    }
}

pub struct FibonacciAllInformation {
    pub scripts: Vec<Script>,
    pub witnesses: Vec<Witness>,
    pub outputs: Vec<Witness>,
}

pub static FIBONACCI_ALL_INFORMATION: OnceLock<FibonacciAllInformation> = OnceLock::new();

impl FibonacciAllInformation {
    pub fn get_input(&self, idx: usize) -> FibonacciVerifierInput {
        FibonacciVerifierInput {
            stack: if idx == 0 {
                vec![]
            } else {
                self.outputs[idx - 1].clone()
            },
            hints: self.witnesses[idx].clone(),
        }
    }
}

pub fn compute_all_information() -> FibonacciAllInformation {
    let mut scripts = vec![];
    let mut witnesses = vec![];

    let hints = Hints::instance();
    let mut cache = HashMap::new();

    let num_to_str = |v: i32| {
        let mut out = [0u8; 8];
        let len = write_scriptint(&mut out, v as i64);
        out[0..len].to_vec()
    };

    for f in [
        super::part1_fiat_shamir_plus_precomputed_merkle_noncompute::generate_dsl,
        super::part2_fiat_shamir_step_numerator_compute::generate_dsl,
        super::part3_fiat_shamir_boundary_compute::generate_dsl,
        super::part4_fiat_shamir_step_denominator_compute::generate_dsl,
        super::part5_oods_point_compute::generate_dsl,
        super::part6_column_line_coeff_trace_compute::generate_dsl,
        super::part7_column_linear_combination_compute::generate_dsl,
        super::part8_prepared_oods_and_alphas_compute::generate_dsl,
        super::part9_sort_queries_first_3_noncompute::generate_dsl,
        super::part10_sort_queries_last_5_noncompute::generate_dsl,
    ] {
        let dsl = f(&hints, &mut cache).unwrap();
        let program = Compiler::compiler(dsl).unwrap();

        scripts.push(program.script);

        let mut witness = vec![];
        for entry in program.hint.iter() {
            match &entry.data {
                Element::Num(v) => {
                    witness.push(num_to_str(*v));
                }
                Element::ManyNum(vv) => {
                    for &v in vv.iter() {
                        witness.push(num_to_str(v));
                    }
                }
                Element::Str(v) => {
                    witness.push(v.clone());
                }
                Element::ManyStr(vv) => {
                    for v in vv.iter() {
                        witness.push(v.clone());
                    }
                }
            }
        }

        witnesses.push(witness);
    }

    for query_idx in 1..=8 {
        for f in [
            super::per_query_part1_reorganize_noncompute::generate_dsl,
            super::per_query_part2_folding_compute::generate_dsl,
            super::per_query_part3_folding_compute::generate_dsl,
            super::per_query_part4_num_denom1_compute::generate_dsl,
            super::per_query_part5_num2_compute::generate_dsl,
            super::per_query_part6_aggregation2_compute::generate_dsl,
            super::per_query_part7_aggregation3_compute::generate_dsl,
            super::per_query_part8_aggregation4_compute::generate_dsl,
        ] {
            let dsl = f(&hints, &mut cache, query_idx).unwrap();
            let program = Compiler::compiler(dsl).unwrap();

            scripts.push(program.script);

            let mut witness = vec![];
            for entry in program.hint.iter() {
                match &entry.data {
                    Element::Num(v) => {
                        witness.push(num_to_str(*v));
                    }
                    Element::ManyNum(vv) => {
                        for &v in vv.iter() {
                            witness.push(num_to_str(v));
                        }
                    }
                    Element::Str(v) => {
                        witness.push(v.clone());
                    }
                    Element::ManyStr(vv) => {
                        for v in vv.iter() {
                            witness.push(v.clone());
                        }
                    }
                }
            }

            witnesses.push(witness);
        }
    }

    let mut outputs = vec![];
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir_verify1").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir_verify2").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir_verify3").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir_verify4").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("column_line_coeff1").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("column_line_coeff2").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("prepared_oods1").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("prepared_oods2").unwrap().hash }
            { cache.get("after_fiat_shamir").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("shared_information").unwrap().hash }
            { cache.get("query1").unwrap().hash }
            { cache.get("query2").unwrap().hash }
            { cache.get("query3").unwrap().hash }
            { cache.get("unsorted").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("global_state").unwrap().hash }
        })
        .unwrap(),
    );

    for query_idx in 1..=8 {
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_fri_folding1", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_fri_folding2", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_num_denom1", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_num_denom1_result", query_idx)).unwrap().hash }
                { cache.get(&format!("query{}_num2", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_aggregation1_result", query_idx)).unwrap().hash }
                { cache.get(&format!("query{}_aggregation2", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_aggregation3", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("query{}_aggregation4", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
            })
            .unwrap(),
        );
    }

    assert_eq!(scripts.len(), witnesses.len());
    assert_eq!(scripts.len(), outputs.len());

    FibonacciAllInformation {
        scripts,
        witnesses,
        outputs,
    }
}

impl CovenantProgram for FibonacciVerifierProgram {
    type State = FibonacciVerifierState;
    type Input = FibonacciVerifierInput;
    const CACHE_NAME: &'static str = "FIBONACCI";

    fn new() -> Self::State {
        FibonacciVerifierState {
            pc: 0,
            stack_hash: vec![0u8; 32],
            stack: vec![],
        }
    }

    fn get_hash(state: &Self::State) -> Vec<u8> {
        assert_eq!(state.stack_hash.len(), 32);
        let mut sha256 = Sha256::new();
        Update::update(&mut sha256, &scriptint_vec(state.pc as i64));
        Update::update(&mut sha256, &state.stack_hash);
        sha256.finalize().to_vec()
    }

    fn get_all_scripts() -> BTreeMap<usize, Script> {
        let all_information = FIBONACCI_ALL_INFORMATION.get_or_init(compute_all_information);

        let mut map = BTreeMap::new();

        let mut output_stack_size = vec![2, 2, 2, 2, 2, 2, 2, 2, 5, 1];

        for _ in 0..8 {
            output_stack_size.extend_from_slice(&[2, 2, 2, 3, 3, 2, 2, 1]);
        }

        for script_idx in 0..(10 + 8 * 8) {
            map.insert(
                script_idx,
                script! {
                    // input:
                    // - old pc
                    // - old stack hash
                    // - new pc
                    // - new stack hash

                    OP_SWAP { script_idx + 1 } OP_EQUALVERIFY
                    OP_ROT { script_idx } OP_EQUALVERIFY

                    if script_idx == 0 {
                        OP_SWAP { vec![0u8; 32] } OP_EQUALVERIFY

                        // stack:
                        // - new stack hash
                        OP_TOALTSTACK
                    } else {
                        // stack:
                        // - old stack hash
                        // - new stack hash
                        OP_TOALTSTACK OP_TOALTSTACK

                        { StackHash::hash_from_hint(output_stack_size[script_idx - 1]) }
                        OP_FROMALTSTACK OP_EQUALVERIFY
                    }

                    { all_information.scripts[script_idx].clone() }

                    OP_DEPTH
                    { output_stack_size[script_idx] }
                    OP_EQUALVERIFY

                    { StackHash::hash_drop(output_stack_size[script_idx]) }
                    OP_FROMALTSTACK OP_EQUALVERIFY
                    OP_TRUE
                },
            );
        }

        map
    }

    fn get_common_prefix() -> Script {
        script! {
            // hint:
            // - old_state
            // - new_state
            //
            // input:
            // - old_state_hash
            // - new_state_hash
            //
            // output:
            // - old pc
            // - old stack hash
            // - new pc
            // - new stack hash
            //

            OP_TOALTSTACK OP_TOALTSTACK

            for _ in 0..2 {
                OP_HINT OP_1ADD OP_1SUB OP_DUP 0 OP_GREATERTHANOREQUAL OP_VERIFY
                OP_HINT OP_SIZE 32 OP_EQUALVERIFY

                OP_2DUP
                OP_CAT
                hash
                OP_FROMALTSTACK OP_EQUALVERIFY
            }
        }
    }

    fn run(id: usize, _: &Self::State, _: &Self::Input) -> Result<Self::State> {
        let all_information = FIBONACCI_ALL_INFORMATION.get_or_init(compute_all_information);

        let final_stack = all_information.outputs[id].to_vec();
        let stack_hash = StackHash::compute(&final_stack);
        Ok(Self::State {
            pc: id + 1,
            stack_hash,
            stack: final_stack,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::dsl::verifier::covenant::{
        compute_all_information, FibonacciVerifierProgram, FibonacciVerifierState,
        FIBONACCI_ALL_INFORMATION,
    };
    use covenants_gadgets::test::{simulation_test, SimulationInstruction};

    #[test]
    fn test_integration() {
        // The integration assumes a fee rate of 7 sat/vByte.
        // Note that in many situations, the fee rate is only 2 sat/vByte.

        let mut fees = vec![
            58975, 90601, 58961, 88459, 37709, 67963, 77924, 75411, 38752, 48055,
        ];

        for _ in 0..8 {
            fees.extend_from_slice(&[11522, 45416, 62055, 58618, 50848, 56889, 62517, 25634]);
        }

        println!(
            "total fee assuming 7 sat/vByte: {}",
            fees.iter().sum::<usize>()
        );

        let mut test_generator = |old_state: &FibonacciVerifierState| {
            let all_information = FIBONACCI_ALL_INFORMATION.get_or_init(compute_all_information);

            if old_state.pc < fees.len() {
                Some(SimulationInstruction {
                    program_index: old_state.pc,
                    fee: fees[old_state.pc],
                    program_input: all_information.get_input(old_state.pc),
                })
            } else {
                unimplemented!()
            }
        };

        simulation_test::<FibonacciVerifierProgram>(74, &mut test_generator);
    }
}
