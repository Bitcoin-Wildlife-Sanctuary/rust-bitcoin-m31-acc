use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::framework::dsl::{Element, DSL};
use crate::dsl::modules::fold::ibutterfly;
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

    let cur_fri_folding_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_fri_folding1", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip the cur_fri_folding_hash
    let res = unzip_elements(
        &mut dsl,
        cur_fri_folding_hash,
        cache
            .get(format!("query{}_fri_folding1", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 23);

    let query_var = res[0];
    let last_layer_var = res[1];
    let folding_alphas_vars = [res[2], res[3], res[4], res[5], res[6]];
    let twiddles_vars = [res[7], res[8], res[9], res[10], res[11]];
    let folding_intermediate_results = [
        (res[12], res[13]),
        (res[14], res[15]),
        (res[16], res[17]),
        (res[18], res[19]),
        (res[20], res[21]),
    ];
    let pack_cur_denominator_inverses_hash = res[22];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: bit-compose the query's 5 bits (after the lowest bit)
    let swap_bits_vars = dsl.execute("skip_one_and_extract_5_bits", &[query_var])?;

    // Step 3: perform the first 2 butterfly
    let mut folded_results_vars = vec![];
    for ((folding_intermediate_result, &twiddle_var), &folding_alpha_var) in
        folding_intermediate_results
            .iter()
            .zip(twiddles_vars.iter().rev())
            .zip(folding_alphas_vars.iter())
            .take(2)
    {
        let ifft_results_vars = ibutterfly(
            &mut dsl,
            table,
            folding_intermediate_result.0,
            folding_intermediate_result.1,
            twiddle_var,
        )?;

        let second_limbs_var = dsl.execute("qm31_to_limbs", &[ifft_results_vars.1])?[0];
        let folding_alpha_limbs_var = dsl.execute("qm31_to_limbs", &[folding_alpha_var])?[0];

        let second_times_folding_alpha_var = dsl.execute(
            "qm31_limbs_mul",
            &[table, second_limbs_var, folding_alpha_limbs_var],
        )?[0];
        folded_results_vars.push(
            dsl.execute(
                "qm31_add",
                &[second_times_folding_alpha_var, ifft_results_vars.0],
            )?[0],
        );
    }

    let swapped_result = dsl.execute(
        "qm31_conditional_swap",
        &[
            folding_intermediate_results[1].0,
            folding_intermediate_results[1].1,
            swap_bits_vars[1],
        ],
    )?[0];
    let _ = dsl.execute(
        "qm31_equalverify",
        &[swapped_result, folded_results_vars[0]],
    )?;

    let swapped_result = dsl.execute(
        "qm31_conditional_swap",
        &[
            folding_intermediate_results[2].0,
            folding_intermediate_results[2].1,
            swap_bits_vars[2],
        ],
    )?[0];
    let _ = dsl.execute(
        "qm31_equalverify",
        &[swapped_result, folded_results_vars[1]],
    )?;

    let mut list_fri_folding = vec![];
    list_fri_folding.push(query_var);
    list_fri_folding.push(last_layer_var);
    list_fri_folding.extend_from_slice(&folding_alphas_vars[2..]);
    list_fri_folding.extend_from_slice(&twiddles_vars[0..3]);
    for folding_intermediate_result in folding_intermediate_results.iter().skip(2) {
        list_fri_folding.push(folding_intermediate_result.0);
        list_fri_folding.push(folding_intermediate_result.1);
    }
    list_fri_folding.push(pack_cur_denominator_inverses_hash);

    // Step 2: store the FRI folding related variables into a pack
    let (pack_cur_fri_folding_hash, pack_cur_fri_folding) =
        zip_elements(&mut dsl, &list_fri_folding)?;

    let name = format!("query{}_fri_folding2", query_idx);
    cache.insert(name, pack_cur_fri_folding);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_fri_folding_hash)?;

    Ok(dsl)
}
