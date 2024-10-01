use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::modules::fiat_shamir::eval_from_partial_evals;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use std::collections::HashMap;

pub fn generate_dsl(_: &Hints, cache: &mut HashMap<String, Zipper>) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let fiat_shamir1_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir1_result")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    let fiat_shamir2_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("fiat_shamir2_result")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    let constraint_oods_coset_vanishing_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("constraint_oods_coset_vanishing_result")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `fiat_shamir1_result_hash`
    let res = unzip_elements(
        &mut dsl,
        fiat_shamir1_result_hash,
        cache.get("fiat_shamir1_result").unwrap(),
    )?;
    assert_eq!(res.len(), 32);

    let mut res = res.into_iter();

    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let composition_fold_random_coeff_var = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();

    let mut trace_oods_values_vars = vec![];
    let mut interaction_oods_values_vars = vec![];
    let mut constant_oods_values_vars = vec![];
    let mut composition_oods_raw_values_vars = vec![];

    for _ in 0..4 {
        trace_oods_values_vars.push(res.next().unwrap());
    }
    for _ in 0..12 {
        interaction_oods_values_vars.push(res.next().unwrap());
    }
    for _ in 0..4 {
        constant_oods_values_vars.push(res.next().unwrap());
    }
    for _ in 0..4 {
        composition_oods_raw_values_vars.push(res.next().unwrap());
    }

    assert!(res.next().is_none());

    // unzip `constraint_oods_coset_vanishing_result_hash`
    let res = unzip_elements(
        &mut dsl,
        constraint_oods_coset_vanishing_result_hash,
        cache.get("constraint_oods_coset_vanishing_result").unwrap(),
    )?;
    assert_eq!(res.len(), 5);

    let constraint_logup_ab_without_randomizer_var = res[0];
    let constraint_algebra_plus_logup_c_var = res[1];
    let coset_vanishing_var = res[2];
    let oods_point_x_var = res[3];
    let oods_point_y_var = res[4];

    // Step 1: convert alpha_var and constraint_logup_ab_without_randomizer_var to limbs
    let composition_fold_random_coeff_limbs_var =
        dsl.execute("qm31_to_limbs", &[composition_fold_random_coeff_var])?[0];
    let constraint_logup_ab_without_randomizer_limbs_var = dsl.execute(
        "qm31_to_limbs",
        &[constraint_logup_ab_without_randomizer_var],
    )?[0];

    // Step 2: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 3: compute constraint_logup_ab_var
    let constraint_logup_ab_var = dsl.execute(
        "qm31_limbs_mul",
        &[
            table,
            composition_fold_random_coeff_limbs_var,
            constraint_logup_ab_without_randomizer_limbs_var,
        ],
    )?[0];

    // Step 4: compute the expected numerator
    let sum_num_var = dsl.execute(
        "qm31_add",
        &[constraint_logup_ab_var, constraint_algebra_plus_logup_c_var],
    )?[0];

    // Step 5: multiply with the coset vanishing inverse
    let sum_num_limbs_var = dsl.execute("qm31_to_limbs", &[sum_num_var])?[0];
    let computed_result_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, sum_num_limbs_var, coset_vanishing_var],
    )?[0];

    // Step 6: reconstruct the composition_oods_value_var
    let composition_oods_value_var = eval_from_partial_evals(
        &mut dsl,
        composition_oods_raw_values_vars[0],
        composition_oods_raw_values_vars[1],
        composition_oods_raw_values_vars[2],
        composition_oods_raw_values_vars[3],
    )?;

    let _ = dsl.execute(
        "qm31_equalverify",
        &[computed_result_var, composition_oods_value_var],
    )?;

    let list_oods_point = vec![oods_point_x_var, oods_point_y_var];
    let (pack_constraint_oods_point_hash, pack_constraint_oods_point) =
        zip_elements(&mut dsl, &list_oods_point)?;

    cache.insert(
        "constraint_oods_point".to_string(),
        pack_constraint_oods_point,
    );

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", fiat_shamir2_result_hash)?;
    dsl.set_program_output("hash", pack_constraint_oods_point_hash)?;

    Ok(dsl)
}
