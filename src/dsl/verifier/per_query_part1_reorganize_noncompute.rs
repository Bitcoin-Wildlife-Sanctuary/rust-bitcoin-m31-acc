use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
use crate::dsl::verifier::hints::Hints;
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

    // Step 1: store the denominator inverses related variables into a pack
    let mut list_denominator_inverses = vec![];
    list_denominator_inverses.push(prepared_oods_shifted_by_0_var.0);
    list_denominator_inverses.push(prepared_oods_shifted_by_0_var.1);
    list_denominator_inverses.push(prepared_oods_shifted_by_1_var.0);
    list_denominator_inverses.push(prepared_oods_shifted_by_1_var.1);
    list_denominator_inverses.push(prepared_oods_shifted_by_2_var.0);
    list_denominator_inverses.push(prepared_oods_shifted_by_2_var.1);
    list_denominator_inverses.push(prepared_oods_var.0);
    list_denominator_inverses.push(prepared_oods_var.1);
    list_denominator_inverses.push(twiddles_vars[5]);
    list_denominator_inverses.push(twiddles_vars[6]);

    // Step 2: store the denominator inverses related variables into a pack
    let (pack_cur_denominator_inverses_hash, pack_cur_denominator_inverses) =
        zip_elements(&mut dsl, &list_denominator_inverses)?;

    let name = format!("query{}_denominator_inverses", query_idx);
    cache.insert(name, pack_cur_denominator_inverses);

    // Step 3: allocate the part for FRI folding
    let mut list_fri_folding = vec![];
    list_fri_folding.push(query_var);
    list_fri_folding.push(last_layer_var);
    list_fri_folding.extend_from_slice(&folding_alphas_vars);
    list_fri_folding.extend_from_slice(&twiddles_vars[0..5]);
    for folding_intermediate_results_var in folding_intermediate_results_vars.iter() {
        list_fri_folding.push(folding_intermediate_results_var.0);
        list_fri_folding.push(folding_intermediate_results_var.1);
    }
    list_fri_folding.push(pack_cur_denominator_inverses_hash);

    // Step 4: store the FRI folding related variables into a pack
    let (pack_cur_fri_folding_hash, pack_cur_fri_folding) =
        zip_elements(&mut dsl, &list_fri_folding)?;

    let name = format!("query{}_fri_folding1", query_idx);
    cache.insert(name, pack_cur_fri_folding);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_fri_folding_hash)?;

    // Handle the rest later on

    Ok(dsl)
}
