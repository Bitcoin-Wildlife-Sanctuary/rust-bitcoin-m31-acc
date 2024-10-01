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

    let constant_results_cur_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("constant_results_{}", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unpack `constant_results_cur_hash`
    let res = unzip_elements(
        &mut dsl,
        constant_results_cur_hash,
        cache
            .get(&format!("constant_results_{}", query_idx))
            .unwrap(),
    )?;
    assert_eq!(res.len(), 13);

    let mut res = res.into_iter();

    let term_trace_to_constant_l = res.next().unwrap();
    let term_trace_to_constant_r = res.next().unwrap();
    let sum_num_interaction_logc_s_l = res.next().unwrap();
    let sum_num_interaction_logc_s_r = res.next().unwrap();

    let composition_queried_results = [res.next().unwrap(), res.next().unwrap()];

    let x_var = res.next().unwrap();
    let y_var = res.next().unwrap();
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
    let alpha_4_var = res[3];
    let _ = res[4];
    let _ = res[5];
    let _ = res[6];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the numerator composition
    // unzip `column_line_composition_hash`
    let res = unzip_elements(
        &mut dsl,
        column_line_composition_hash,
        cache.get("column_line_composition").unwrap(),
    )?;
    assert_eq!(res.len(), 8);

    let column_line_composition_vars = vec![
        (res[0], res[1]),
        (res[2], res[3]),
        (res[4], res[5]),
        (res[6], res[7]),
    ];

    let break_down_qm31 = |dsl: &mut DSL, v: usize| -> Result<[usize; 4]> {
        let first = dsl.execute("qm31_first", &[v])?[0];
        let second = dsl.execute("qm31_second", &[v])?[0];

        let first_real = dsl.execute("cm31_real", &[first])?[0];
        let first_imag = dsl.execute("cm31_imag", &[first])?[0];

        let second_real = dsl.execute("cm31_real", &[second])?[0];
        let second_imag = dsl.execute("cm31_imag", &[second])?[0];

        Ok([first_real, first_imag, second_real, second_imag])
    };

    let composition_vars = (
        break_down_qm31(&mut dsl, composition_queried_results[0])?,
        break_down_qm31(&mut dsl, composition_queried_results[1])?,
    );

    let mut numerator_composition = vec![];

    for i in 0..4 {
        numerator_composition.push(apply_twin(
            &mut dsl,
            table,
            y_var,
            composition_vars.0[i],
            composition_vars.1[i],
            column_line_composition_vars[i].0,
            column_line_composition_vars[i].1,
        )?);
    }

    let composition_1_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_composition[0].0])?[0];
    let alpha3_composition_1_l =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_3_var, composition_1_limbs_l)?;
    let composition_1_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_composition[0].1])?[0];
    let alpha3_composition_1_r =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_3_var, composition_1_limbs_r)?;

    let composition_2_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_composition[1].0])?[0];
    let alpha2_composition_2_l =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_2_var, composition_2_limbs_l)?;
    let composition_2_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_composition[1].1])?[0];
    let alpha2_composition_2_r =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_2_var, composition_2_limbs_r)?;

    let composition_3_limbs_l = dsl.execute("cm31_to_limbs", &[numerator_composition[2].0])?[0];
    let alpha_composition_3_l =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_var, composition_3_limbs_l)?;
    let composition_3_limbs_r = dsl.execute("cm31_to_limbs", &[numerator_composition[2].1])?[0];
    let alpha_composition_3_r =
        qm31_mul_cm31_limbs(&mut dsl, table, alpha_var, composition_3_limbs_r)?;

    let mut sum_num_composition_l = dsl.execute(
        "qm31_add",
        &[alpha3_composition_1_l, alpha2_composition_2_l],
    )?[0];
    sum_num_composition_l =
        dsl.execute("qm31_add", &[sum_num_composition_l, alpha_composition_3_l])?[0];
    sum_num_composition_l = dsl.execute(
        "qm31_add_cm31",
        &[sum_num_composition_l, numerator_composition[3].0],
    )?[0];

    let mut sum_num_composition_r = dsl.execute(
        "qm31_add",
        &[alpha3_composition_1_r, alpha2_composition_2_r],
    )?[0];
    sum_num_composition_r =
        dsl.execute("qm31_add", &[sum_num_composition_r, alpha_composition_3_r])?[0];
    sum_num_composition_r = dsl.execute(
        "qm31_add_cm31",
        &[sum_num_composition_r, numerator_composition[3].1],
    )?[0];

    let alpha_4_limbs = dsl.execute("qm31_to_limbs", &[alpha_4_var])?[0];
    let sum_num_composition_l_limbs = dsl.execute("qm31_to_limbs", &[sum_num_composition_l])?[0];
    let sum_num_composition_r_limbs = dsl.execute("qm31_to_limbs", &[sum_num_composition_r])?[0];

    let alpha4composition_l = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_4_limbs, sum_num_composition_l_limbs],
    )?[0];
    let alpha4composition_r = dsl.execute(
        "qm31_limbs_mul",
        &[table, alpha_4_limbs, sum_num_composition_r_limbs],
    )?[0];

    let term_trace_to_composition_l =
        dsl.execute("qm31_add", &[term_trace_to_constant_l, alpha4composition_l])?[0];
    let term_trace_to_composition_r =
        dsl.execute("qm31_add", &[term_trace_to_constant_r, alpha4composition_r])?[0];

    // unzip `prepared_oods_hash`
    let res = unzip_elements(
        &mut dsl,
        prepared_oods_hash,
        cache.get("prepared_oods").unwrap(),
    )?;
    assert_eq!(res.len(), 6);

    let _ = res[0];
    let _ = res[1];
    let prepared_oods_var = (res[2], res[3]);
    let prepared_oods_shifted_by_1_var = (res[4], res[5]);

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

    let mut list_num_composition_results_cur = vec![];
    list_num_composition_results_cur.push(term_trace_to_composition_l);
    list_num_composition_results_cur.push(term_trace_to_composition_r);
    list_num_composition_results_cur.push(sum_num_interaction_logc_s_l);
    list_num_composition_results_cur.push(sum_num_interaction_logc_s_r);
    list_num_composition_results_cur.push(x_var);
    list_num_composition_results_cur.push(y_var);
    list_num_composition_results_cur.push(prepared_oods_var.0);
    list_num_composition_results_cur.push(prepared_oods_var.1);
    list_num_composition_results_cur.push(prepared_oods_shifted_by_1_var.0);
    list_num_composition_results_cur.push(prepared_oods_shifted_by_1_var.1);
    list_num_composition_results_cur.push(fri_fold_random_coeff_var);
    list_num_composition_results_cur.push(expected_entry_quotient);

    let (pack_composition_results_cur_hash, pack_composition_results_cur) =
        zip_elements(&mut dsl, &list_num_composition_results_cur)?;

    cache.insert(
        format!("composition_results_{}", query_idx).to_string(),
        pack_composition_results_cur,
    );

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_composition_results_cur_hash)?;

    Ok(dsl)
}
