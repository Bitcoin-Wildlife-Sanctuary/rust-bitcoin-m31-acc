use crate::algorithms::utils::OP_HINT;
use crate::dsl::plonk::hints::Hints;
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

pub struct PlonkVerifierProgram {}

#[derive(Clone)]
pub struct PlonkVerifierInput {
    pub stack: Witness,
    pub hints: Witness,
}

impl From<PlonkVerifierInput> for Script {
    fn from(input: PlonkVerifierInput) -> Script {
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

/// The state of the Plonk split program.
#[derive(Clone, Debug)]
pub struct PlonkVerifierState {
    /// The program counter.
    pub pc: usize,
    /// The hash of the stack.
    pub stack_hash: Vec<u8>,
    /// The stack from the execution.
    pub stack: Vec<Vec<u8>>,
}

impl From<PlonkVerifierState> for Script {
    fn from(v: PlonkVerifierState) -> Self {
        script! {
            { v.pc }
            { v.stack_hash }
        }
    }
}

pub struct PlonkAllInformation {
    pub scripts: Vec<Script>,
    pub witnesses: Vec<Witness>,
    pub outputs: Vec<Witness>,
}

pub static PLONK_ALL_INFORMATION: OnceLock<PlonkAllInformation> = OnceLock::new();

impl PlonkAllInformation {
    pub fn get_input(&self, idx: usize) -> PlonkVerifierInput {
        PlonkVerifierInput {
            stack: if idx == 0 {
                vec![]
            } else {
                self.outputs[idx - 1].clone()
            },
            hints: self.witnesses[idx].clone(),
        }
    }
}

pub fn compute_all_information() -> PlonkAllInformation {
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
        super::part1_fiat_shamir1_noncompute::generate_dsl,
        super::part2_fiat_shamir2_noncompute::generate_dsl,
        super::part3_constraint_algebra_logup_ab_compute::generate_dsl,
        super::part4_constraint_logup_c_compute::generate_dsl,
        super::part5_constraint_oods_coset_vanishing_compute::generate_dsl,
        super::part6_constraint_equalverify_compute::generate_dsl,
        super::part7_constraint_oods_shifted_and_prepared_compute::generate_dsl,
        super::part8_alphas_compute::generate_dsl,
        super::part9_column_line_coeffs1_compute::generate_dsl,
        super::part10_column_line_coeffs2_compute::generate_dsl,
        super::part11_column_line_coeffs3_compute::generate_dsl,
        super::part12_column_line_coeffs4_compute::generate_dsl,
        super::part13_sort_queries1_noncompute::generate_dsl,
        super::part14_sort_queries2_noncompute::generate_dsl,
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
            super::per_query_part1_folding_compute::generate_dsl,
            super::per_query_part2_num_trace_compute::generate_dsl,
            super::per_query_part3_num_interaction1_compute::generate_dsl,
            super::per_query_part4_num_interaction2_compute::generate_dsl,
            super::per_query_part5_num_interaction3_compute::generate_dsl,
            super::per_query_part6_num_constant_compute::generate_dsl,
            super::per_query_part7_num_composition_compute::generate_dsl,
            super::per_query_part8_denom_and_equalverify_compute::generate_dsl,
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
            { cache.get("fiat_shamir2").unwrap().hash }
            { cache.get("fiat_shamir1_result").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
            { cache.get("constraint_logup_ab_result").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
            { cache.get("constraint_logup_c_result").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
            { cache.get("constraint_oods_coset_vanishing_result").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
            { cache.get("constraint_oods_point").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
            { cache.get("prepared_oods").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("fiat_shamir1_result").unwrap().hash }
            { cache.get("fiat_shamir2_result").unwrap().hash }
            { cache.get("prepared_oods").unwrap().hash }
            { cache.get("alphas").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("column_line_coeffs1").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("column_line_coeffs2").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("column_line_coeffs3").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("shared_information").unwrap().hash }
            { cache.get("sort_queries1").unwrap().hash }
        })
        .unwrap(),
    );
    outputs.push(
        convert_to_witness(script! {
            { cache.get("shared_information").unwrap().hash }
            { cache.get("sort_queries1").unwrap().hash }
            { cache.get("folding_intermediate_1").unwrap().hash }
            { cache.get("folding_intermediate_2").unwrap().hash }
            { cache.get("folding_intermediate_3").unwrap().hash }
            { cache.get("folding_intermediate_4").unwrap().hash }
            { cache.get("folding_intermediate_5").unwrap().hash }
            { cache.get("folding_intermediate_6").unwrap().hash }
            { cache.get("folding_intermediate_7").unwrap().hash }
            { cache.get("folding_intermediate_8").unwrap().hash }
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
                { cache.get(&format!("query_post_folding_{}", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("trace_results_{}", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("interaction1_results_{}", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("interaction2_results_{}", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("interaction3_results_{}", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("constant_results_{}", query_idx)).unwrap().hash }
            })
            .unwrap(),
        );
        outputs.push(
            convert_to_witness(script! {
                { cache.get("global_state").unwrap().hash }
                { cache.get(&format!("composition_results_{}", query_idx)).unwrap().hash }
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

    PlonkAllInformation {
        scripts,
        witnesses,
        outputs,
    }
}

impl CovenantProgram for PlonkVerifierProgram {
    type State = PlonkVerifierState;
    type Input = PlonkVerifierInput;
    const CACHE_NAME: &'static str = "PLONK";

    fn new() -> Self::State {
        PlonkVerifierState {
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
        let all_information = PLONK_ALL_INFORMATION.get_or_init(compute_all_information);

        let mut map = BTreeMap::new();

        let mut output_stack_size = vec![2, 2, 3, 3, 3, 3, 3, 4, 1, 1, 1, 2, 10, 1];

        for _ in 0..8 {
            output_stack_size.extend_from_slice(&[2, 2, 2, 2, 2, 2, 2, 1]);
        }

        for script_idx in 0..(14 + 8 * 8) {
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
        let all_information = PLONK_ALL_INFORMATION.get_or_init(compute_all_information);

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
    use crate::dsl::plonk::covenant::{
        compute_all_information, PlonkVerifierProgram, PlonkVerifierState, PLONK_ALL_INFORMATION,
    };
    use covenants_gadgets::test::{simulation_test, SimulationInstruction};

    #[test]
    fn test_integration() {
        // The integration assumes a fee rate of 7 sat/vByte.
        // Note that in many situations, the fee rate is only 2 sat/vByte.

        let mut fees = vec![
            50211, 48559, 117929, 38115, 107898, 35238, 42609, 75901, 46389, 69587, 75880, 43407,
            66556, 41447,
        ];

        for _ in 0..8 {
            fees.extend_from_slice(&[95865, 88641, 89467, 90027, 65905, 89159, 89019, 72933]);
        }

        println!(
            "total fee assuming 7 sat/vByte: {}",
            fees.iter().sum::<usize>()
        );

        let mut test_generator = |old_state: &PlonkVerifierState| {
            let all_information = PLONK_ALL_INFORMATION.get_or_init(compute_all_information);

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

        simulation_test::<PlonkVerifierProgram>(74, &mut test_generator);
    }
}
