use crate::dsl::modules::fiat_shamir::boundary_constraint_evaluation;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use fibonacci_example::FIB_LOG_SIZE;
use std::collections::HashMap;
use stwo_prover::core::fields::m31::M31;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<&str, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir_verify2_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir_verify2")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;
    let after_fiat_shamir_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("after_fiat_shamir")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `fiat_shamir_verify2`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir_verify2_hash,
        cache.get("fiat_shamir_verify2").unwrap(),
    )?;
    assert_eq!(res.len(), 13);

    let random_coeff_1_var = res[0];
    let trace_oods_values_vars = [res[1], res[2], res[3]];
    let composition_oods_raw_values_vars = [res[4], res[5], res[6], res[7]];
    let oods_point_x_var = res[8];
    let oods_point_y_var = res[9];
    let step_constraint_numerator_var = res[10];
    let composition_oods_value_var = res[11];
    let random_coeff_2_var = res[12];

    // Step 1: allocate the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: boundary_constraint_var
    let boundary_constraint_var = boundary_constraint_evaluation(
        &mut dsl,
        table,
        trace_oods_values_vars[0],
        oods_point_x_var,
        oods_point_y_var,
        FIB_LOG_SIZE,
        M31::from_u32_unchecked(443693538),
    )?;

    let random_coeff_1_limbs_var = dsl.execute("qm31_to_limbs", &[random_coeff_1_var])?[0];
    let step_constraint_numerator_limbs_var =
        dsl.execute("qm31_to_limbs", &[step_constraint_numerator_var])?[0];
    let step_constraint_numerator_mul_random_coeff1_var = dsl.execute(
        "qm31_limbs_mul",
        &[
            table,
            random_coeff_1_limbs_var,
            step_constraint_numerator_limbs_var,
        ],
    )?[0];

    let list_fiat_shamir_verify3 = [
        trace_oods_values_vars[0],
        trace_oods_values_vars[1],
        trace_oods_values_vars[2],
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
        oods_point_x_var,
        oods_point_y_var,
        step_constraint_numerator_mul_random_coeff1_var,
        boundary_constraint_var,
        composition_oods_value_var,
        random_coeff_2_var,
    ];

    let (pack_fiat_shamir_verify3_hash, pack_fiat_shamir_verify3) =
        zip_elements(&mut dsl, &list_fiat_shamir_verify3)?;

    cache.insert("fiat_shamir_verify3", pack_fiat_shamir_verify3);
    dsl.set_program_output("hash", pack_fiat_shamir_verify3_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
