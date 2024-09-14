use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::{load_data_types, load_functions};
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use bitcoin_script_dsl::options::Options;
use std::collections::HashMap;

pub fn generate_dsl(
    _: &Hints,
    cache: &mut HashMap<String, Zipper>,
    query_idx: usize,
) -> Result<DSL> {
    // to be merged, as a subfunction, to part9 and part10

    let mut dsl = DSL::new();

    load_data_types(&mut dsl)?;
    load_functions(&mut dsl)?;

    // assume, that the inputs, contain the hashes from the previous step
    let global_state_hash = dsl.alloc_input(
        "hash",
        Element::Str(cache.get("global_state").unwrap().hash.as_ref().to_vec()),
    )?;

    // unzip the global state
    let res = unzip_elements(
        &mut dsl,
        global_state_hash,
        cache.get("global_state").unwrap(),
    )?;
    assert_eq!(res.len(), 9);

    let shared_information_hash = res[0];

    let mut all_queries_hashes = vec![];
    for i in 1..=8 {
        all_queries_hashes.push(res[i]);
    }

    let current_hash = dsl.execute_with_options(
        "select_among_eight",
        &all_queries_hashes,
        &Options::new().with_u32("pick", query_idx as u32),
    )?[0];

    // unzip the shared information
    let res = unzip_elements(
        &mut dsl,
        shared_information_hash,
        cache.get("shared_information").unwrap(),
    )?;
    assert_eq!(res.len(), 35);

    let prepared_oods_shifted_by_0_var = (res[0], res[1]);
    let prepared_oods_shifted_by_1_var = (res[2], res[3]);
    let prepared_oods_shifted_by_2_var = (res[4], res[5]);
    let prepared_oods_var = (res[6], res[7]);
    let column_line_coeff_trace_0_var = (res[8], res[9]);
    let column_line_coeff_trace_1_var = (res[10], res[11]);
    let column_line_coeff_trace_2_var = (res[12], res[13]);
    let column_line_coeff_composition_vars = [
        (res[14], res[15]),
        (res[16], res[17]),
        (res[18], res[19]),
        (res[20], res[21]),
    ];
    let alphas = [res[22], res[23], res[24], res[25], res[26], res[27]];
    let circle_poly_alpha_var = res[28];
    let last_layer_var = res[29];
    let folding_alphas_vars = [res[30], res[31], res[32], res[33], res[34]];

    // unzip the current hash
    let res = unzip_elements(
        &mut dsl,
        current_hash,
        cache.get(format!("query{}", query_idx).as_str()).unwrap(),
    )?;
    assert_eq!(res.len(), 22);

    let query_var = res[0];
    let trace_queried_results = (res[1], res[2]);
    let composition_queried_results = (res[3], res[4]);
    let twiddles_vars = [res[5], res[6], res[7], res[8], res[9], res[10], res[11]];
    let folding_intermediate_results_vars = [
        (res[12], res[13]),
        (res[14], res[15]),
        (res[16], res[17]),
        (res[18], res[19]),
        (res[20], res[21]),
    ];

    // Step 1: store the aggregation related variables into a pack
    let mut list_aggregation2 = vec![];
    list_aggregation2.extend_from_slice(&alphas);
    list_aggregation2.push(circle_poly_alpha_var);
    list_aggregation2.push(query_var);
    list_aggregation2.push(folding_intermediate_results_vars[0].0);
    list_aggregation2.push(folding_intermediate_results_vars[0].1);

    let (pack_cur_aggregation2_hash, pack_cur_aggregation2) =
        zip_elements(&mut dsl, &list_aggregation2)?;

    let name = format!("query{}_aggregation2", query_idx);
    cache.insert(name, pack_cur_aggregation2);

    // Step 2: store the numerator related variables into a pack
    let mut list_num2 = vec![];
    list_num2.push(twiddles_vars[6]);
    list_num2.push(composition_queried_results.0);
    list_num2.push(composition_queried_results.1);
    for column_line_coeff_composition_var in column_line_coeff_composition_vars.iter() {
        list_num2.push(column_line_coeff_composition_var.0);
        list_num2.push(column_line_coeff_composition_var.1);
    }
    list_num2.push(alphas[0]);
    list_num2.push(pack_cur_aggregation2_hash);

    let (pack_cur_num2_hash, pack_cur_num2) = zip_elements(&mut dsl, &list_num2)?;

    let name = format!("query{}_num2", query_idx);
    cache.insert(name, pack_cur_num2);

    // Step 3: store the denominator inverses related variables into a pack
    let mut list_num_denom_1 = vec![];
    list_num_denom_1.push(prepared_oods_shifted_by_0_var.0);
    list_num_denom_1.push(prepared_oods_shifted_by_0_var.1);
    list_num_denom_1.push(prepared_oods_shifted_by_1_var.0);
    list_num_denom_1.push(prepared_oods_shifted_by_1_var.1);
    list_num_denom_1.push(prepared_oods_shifted_by_2_var.0);
    list_num_denom_1.push(prepared_oods_shifted_by_2_var.1);
    list_num_denom_1.push(prepared_oods_var.0);
    list_num_denom_1.push(prepared_oods_var.1);
    list_num_denom_1.push(twiddles_vars[5]);
    list_num_denom_1.push(twiddles_vars[6]);

    list_num_denom_1.push(trace_queried_results.0);
    list_num_denom_1.push(trace_queried_results.1);
    list_num_denom_1.push(column_line_coeff_trace_0_var.0);
    list_num_denom_1.push(column_line_coeff_trace_0_var.1);
    list_num_denom_1.push(column_line_coeff_trace_1_var.0);
    list_num_denom_1.push(column_line_coeff_trace_1_var.1);
    list_num_denom_1.push(column_line_coeff_trace_2_var.0);
    list_num_denom_1.push(column_line_coeff_trace_2_var.1);

    list_num_denom_1.push(pack_cur_num2_hash);

    let (pack_cur_num_denom1_hash, pack_cur_num_denom1) =
        zip_elements(&mut dsl, &list_num_denom_1)?;

    let name = format!("query{}_num_denom1", query_idx);
    cache.insert(name, pack_cur_num_denom1);

    // Step 4: allocate the part for FRI folding
    let mut list_fri_folding = vec![];
    list_fri_folding.push(query_var);
    list_fri_folding.push(last_layer_var);
    list_fri_folding.extend_from_slice(&folding_alphas_vars);
    list_fri_folding.extend_from_slice(&twiddles_vars[0..5]);
    for folding_intermediate_results_var in folding_intermediate_results_vars.iter() {
        list_fri_folding.push(folding_intermediate_results_var.0);
        list_fri_folding.push(folding_intermediate_results_var.1);
    }
    list_fri_folding.push(pack_cur_num_denom1_hash);

    // Step 5: store the FRI folding related variables into a pack
    let (pack_cur_fri_folding_hash, pack_cur_fri_folding) =
        zip_elements(&mut dsl, &list_fri_folding)?;

    let name = format!("query{}_fri_folding1", query_idx);
    cache.insert(name, pack_cur_fri_folding);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_fri_folding_hash)?;

    Ok(dsl)
}
