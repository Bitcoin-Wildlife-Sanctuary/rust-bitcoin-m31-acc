use crate::dsl::building_blocks::qm31::qm31_mul_cm31_limbs;
use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::load_data_types;
use crate::dsl::load_functions;
use crate::dsl::tools::{unzip_elements, zip_elements, Zipper};
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

    let cur_aggregation1_result_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_aggregation1_result", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    let cur_aggregation2_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_aggregation2", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip the cur_aggregation1_result_hash
    let res = unzip_elements(
        &mut dsl,
        cur_aggregation1_result_hash,
        cache
            .get(format!("query{}_aggregation1_result", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 17);

    let y_var = res[0];
    let u2a2 = res[1];
    let u3a3 = res[2];
    let v1b1 = res[3];
    let v2b2 = res[4];
    let v3b3 = res[5];
    let alpha6u1a1 = res[6];
    let denominator_inverses_oods = (res[7], res[8]);
    let numerator_composition_0 = (res[9], res[10]);
    let numerator_composition_1 = (res[11], res[12]);
    let numerator_composition_2 = (res[13], res[14]);
    let numerator_composition_3 = (res[15], res[16]);

    // unzip the cur_aggregation2_hash
    let res = unzip_elements(
        &mut dsl,
        cur_aggregation2_hash,
        cache
            .get(format!("query{}_aggregation2", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 10);

    let alphas = [res[0], res[1], res[2], res[3], res[4], res[5]];
    let circle_poly_alpha_var = res[6];
    let query_var = res[7];
    let folding_immediate_results_vars = (res[8], res[9]);

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    // Step 2: continue with the aggregation
    let u2a2_limbs = dsl.execute("cm31_to_limbs", &[u2a2])?[0];
    let alpha5u2a2 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[1], u2a2_limbs)?;

    let u3a3_limbs = dsl.execute("cm31_to_limbs", &[u3a3])?[0];
    let alpha4u3a3 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[2], u3a3_limbs)?;

    let mut sum_ua654 = dsl.execute("qm31_add", &[alpha6u1a1, alpha5u2a2])?[0];
    sum_ua654 = dsl.execute("qm31_add", &[sum_ua654, alpha4u3a3])?[0];

    let v1b1_limbs = dsl.execute("cm31_to_limbs", &[v1b1])?[0];
    let alpha6v1b1 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[0], v1b1_limbs)?;

    let v2b2_limbs = dsl.execute("cm31_to_limbs", &[v2b2])?[0];
    let alpha5v2b2 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[1], v2b2_limbs)?;

    let v3b3_limbs = dsl.execute("cm31_to_limbs", &[v3b3])?[0];
    let alpha4v3b3 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[2], v3b3_limbs)?;

    let mut sum_vb654 = dsl.execute("qm31_add", &[alpha6v1b1, alpha5v2b2])?[0];
    sum_vb654 = dsl.execute("qm31_add", &[sum_vb654, alpha4v3b3])?[0];

    let c1_limbs = dsl.execute("cm31_to_limbs", &[numerator_composition_0.0])?[0];
    let alpha3c1 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[3], c1_limbs)?;

    let swap_bit_var = dsl.execute("skip_one_and_extract_5_bits", &[query_var])?[0];
    let expected_entry_quotient = dsl.execute(
        "qm31_conditional_swap",
        &[
            folding_immediate_results_vars.0,
            folding_immediate_results_vars.1,
            swap_bit_var,
        ],
    )?[0];

    // Step 3: allocate the list for the third part of aggregation
    let mut list_aggregation3 = vec![];
    list_aggregation3.push(y_var);
    list_aggregation3.push(denominator_inverses_oods.0);
    list_aggregation3.push(denominator_inverses_oods.1);
    list_aggregation3.push(numerator_composition_0.1);
    list_aggregation3.push(numerator_composition_1.0);
    list_aggregation3.push(numerator_composition_1.1);
    list_aggregation3.push(numerator_composition_2.0);
    list_aggregation3.push(numerator_composition_2.1);
    list_aggregation3.push(numerator_composition_3.0);
    list_aggregation3.push(numerator_composition_3.1);
    list_aggregation3.push(alphas[3]);
    list_aggregation3.push(alphas[4]);
    list_aggregation3.push(alphas[5]);
    list_aggregation3.push(circle_poly_alpha_var);
    list_aggregation3.push(expected_entry_quotient);
    list_aggregation3.push(sum_ua654);
    list_aggregation3.push(sum_vb654);
    list_aggregation3.push(alpha3c1);

    // Step 4: store the third part of aggregation related variables into a pack
    let (pack_cur_aggregation3_hash, pack_cur_aggregation3) =
        zip_elements(&mut dsl, &list_aggregation3)?;

    let name = format!("query{}_aggregation3", query_idx);
    cache.insert(name, pack_cur_aggregation3);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_aggregation3_hash)?;

    Ok(dsl)
}
