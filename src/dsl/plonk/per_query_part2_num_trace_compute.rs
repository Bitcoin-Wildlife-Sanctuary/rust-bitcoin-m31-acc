use crate::dsl::building_blocks::qm31::qm31_mul_cm31_limbs;
use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::modules::quotients::apply_twin;
use crate::dsl::plonk::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
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

    let query_post_folding_cur_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query_post_folding_{}", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip `query_post_folding_cur_hash`
    let res = unzip_elements(
        &mut dsl,
        query_post_folding_cur_hash,
        cache
            .get(&format!("query_post_folding_{}", query_idx))
            .unwrap(),
    )?;
    assert_eq!(res.len(), 32);

    let mut res = res.into_iter();

    let mut trace_queried_results = vec![];
    let mut interaction_queried_results = vec![];
    let mut constant_queried_results = vec![];
    let mut composition_queried_results = vec![];

    for _ in 0..8 {
        trace_queried_results.push(res.next().unwrap());
    }
    for _ in 0..4 {
        interaction_queried_results.push(res.next().unwrap());
    }
    for _ in 0..8 {
        constant_queried_results.push(res.next().unwrap());
    }
    for _ in 0..2 {
        composition_queried_results.push(res.next().unwrap());
    }
    let x_var = res.next().unwrap();
    let y_var = res.next().unwrap();
    let column_line_trace_hash = res.next().unwrap();
    let column_line_interaction_hash = res.next().unwrap();
    let column_line_interaction_shifted_and_constant_hash = res.next().unwrap();
    let column_line_composition_hash = res.next().unwrap();
    let prepared_oods_hash = res.next().unwrap();
    let fri_fold_random_coeff_var = res.next().unwrap();
    let alphas_hash = res.next().unwrap();
    let expected_entry_quotient = res.next().unwrap();

    assert!(res.next().is_none());

    // unzip `column_line_trace_hash`
    let res = unzip_elements(
        &mut dsl,
        column_line_trace_hash,
        cache.get("column_line_trace").unwrap(),
    )?;
    assert_eq!(res.len(), 8);

    let column_line_trace_vars = vec![
        (res[0], res[1]),
        (res[2], res[3]),
        (res[4], res[5]),
        (res[6], res[7]),
    ];

    // unzip `alphas_hash`
    let res = unzip_elements(&mut dsl, alphas_hash, cache.get("alphas").unwrap())?;
    assert_eq!(res.len(), 7);
    // 1, 2, 3, 4, 8, 12, 20

    let alpha_var = res[0];
    let alpha_2_var = res[1];
    let alpha_3_var = res[2];
    let _ = res[3];
    let _ = res[4];
    let _ = res[5];
    let alpha_20_var = res[6];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the numerator trace
    let numerator_trace_mult = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results[0],
        trace_queried_results[1],
        column_line_trace_vars[0].0,
        column_line_trace_vars[0].1,
    )?;

    let numerator_trace_a_val = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results[2],
        trace_queried_results[3],
        column_line_trace_vars[1].0,
        column_line_trace_vars[1].1,
    )?;

    let numerator_trace_b_val = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results[4],
        trace_queried_results[5],
        column_line_trace_vars[2].0,
        column_line_trace_vars[2].1,
    )?;

    let numerator_trace_c_val = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results[6],
        trace_queried_results[7],
        column_line_trace_vars[3].0,
        column_line_trace_vars[3].1,
    )?;

    let mult_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_trace_mult.0])?[0];
    let alpha3_mult_l = qm31_mul_cm31_limbs(&mut dsl, table, alpha_3_var, mult_limbs_l)?;
    let mult_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_trace_mult.1])?[0];
    let alpha3_mult_r = qm31_mul_cm31_limbs(&mut dsl, table, alpha_3_var, mult_limbs_r)?;

    let a_val_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_trace_a_val.0])?[0];
    let alpha2_a_val_l = qm31_mul_cm31_limbs(&mut dsl, table, alpha_2_var, a_val_limbs_l)?;
    let a_val_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_trace_a_val.1])?[0];
    let alpha2_a_val_r = qm31_mul_cm31_limbs(&mut dsl, table, alpha_2_var, a_val_limbs_r)?;

    let b_val_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_trace_b_val.0])?[0];
    let alpha_b_val_l = qm31_mul_cm31_limbs(&mut dsl, table, alpha_var, b_val_limbs_l)?;
    let b_val_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_trace_b_val.1])?[0];
    let alpha_b_val_r = qm31_mul_cm31_limbs(&mut dsl, table, alpha_var, b_val_limbs_r)?;

    let mut sum_num_trace_l = dsl.execute("qm31_add", &[alpha3_mult_l, alpha2_a_val_l])?[0];
    sum_num_trace_l = dsl.execute("qm31_add", &[sum_num_trace_l, alpha_b_val_l])?[0];
    sum_num_trace_l = dsl.execute("qm31_add_cm31", &[sum_num_trace_l, numerator_trace_c_val.0])?[0];

    let mut sum_num_trace_r = dsl.execute("qm31_add", &[alpha3_mult_r, alpha2_a_val_r])?[0];
    sum_num_trace_r = dsl.execute("qm31_add", &[sum_num_trace_r, alpha_b_val_r])?[0];
    sum_num_trace_r = dsl.execute("qm31_add_cm31", &[sum_num_trace_r, numerator_trace_c_val.1])?[0];

    let alpha_20_limbs_var = dsl.execute("qm31_to_limbs", &[alpha_20_var])?[0];
    let sum_num_trace_l_limbs = dsl.execute("qm31_to_limbs", &[sum_num_trace_l])?[0];
    let sum_num_trace_r_limbs = dsl.execute("qm31_to_limbs", &[sum_num_trace_r])?[0];

    let alpha20trace_l = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_20_limbs_var, sum_num_trace_l_limbs],
    )?[0];
    let alpha20trace_r = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_20_limbs_var, sum_num_trace_r_limbs],
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

    let mut list_num_trace_results_cur = vec![];
    list_num_trace_results_cur.push(alpha20trace_l);
    list_num_trace_results_cur.push(alpha20trace_r);
    list_num_trace_results_cur.extend_from_slice(&interaction_queried_results);
    list_num_trace_results_cur.extend_from_slice(&constant_queried_results);
    list_num_trace_results_cur.extend_from_slice(&composition_queried_results);
    list_num_trace_results_cur.push(x_var);
    list_num_trace_results_cur.push(y_var);
    list_num_trace_results_cur.push(column_line_interaction_hash);
    list_num_trace_results_cur.push(column_line_interaction_shifted_and_constant_hash);
    list_num_trace_results_cur.push(column_line_composition_hash);
    list_num_trace_results_cur.push(prepared_oods_hash);
    list_num_trace_results_cur.push(fri_fold_random_coeff_var);
    list_num_trace_results_cur.push(alphas_hash);
    list_num_trace_results_cur.push(expected_entry_quotient);

    let (pack_trace_results_cur_hash, pack_trace_results_cur) =
        zip_elements(&mut dsl, &list_num_trace_results_cur)?;

    cache.insert(
        format!("trace_results_{}", query_idx).to_string(),
        pack_trace_results_cur,
    );

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_trace_results_cur_hash)?;

    Ok(dsl)
}
