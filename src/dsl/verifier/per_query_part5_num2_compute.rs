use crate::dsl::building_blocks::qm31::qm31_mul_cm31_limbs;
use crate::dsl::modules::quotients::apply_twin;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
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

    let cur_num_denom1_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_num_denom1_result", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    let cur_numerator_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_num2", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip the cur_numerator_hash
    let res = unzip_elements(
        &mut dsl,
        cur_numerator_hash,
        cache
            .get(format!("query{}_num2", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 13);

    let y_var = res[0];
    let composition_queried_results = (res[1], res[2]);
    let column_line_coeff_composition_var = [
        (res[3], res[4]),
        (res[5], res[6]),
        (res[7], res[8]),
        (res[9], res[10]),
    ];
    let alpha6_var = res[11];
    let pack_cur_aggregation2_hash = res[12];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: compute the numerators for composition
    let composition_queried_results =
        [composition_queried_results.0, composition_queried_results.1]
            .iter()
            .map(|&cur| {
                let first = dsl.execute("qm31_first", &[cur])?[0];
                let second = dsl.execute("qm31_second", &[cur])?[0];

                let first_real = dsl.execute("cm31_real", &[first])?[0];
                let first_imag = dsl.execute("cm31_imag", &[first])?[0];
                let second_real = dsl.execute("cm31_real", &[second])?[0];
                let second_imag = dsl.execute("cm31_imag", &[second])?[0];

                Ok([first_real, first_imag, second_real, second_imag])
            })
            .collect::<Result<Vec<_>>>()?;

    let numerator_composition_0 = apply_twin(
        &mut dsl,
        table,
        y_var,
        composition_queried_results[0][0],
        composition_queried_results[1][0],
        column_line_coeff_composition_var[0].0,
        column_line_coeff_composition_var[0].1,
    )?;
    let numerator_composition_1 = apply_twin(
        &mut dsl,
        table,
        y_var,
        composition_queried_results[0][1],
        composition_queried_results[1][1],
        column_line_coeff_composition_var[1].0,
        column_line_coeff_composition_var[1].1,
    )?;
    let numerator_composition_2 = apply_twin(
        &mut dsl,
        table,
        y_var,
        composition_queried_results[0][2],
        composition_queried_results[1][2],
        column_line_coeff_composition_var[2].0,
        column_line_coeff_composition_var[2].1,
    )?;
    let numerator_composition_3 = apply_twin(
        &mut dsl,
        table,
        y_var,
        composition_queried_results[0][3],
        composition_queried_results[1][3],
        column_line_coeff_composition_var[3].0,
        column_line_coeff_composition_var[3].1,
    )?;

    // unzip the cur_num_denom1_result_hash
    let res = unzip_elements(
        &mut dsl,
        cur_num_denom1_result_hash,
        cache
            .get(format!("query{}_num_denom1_result", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 14);

    let denominator_inverses_oods_shifteds_by_0 = (res[0], res[1]);
    let denominator_inverses_oods_shifteds_by_1 = (res[2], res[3]);
    let denominator_inverses_oods_shifteds_by_2 = (res[4], res[5]);
    let denominator_inverses_oods = (res[6], res[7]);

    let numerator_trace_0 = (res[8], res[9]);
    let numerator_trace_1 = (res[10], res[11]);
    let numerator_trace_2 = (res[12], res[13]);

    let u1_limbs = denominator_inverses_oods_shifteds_by_0.0;
    let a1_limbs = dsl.execute("cm31_to_limbs", &[numerator_trace_0.0])?[0];
    let u1a1 = dsl.execute("cm31_limbs_mul", &[table, u1_limbs, a1_limbs])?[0];

    let u2_limbs = denominator_inverses_oods_shifteds_by_1.0;
    let a2_limbs = dsl.execute("cm31_to_limbs", &[numerator_trace_1.0])?[0];
    let u2a2 = dsl.execute("cm31_limbs_mul", &[table, u2_limbs, a2_limbs])?[0];

    let u3_limbs = denominator_inverses_oods_shifteds_by_2.0;
    let a3_limbs = dsl.execute("cm31_to_limbs", &[numerator_trace_2.0])?[0];
    let u3a3 = dsl.execute("cm31_limbs_mul", &[table, u3_limbs, a3_limbs])?[0];

    let v1_limbs = denominator_inverses_oods_shifteds_by_0.1;
    let b1_limbs = dsl.execute("cm31_to_limbs", &[numerator_trace_0.1])?[0];
    let v1b1 = dsl.execute("cm31_limbs_mul", &[table, v1_limbs, b1_limbs])?[0];

    let v2_limbs = denominator_inverses_oods_shifteds_by_1.1;
    let b2_limbs = dsl.execute("cm31_to_limbs", &[numerator_trace_1.1])?[0];
    let v2b2 = dsl.execute("cm31_limbs_mul", &[table, v2_limbs, b2_limbs])?[0];

    let v3_limbs = denominator_inverses_oods_shifteds_by_2.1;
    let b3_limbs = dsl.execute("cm31_to_limbs", &[numerator_trace_2.1])?[0];
    let v3b3 = dsl.execute("cm31_limbs_mul", &[table, v3_limbs, b3_limbs])?[0];

    let u1a1_limbs = dsl.execute("cm31_to_limbs", &[u1a1])?[0];
    let alpha6u1a1 = qm31_mul_cm31_limbs(&mut dsl, table, alpha6_var, u1a1_limbs)?;

    let mut list_aggregation1_result = vec![];
    list_aggregation1_result.push(y_var);
    list_aggregation1_result.push(u2a2);
    list_aggregation1_result.push(u3a3);
    list_aggregation1_result.push(v1b1);
    list_aggregation1_result.push(v2b2);
    list_aggregation1_result.push(v3b3);
    list_aggregation1_result.push(alpha6u1a1);
    list_aggregation1_result.push(denominator_inverses_oods.0);
    list_aggregation1_result.push(denominator_inverses_oods.1);
    list_aggregation1_result.push(numerator_composition_0.0);
    list_aggregation1_result.push(numerator_composition_0.1);
    list_aggregation1_result.push(numerator_composition_1.0);
    list_aggregation1_result.push(numerator_composition_1.1);
    list_aggregation1_result.push(numerator_composition_2.0);
    list_aggregation1_result.push(numerator_composition_2.1);
    list_aggregation1_result.push(numerator_composition_3.0);
    list_aggregation1_result.push(numerator_composition_3.1);

    // Step 4: store the denominator inverses related variables into a pack
    let (pack_cur_aggregation1_result_hash, pack_cur_aggregation1_result) =
        zip_elements(&mut dsl, &list_aggregation1_result)?;

    let name = format!("query{}_aggregation1_result", query_idx);
    cache.insert(name, pack_cur_aggregation1_result);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_aggregation1_result_hash)?;
    dsl.set_program_output("hash", pack_cur_aggregation2_hash)?;

    Ok(dsl)
}
