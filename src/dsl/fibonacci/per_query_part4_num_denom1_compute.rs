use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::modules::quotients::{apply_twin, denominator_inverse_limbs_from_prepared};
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

    let cur_num_denom1_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_num_denom1", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip the cur_denominator_inverses_hash
    let res = unzip_elements(
        &mut dsl,
        cur_num_denom1_hash,
        cache
            .get(format!("query{}_num_denom1", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 19);

    let prepared_oods_shifted_by_0_var = (res[0], res[1]);
    let prepared_oods_shifted_by_1_var = (res[2], res[3]);
    let prepared_oods_shifted_by_2_var = (res[4], res[5]);
    let prepared_oods_var = (res[6], res[7]);
    let x_var = res[8];
    let y_var = res[9];
    let trace_queried_results = (res[10], res[11]);
    let column_line_coeff_trace_0_var = (res[12], res[13]);
    let column_line_coeff_trace_1_var = (res[14], res[15]);
    let column_line_coeff_trace_2_var = (res[16], res[17]);
    let pack_cur_num2_hash = res[18];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    let mut denominator_inverses = vec![];
    denominator_inverses.push(denominator_inverse_limbs_from_prepared(
        &mut dsl,
        table,
        prepared_oods_shifted_by_0_var.0,
        prepared_oods_shifted_by_0_var.1,
        x_var,
        y_var,
    )?);
    denominator_inverses.push(denominator_inverse_limbs_from_prepared(
        &mut dsl,
        table,
        prepared_oods_shifted_by_1_var.0,
        prepared_oods_shifted_by_1_var.1,
        x_var,
        y_var,
    )?);
    denominator_inverses.push(denominator_inverse_limbs_from_prepared(
        &mut dsl,
        table,
        prepared_oods_shifted_by_2_var.0,
        prepared_oods_shifted_by_2_var.1,
        x_var,
        y_var,
    )?);
    denominator_inverses.push(denominator_inverse_limbs_from_prepared(
        &mut dsl,
        table,
        prepared_oods_var.0,
        prepared_oods_var.1,
        x_var,
        y_var,
    )?);

    let numerator_trace_0 = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results.0,
        trace_queried_results.1,
        column_line_coeff_trace_0_var.0,
        column_line_coeff_trace_0_var.1,
    )?;
    let numerator_trace_1 = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results.0,
        trace_queried_results.1,
        column_line_coeff_trace_1_var.0,
        column_line_coeff_trace_1_var.1,
    )?;
    let numerator_trace_2 = apply_twin(
        &mut dsl,
        table,
        y_var,
        trace_queried_results.0,
        trace_queried_results.1,
        column_line_coeff_trace_2_var.0,
        column_line_coeff_trace_2_var.1,
    )?;

    let mut list_num_denom1_result = vec![];
    for denominator_inverse in denominator_inverses.iter() {
        list_num_denom1_result.push(denominator_inverse.0);
        list_num_denom1_result.push(denominator_inverse.1);
    }
    list_num_denom1_result.push(numerator_trace_0.0);
    list_num_denom1_result.push(numerator_trace_0.1);
    list_num_denom1_result.push(numerator_trace_1.0);
    list_num_denom1_result.push(numerator_trace_1.1);
    list_num_denom1_result.push(numerator_trace_2.0);
    list_num_denom1_result.push(numerator_trace_2.1);

    let (pack_cur_num_denom1_result_hash, pack_cur_num_denom1_result) =
        zip_elements(&mut dsl, &list_num_denom1_result)?;

    let name = format!("query{}_num_denom1_result", query_idx);
    cache.insert(name, pack_cur_num_denom1_result);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_num_denom1_result_hash)?;
    dsl.set_program_output("hash", pack_cur_num2_hash)?;

    Ok(dsl)
}
