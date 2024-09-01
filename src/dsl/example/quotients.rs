use crate::dsl::cm31::cm31_mul_m31_limbs;
use crate::dsl::qm31::qm31_mul_cm31_limbs;
use anyhow::Result;
use bitcoin_script_dsl::dsl::DSL;

#[derive(Clone)]
pub struct DenominatorInversesIndices {
    pub u1: usize,
    pub u2: usize,
    pub u3: usize,
    pub u4: usize,
}

#[derive(Clone)]
pub struct NominatorsIndices {
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub c1: usize,
    pub c2: usize,
    pub c3: usize,
    pub c4: usize,
}

pub fn aggregation(
    dsl: &mut DSL,
    table: usize,
    denominator_inverses_indices: &DenominatorInversesIndices,
    nominators_indices: &NominatorsIndices,
    power_alphas: &[usize],
) -> Result<usize> {
    // alpha^6 * (a1 * u1) + alpha^5 * (a2 * u2) + alpha^4 * (a3 * u3) +
    // (alpha^3 * c1 + alpha^2 * c2 + alpha * c3 + c4) * u4

    let u1_limbs = dsl.execute("cm31_to_limbs", &[denominator_inverses_indices.u1])?[0];
    let a1_limbs = dsl.execute("cm31_to_limbs", &[nominators_indices.a1])?[0];
    let u1a1 = dsl.execute("cm31_limbs_mul", &[table, u1_limbs, a1_limbs])?[0];

    let u2_limbs = dsl.execute("cm31_to_limbs", &[denominator_inverses_indices.u2])?[0];
    let a2_limbs = dsl.execute("cm31_to_limbs", &[nominators_indices.a2])?[0];
    let u2a2 = dsl.execute("cm31_limbs_mul", &[table, u2_limbs, a2_limbs])?[0];

    let u3_limbs = dsl.execute("cm31_to_limbs", &[denominator_inverses_indices.u3])?[0];
    let a3_limbs = dsl.execute("cm31_to_limbs", &[nominators_indices.a3])?[0];
    let u3a3 = dsl.execute("cm31_limbs_mul", &[table, u3_limbs, a3_limbs])?[0];

    let c1_limbs = dsl.execute("cm31_to_limbs", &[nominators_indices.c1])?[0];
    let alpha3c1 = qm31_mul_cm31_limbs(dsl, table, power_alphas[3], c1_limbs)?;
    // power_alphas[3] = alpha^3

    let c2_limbs = dsl.execute("cm31_to_limbs", &[nominators_indices.c2])?[0];
    let alpha2c2 = qm31_mul_cm31_limbs(dsl, table, power_alphas[4], c2_limbs)?;
    // power_alphas[4] = alpha^2

    let c3_limbs = dsl.execute("cm31_to_limbs", &[nominators_indices.c3])?[0];
    let alpha1c3 = qm31_mul_cm31_limbs(dsl, table, power_alphas[5], c3_limbs)?;
    // power_alphas[5] = alpha

    let mut sum = dsl.execute("qm31_add", &[alpha3c1, alpha2c2])?[0];
    sum = dsl.execute("qm31_add", &[sum, alpha1c3])?[0];
    sum = dsl.execute("qm31_add_cm31", &[sum, nominators_indices.c4])?[0];

    let u4_limbs = dsl.execute("cm31_to_limbs", &[denominator_inverses_indices.u4])?[0];
    let sumu4 = qm31_mul_cm31_limbs(dsl, table, sum, u4_limbs)?;

    let u1a1_limbs = dsl.execute("cm31_to_limbs", &[u1a1])?[0];
    let alpha6u1a1 = qm31_mul_cm31_limbs(dsl, table, power_alphas[0], u1a1_limbs)?;

    let u2a2_limbs = dsl.execute("cm31_to_limbs", &[u2a2])?[0];
    let alpha5u2a2 = qm31_mul_cm31_limbs(dsl, table, power_alphas[1], u2a2_limbs)?;

    let u3a3_limbs = dsl.execute("cm31_to_limbs", &[u3a3])?[0];
    let alpha4u3a3 = qm31_mul_cm31_limbs(dsl, table, power_alphas[2], u3a3_limbs)?;

    let mut res = dsl.execute("qm31_add", &[sumu4, alpha6u1a1])?[0];
    res = dsl.execute("qm31_add", &[res, alpha5u2a2])?[0];
    res = dsl.execute("qm31_add", &[res, alpha4u3a3])?[0];

    Ok(res)
}

pub fn denominator_inverse_limbs_from_prepared(
    dsl: &mut DSL,
    table: usize,
    x_imag_div_y_imag: usize,
    cross_term: usize,
    z_x: usize,
    z_y: usize,
) -> Result<(usize, usize)> {
    let cross_term_plus_z_x = dsl.execute("cm31_add_m31", &[cross_term, z_x])?[0];

    let z_y_limbs = dsl.execute("m31_to_limbs", &[z_y])?[0];
    let x_imag_div_y_imag_times_z_y = cm31_mul_m31_limbs(dsl, table, x_imag_div_y_imag, z_y_limbs)?;

    let result_for_z = dsl.execute(
        "cm31_sub",
        &[cross_term_plus_z_x, x_imag_div_y_imag_times_z_y],
    )?[0];
    let result_for_conjugated_z = dsl.execute(
        "cm31_add",
        &[cross_term_plus_z_x, x_imag_div_y_imag_times_z_y],
    )?[0];

    let result_for_z_limbs = dsl.execute("cm31_to_limbs", &[result_for_z])?[0];
    let inverse_for_z_limbs = dsl.execute("cm31_limbs_inverse", &[table, result_for_z_limbs])?[0];

    let result_for_conjugated_z_limbs =
        dsl.execute("cm31_to_limbs", &[result_for_conjugated_z])?[0];
    let inverse_for_conjugated_z_limbs = dsl.execute(
        "cm31_limbs_inverse",
        &[table, result_for_conjugated_z_limbs],
    )?[0];

    Ok((inverse_for_z_limbs, inverse_for_conjugated_z_limbs))
}

#[cfg(test)]
mod test {
    use crate::dsl::cm31::reformat_cm31_to_dsl_element;
    use crate::dsl::example::quotients::{
        aggregation, denominator_inverse_limbs_from_prepared, DenominatorInversesIndices,
        NominatorsIndices,
    };
    use crate::dsl::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::{load_data_types, load_functions};
    use crate::utils::convert_cm31_to_limbs;
    use bitcoin_circle_stark::constraints::{
        fast_twin_pair_vanishing_from_prepared, PreparedPairVanishing,
    };
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use num_traits::Zero;
    use rand::{Rng, RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::circle::{M31_CIRCLE_GEN, SECURE_FIELD_CIRCLE_GEN};
    use stwo_prover::core::fields::cm31::CM31;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::fields::qm31::QM31;
    use stwo_prover::core::fields::FieldExpOps;

    #[test]
    fn test_aggregation() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let u1 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let u2 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let u3 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let u4 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));

        let a1 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let a2 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let a3 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let c1 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let c2 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let c3 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));
        let c4 = CM31(M31::reduce(prng.next_u64()), M31::reduce(prng.next_u64()));

        let alpha = get_rand_qm31(&mut prng);

        let expected = alpha.pow(6).mul_cm31(u1 * a1)
            + alpha.pow(5).mul_cm31(u2 * a2)
            + alpha.pow(4).mul_cm31(u3 * a3)
            + (alpha.pow(3).mul_cm31(c1)
                + alpha.pow(2).mul_cm31(c2)
                + alpha.mul_cm31(c3)
                + QM31(c4, CM31::zero()))
            .mul_cm31(u4);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let u1_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(u1)))
            .unwrap();
        let u2_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(u2)))
            .unwrap();
        let u3_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(u3)))
            .unwrap();
        let u4_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(u4)))
            .unwrap();

        let a1_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(a1)))
            .unwrap();
        let a2_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(a2)))
            .unwrap();
        let a3_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(a3)))
            .unwrap();
        let c1_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(c1)))
            .unwrap();
        let c2_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(c2)))
            .unwrap();
        let c3_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(c3)))
            .unwrap();
        let c4_var = dsl
            .alloc_input("cm31", Element::ManyNum(reformat_cm31_to_dsl_element(c4)))
            .unwrap();

        let denominator_inverses_indices = DenominatorInversesIndices {
            u1: u1_var,
            u2: u2_var,
            u3: u3_var,
            u4: u4_var,
        };

        let nominators_indices = NominatorsIndices {
            a1: a1_var,
            a2: a2_var,
            a3: a3_var,
            c1: c1_var,
            c2: c2_var,
            c3: c3_var,
            c4: c4_var,
        };

        let alpha6 = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha.pow(6))),
            )
            .unwrap();
        let alpha5 = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha.pow(5))),
            )
            .unwrap();
        let alpha4 = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha.pow(4))),
            )
            .unwrap();
        let alpha3 = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha.pow(3))),
            )
            .unwrap();
        let alpha2 = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha.pow(2))),
            )
            .unwrap();
        let alpha1 = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha)),
            )
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = aggregation(
            &mut dsl,
            table,
            &denominator_inverses_indices,
            &nominators_indices,
            &[alpha6, alpha5, alpha4, alpha3, alpha2, alpha1],
        )
        .unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(expected)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { expected }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_denominator_inverse_from_prepared() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let random_qm31_point = SECURE_FIELD_CIRCLE_GEN.mul(prng.gen());
        let prepared = PreparedPairVanishing::from(random_qm31_point);
        let x_imag_div_y_imag = prepared.x_imag_div_y_imag.clone();
        let cross_term = prepared.cross_term.clone();

        let random_m31_point = M31_CIRCLE_GEN.mul(prng.gen());

        let twin_pair_vanishing_result =
            fast_twin_pair_vanishing_from_prepared(prepared, random_m31_point);

        let expected = (
            twin_pair_vanishing_result.0.inverse(),
            twin_pair_vanishing_result.1.inverse(),
        );

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let x_imag_div_y_imag_var = dsl
            .alloc_input(
                "cm31",
                Element::ManyNum(reformat_cm31_to_dsl_element(x_imag_div_y_imag)),
            )
            .unwrap();
        let cross_term_var = dsl
            .alloc_input(
                "cm31",
                Element::ManyNum(reformat_cm31_to_dsl_element(cross_term)),
            )
            .unwrap();
        let z_x = dsl
            .alloc_input("m31", Element::Num(random_m31_point.x.0 as i32))
            .unwrap();
        let z_y = dsl
            .alloc_input("m31", Element::Num(random_m31_point.y.0 as i32))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = denominator_inverse_limbs_from_prepared(
            &mut dsl,
            table,
            x_imag_div_y_imag_var,
            cross_term_var,
            z_x,
            z_y,
        )
        .unwrap();

        assert_eq!(
            dsl.get_many_num(res.0).unwrap(),
            convert_cm31_to_limbs(expected.0)
        );
        assert_eq!(
            dsl.get_many_num(res.1).unwrap(),
            convert_cm31_to_limbs(expected.1)
        );

        dsl.set_program_output("cm31_limbs", res.0).unwrap();
        dsl.set_program_output("cm31_limbs", res.1).unwrap();

        test_program(
            dsl,
            script! {
                { convert_cm31_to_limbs(expected.0).to_vec() }
                { convert_cm31_to_limbs(expected.1).to_vec() }
            },
        )
        .unwrap();
    }
}
