use crate::dsl::modules::fiat_shamir::step_constraint_denominator_inverse_evaluation;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use fibonacci_example::FIB_LOG_SIZE;
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();
    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir_verify3_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir_verify3")
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

    // unzip `fiat_shamir_verify3`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir_verify3_hash,
        cache.get("fiat_shamir_verify3").unwrap(),
    )?;
    assert_eq!(res.len(), 13);

    let trace_oods_values_vars = [res[0], res[1], res[2]];
    let composition_oods_raw_values_vars = [res[3], res[4], res[5], res[6]];
    let oods_point_x_var = res[7];
    let oods_point_y_var = res[8];
    let step_constraint_numerator_mul_random_coeff1_var = res[9];
    let boundary_constraint_var = res[10];
    let composition_oods_value_var = res[11];
    let random_coeff_2_var = res[12];

    // Step 1: allocate the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: step_constraint_denominator_evaluation
    let step_constraint_denominator_inverse_limbs_var =
        step_constraint_denominator_inverse_evaluation(
            &mut dsl,
            table,
            oods_point_x_var,
            oods_point_y_var,
            FIB_LOG_SIZE,
        )?;

    // Step 3: compute step_constraint_evaluation
    let step_constraint_numerator_mul_random_coeff1_limbs_var = dsl.execute(
        "qm31_to_limbs",
        &[step_constraint_numerator_mul_random_coeff1_var],
    )?[0];
    let step_constraint_var = dsl.execute(
        "qm31_limbs_mul",
        &[
            table,
            step_constraint_numerator_mul_random_coeff1_limbs_var,
            step_constraint_denominator_inverse_limbs_var,
        ],
    )?[0];

    // Step 4: check equality
    let computed_composition_oods_value_var =
        dsl.execute("qm31_add", &[step_constraint_var, boundary_constraint_var])?[0];
    let _ = dsl.execute(
        "qm31_equalverify",
        &[
            computed_composition_oods_value_var,
            composition_oods_value_var,
        ],
    )?;

    let list_fiat_shamir_verify4 = [
        trace_oods_values_vars[0],
        trace_oods_values_vars[1],
        trace_oods_values_vars[2],
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
        oods_point_x_var,
        oods_point_y_var,
        random_coeff_2_var,
    ];

    let (pack_fiat_shamir_verify4_hash, pack_fiat_shamir_verify4) =
        zip_elements(&mut dsl, &list_fiat_shamir_verify4)?;

    cache.insert("fiat_shamir_verify4".to_string(), pack_fiat_shamir_verify4);
    dsl.set_program_output("hash", pack_fiat_shamir_verify4_hash)?;
    dsl.set_program_output("hash", after_fiat_shamir_hash)?;

    Ok(dsl)
}
