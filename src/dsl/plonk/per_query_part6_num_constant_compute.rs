use crate::dsl::building_blocks::qm31::qm31_mul_cm31_limbs;
use crate::dsl::modules::quotients::apply_twin;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use std::collections::HashMap;

pub fn generate_dsl(
    _: &Hints,
    cache: &mut HashMap<String, Zipper>,
    query_idx: usize,
) -> Result<DSL> {
    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let global_state_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("global_state").unwrap().hash.as_ref().to_vec()),
    )?;

    let interaction3_results_cur_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("interaction3_results_{}", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unpack `interaction3_results_cur_hash`
    let res = unzip_elements(
        &mut dsl,
        interaction3_results_cur_hash,
        cache
            .get(&format!("interaction3_results_{}", query_idx))
            .unwrap(),
    )?;
    assert_eq!(res.len(), 22);

    let mut res = res.into_iter();

    let term_trace_and_interaction_l = res.next().unwrap();
    let term_trace_and_interaction_r = res.next().unwrap();
    let sum_num_interaction_logc_s_l = res.next().unwrap();
    let sum_num_interaction_logc_s_r = res.next().unwrap();

    let mut constant_queried_results = vec![];
    for _ in 0..8 {
        constant_queried_results.push(res.next().unwrap());
    }

    let composition_queried_results = [res.next().unwrap(), res.next().unwrap()];

    let x_var = res.next().unwrap();
    let y_var = res.next().unwrap();
    let column_line_interaction_shifted_and_constant_hash = res.next().unwrap();
    let column_line_composition_hash = res.next().unwrap();
    let prepared_oods_hash = res.next().unwrap();
    let fri_fold_random_coeff_var = res.next().unwrap();
    let alphas_hash = res.next().unwrap();
    let expected_entry_quotient = res.next().unwrap();

    assert!(res.next().is_none());

    // unzip `alphas_hash`
    let res = unzip_elements(&mut dsl, alphas_hash, cache.get("alphas").unwrap())?;
    assert_eq!(res.len(), 7);
    // 1, 2, 3, 4, 8, 12, 20

    let alpha_var = res[0];
    let alpha_2_var = res[1];
    let alpha_3_var = res[2];
    let _ = res[3];
    let alpha_8_var = res[4];
    let _ = res[5];
    let _ = res[6];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the numerator constant
    // unzip `column_line_interaction_shifted_and_constant_hash`
    let res = unzip_elements(
        &mut dsl,
        column_line_interaction_shifted_and_constant_hash,
        cache
            .get("column_line_interaction_shifted_and_constant")
            .unwrap(),
    )?;
    assert_eq!(res.len(), 16);

    let column_line_interaction_shifted_and_constant_vars = vec![
        (res[0], res[1]),
        (res[2], res[3]),
        (res[4], res[5]),
        (res[6], res[7]),
        (res[8], res[9]),
        (res[10], res[11]),
        (res[12], res[13]),
        (res[14], res[15]),
    ];

    let mut numerator_constant = vec![];

    for i in 0..4 {
        numerator_constant.push(apply_twin(
            &mut dsl,
            table,
            y_var,
            constant_queried_results[i * 2],
            constant_queried_results[i * 2 + 1],
            column_line_interaction_shifted_and_constant_vars[i + 4].0,
            column_line_interaction_shifted_and_constant_vars[i + 4].1,
        )?);
    }

    let constant_1_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_constant[0].0])?[0];
    let alpha3_constant_1_l =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_3_var, constant_1_limbs_l)?;
    let constant_1_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_constant[0].1])?[0];
    let alpha3_constant_1_r =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_3_var, constant_1_limbs_r)?;

    let constant_2_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_constant[1].0])?[0];
    let alpha2_constant_2_l =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_2_var, constant_2_limbs_l)?;
    let constant_2_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_constant[1].1])?[0];
    let alpha2_constant_2_r =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_2_var, constant_2_limbs_r)?;

    let constant_3_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_constant[2].0])?[0];
    let alpha_constant_3_l = qm31_mul_cm31_limbs(&mut dsl, table, alpha_var, constant_3_limbs_l)?;
    let constant_3_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_constant[2].1])?[0];
    let alpha_constant_3_r = qm31_mul_cm31_limbs(&mut dsl, table, alpha_var, constant_3_limbs_r)?;

    let mut sum_num_constant_l =
        dsl.execute("qm31_add", &[alpha3_constant_1_l, alpha2_constant_2_l])?[0];
    sum_num_constant_l = dsl.execute("qm31_add", &[sum_num_constant_l, alpha_constant_3_l])?[0];
    sum_num_constant_l = dsl.execute(
        "qm31_add_cm31",
        &[sum_num_constant_l, numerator_constant[3].0],
    )?[0];

    let mut sum_num_constant_r =
        dsl.execute("qm31_add", &[alpha3_constant_1_r, alpha2_constant_2_r])?[0];
    sum_num_constant_r = dsl.execute("qm31_add", &[sum_num_constant_r, alpha_constant_3_r])?[0];
    sum_num_constant_r = dsl.execute(
        "qm31_add_cm31",
        &[sum_num_constant_r, numerator_constant[3].1],
    )?[0];

    let alpha_8_limbs = dsl.execute("qm31_to_limbs", &[alpha_8_var])?[0];
    let sum_num_constant_l_limbs = dsl.execute("qm31_to_limbs", &[sum_num_constant_l])?[0];
    let sum_num_constant_r_limbs = dsl.execute("qm31_to_limbs", &[sum_num_constant_r])?[0];

    let alpha8constant_l = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_8_limbs, sum_num_constant_l_limbs],
    )?[0];
    let alpha8constant_r = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_8_limbs, sum_num_constant_r_limbs],
    )?[0];

    let term_trace_to_constant_l = dsl.execute(
        "qm31_add",
        &[term_trace_and_interaction_l, alpha8constant_l],
    )?[0];
    let term_trace_to_constant_r = dsl.execute(
        "qm31_add",
        &[term_trace_and_interaction_r, alpha8constant_r],
    )?[0];

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

    let mut list_num_constant_results_cur = vec![];
    list_num_constant_results_cur.push(term_trace_to_constant_l);
    list_num_constant_results_cur.push(term_trace_to_constant_r);
    list_num_constant_results_cur.push(sum_num_interaction_logc_s_l);
    list_num_constant_results_cur.push(sum_num_interaction_logc_s_r);
    list_num_constant_results_cur.extend_from_slice(&composition_queried_results);
    list_num_constant_results_cur.push(x_var);
    list_num_constant_results_cur.push(y_var);
    list_num_constant_results_cur.push(column_line_composition_hash);
    list_num_constant_results_cur.push(prepared_oods_hash);
    list_num_constant_results_cur.push(fri_fold_random_coeff_var);
    list_num_constant_results_cur.push(alphas_hash);
    list_num_constant_results_cur.push(expected_entry_quotient);

    let (pack_constant_results_cur_hash, pack_constant_results_cur) =
        zip_elements(&mut dsl, &list_num_constant_results_cur)?;

    cache.insert(
        format!("constant_results_{}", query_idx).to_string(),
        pack_constant_results_cur,
    );

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_constant_results_cur_hash)?;

    Ok(dsl)
}
