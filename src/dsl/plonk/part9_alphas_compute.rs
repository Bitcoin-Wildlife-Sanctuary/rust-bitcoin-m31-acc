use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
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

    let prepared_oods_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("prepared_oods").unwrap().hash.as_ref().to_vec()),
    )?;

    // The computation will look as follows
    //   (alpha^20) (alpha^3 * g_mul(X) + alpha^2 * g_a_val(X) + alpha * g_b_val(X) + g_c_val(X))
    // + (alpha^8) (alpha^11 * g_logab1(X) + alpha^10 * g_logab2(X) + alpha^9 * g_logab3(X) + alpha^8 * g_logab4(X)
    //             + alpha^7 * g_logc1(X) + alpha^6 * g_logc2(X) + alpha^5 * g_logc3(X) + alpha^4 * g_logc4(X))
    // + (alpha^4) (alpha^3 * g_op(X) + alpha^2 * g_a_wire(X) + alpha * g_b_wire(X) + g_c_wire(X))
    // + (alpha^3 * g_compose1(X) + alpha^2 * g_compose2(X) + alpha * g_compose3(X) + g_compose4(X))
    //
    // divided by v_0(X)
    //
    // plus
    //
    // (alpha^8) (alpha^3 * g_logc_shifted_1(X) + alpha^2 * g_logc_shifted_2(X) + alpha * g_logc_shifted_3(X) + g_logc_shifted_4(X))
    //
    // divided by v_1(X)

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
    let _ = res.next().unwrap();
    let _ = res.next().unwrap();
    let line_batch_random_coeff_var = res.next().unwrap();
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

    // Step 2: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // The needed alphas are:
    // - alpha, alpha^2, alpha^3, alpha^4, alpha^8, alpha^12, alpha^20

    // note: alpha is line_batch_random_coeff_var
    // not the alpha_var that was actually the alpha for logUp argument

    // Step 3: compute the limbs of alpha
    let alpha_limbs_var = dsl.execute("qm31_to_limbs", &[line_batch_random_coeff_var])?[0];
    let alpha_2_var = dsl.execute("qm31_limbs_mul", &[table, alpha_limbs_var, alpha_limbs_var])?[0];
    let alpha_2_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_2_var])?[0];
    let alpha_3_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_limbs_var, alpha_2_limbs_var],
    )?[0];
    let alpha_4_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_2_limbs_var, alpha_2_limbs_var],
    )?[0];
    let alpha_4_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_4_var])?[0];
    let alpha_8_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_4_limbs_var, alpha_4_limbs_var],
    )?[0];
    let alpha_8_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_8_var])?[0];
    let alpha_12_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_8_limbs_var, alpha_4_limbs_var],
    )?[0];
    let alpha_12_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_12_var])?[0];
    let alpha_20_var = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_12_limbs_var, alpha_8_limbs_var],
    )?[0];

    let list_alphas = vec![
        line_batch_random_coeff_var,
        alpha_2_var,
        alpha_3_var,
        alpha_4_var,
        alpha_8_var,
        alpha_12_var,
        alpha_20_var,
    ];
    let (pack_alphas_hash, pack_alphas) = zip_elements(&mut dsl, &list_alphas)?;

    cache.insert("alphas".to_string(), pack_alphas);

    dsl.set_program_output("hash", fiat_shamir1_result_hash)?;
    dsl.set_program_output("hash", fiat_shamir2_result_hash)?;
    dsl.set_program_output("hash", prepared_oods_hash)?;
    dsl.set_program_output("hash", pack_alphas_hash)?;

    Ok(dsl)
}
