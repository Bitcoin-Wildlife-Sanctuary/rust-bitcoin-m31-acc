use crate::dsl::building_blocks::qm31::qm31_mul_cm31_limbs;
use crate::dsl::fibonacci::hints::Hints;
use crate::dsl::framework::dsl::{Element, DSL};
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

    let cur_aggregation3_hash = dsl.alloc_input(
        "hash",
        Element::Str(
            cache
                .get(&format!("query{}_aggregation3", query_idx))
                .unwrap()
                .hash
                .as_ref()
                .to_vec(),
        ),
    )?;

    // unzip the cur_aggregation1_result_hash
    let res = unzip_elements(
        &mut dsl,
        cur_aggregation3_hash,
        cache
            .get(format!("query{}_aggregation3", query_idx).as_str())
            .unwrap(),
    )?;
    assert_eq!(res.len(), 18);

    let y_var = res[0];
    let denominator_inverses_oods = (res[1], res[2]);
    let numerator_composition_0_1 = res[3];
    let numerator_composition_1 = (res[4], res[5]);
    let numerator_composition_2 = (res[6], res[7]);
    let numerator_composition_3 = (res[8], res[9]);
    let alphas = [res[10], res[11], res[12]];
    let circle_poly_alpha_var = res[13];
    let expected_entry_quotient = res[14];
    let sum_ua654 = res[15];
    let sum_vb654 = res[16];
    let alpha3c1 = res[17];

    // Step 1: push the table
    let table = dsl.execute("push_table", &[])?[0];

    let c2_limbs = dsl.execute("cm31_to_limbs", &[numerator_composition_1.0])?[0];
    let alpha2c2 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[1], c2_limbs)?;

    let c3_limbs = dsl.execute("cm31_to_limbs", &[numerator_composition_2.0])?[0];
    let alpha1c3 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[2], c3_limbs)?;

    let mut sum_c = dsl.execute("qm31_add", &[alpha3c1, alpha2c2])?[0];
    sum_c = dsl.execute("qm31_add", &[sum_c, alpha1c3])?[0];
    sum_c = dsl.execute("qm31_add_cm31", &[sum_c, numerator_composition_3.0])?[0];

    let u4_limbs = denominator_inverses_oods.0;
    let sum_u4 = qm31_mul_cm31_limbs(&mut dsl, table, sum_c, u4_limbs)?;

    let sum_left = dsl.execute("qm31_add", &[sum_ua654, sum_u4])?[0];

    let d1_limbs = dsl.execute("cm31_to_limbs", &[numerator_composition_0_1])?[0];
    let alpha3d1 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[0], d1_limbs)?;

    let d2_limbs = dsl.execute("cm31_to_limbs", &[numerator_composition_1.1])?[0];
    let alpha2d2 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[1], d2_limbs)?;

    let d3_limbs = dsl.execute("cm31_to_limbs", &[numerator_composition_2.1])?[0];
    let alpha1d3 = qm31_mul_cm31_limbs(&mut dsl, table, alphas[2], d3_limbs)?;

    let mut sum_d = dsl.execute("qm31_add", &[alpha3d1, alpha2d2])?[0];
    sum_d = dsl.execute("qm31_add", &[sum_d, alpha1d3])?[0];
    sum_d = dsl.execute("qm31_add_cm31", &[sum_d, numerator_composition_3.1])?[0];

    let v4_limbs = denominator_inverses_oods.1;
    let sum_v4 = qm31_mul_cm31_limbs(&mut dsl, table, sum_d, v4_limbs)?;

    let sum_right = dsl.execute("qm31_add", &[sum_vb654, sum_v4])?[0];

    let mut list_aggregation4 = vec![];
    list_aggregation4.push(y_var);
    list_aggregation4.push(sum_left);
    list_aggregation4.push(sum_right);
    list_aggregation4.push(circle_poly_alpha_var);
    list_aggregation4.push(expected_entry_quotient);

    let (pack_cur_aggregation4_hash, pack_cur_aggregation4) =
        zip_elements(&mut dsl, &list_aggregation4)?;

    let name = format!("query{}_aggregation4", query_idx);
    cache.insert(name, pack_cur_aggregation4);

    dsl.set_program_output("hash", global_state_hash)?;
    dsl.set_program_output("hash", pack_cur_aggregation4_hash)?;

    Ok(dsl)
}
