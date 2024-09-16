use crate::dsl::building_blocks::point::get_random_point_full;
use crate::dsl::modules::fiat_shamir::{
    coset_vanishing, step_constraint_denominator_inverse_evaluation,
};
use crate::dsl::plonk::hints::{Hints, LOG_N_ROWS};
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use std::collections::HashMap;
use stwo_prover::core::poly::circle::CanonicCoset;

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

    let constraint_logup_c_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get("constraint_logup_c_result")
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `constraint_logup_c_result`
    let res = unzip_elements(
        &mut dsl,
        constraint_logup_c_result_hash,
        cache.get("constraint_logup_c_result").unwrap(),
    )?;
    assert_eq!(res.len(), 3);

    let constraint_logup_ab_without_randomizer_var = res[0];
    let constraint_algebra_plus_logup_c_var = res[1];
    let before_oods_channel_var = res[2];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the OODS point
    let (_, oods_point_x_var, oods_point_y_var) =
        get_random_point_full(&mut dsl, table, before_oods_channel_var)?;

    // Step 3: compute the coset vanishing
    let coset_vanishing_var = {
        let constraint_zero_domain = CanonicCoset::new(LOG_N_ROWS).coset;

        let denominator = coset_vanishing(
            &mut dsl,
            table,
            oods_point_x_var,
            oods_point_y_var,
            constraint_zero_domain,
        )?;

        let denominator_limbs = dsl.execute("qm31_to_limbs", &[denominator])?[0];
        let denominator_limbs_inverse =
            dsl.execute("qm31_limbs_inverse", &[table, denominator_limbs])?[0];

        denominator_limbs_inverse
    };

    let list_oods_coset_vanishing_result = [
        constraint_logup_ab_without_randomizer_var,
        constraint_algebra_plus_logup_c_var,
        coset_vanishing_var,
        oods_point_x_var,
        oods_point_y_var,
    ];
    let (
        pack_constraint_oods_coset_vanishing_result_hash,
        pack_constraint_oods_coset_vanishing_result,
    ) = zip_elements(&mut dsl, &list_oods_coset_vanishing_result)?;

    cache.insert(
        "constraint_oods_coset_vanishing_result".to_string(),
        pack_constraint_oods_coset_vanishing_result,
    );

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", fiat_shamir2_result_hash)?;
    dsl.set_program_output("hash", pack_constraint_oods_coset_vanishing_result_hash)?;

    Ok(dsl)
}
