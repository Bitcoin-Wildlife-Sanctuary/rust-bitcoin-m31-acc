use crate::dsl::qm31::qm31_mul_m31_limbs;
use crate::utils::convert_m31_to_limbs;
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use num_traits::One;
use stwo_prover::core::circle::{CirclePoint, Coset};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;

pub fn eval_from_partial_evals(
    dsl: &mut DSL,
    a: usize,
    b: usize,
    c: usize,
    d: usize,
) -> Result<usize> {
    let b_shift_by_i = dsl.execute("qm31_shift_by_i", &[b])?[0];
    let c_shift_by_j = dsl.execute("qm31_shift_by_j", &[c])?[0];
    let d_shift_by_ij = dsl.execute("qm31_shift_by_ij", &[d])?[0];

    let mut sum = dsl.execute("qm31_add", &[a, b_shift_by_i])?[0];
    sum = dsl.execute("qm31_add", &[sum, c_shift_by_j])?[0];
    sum = dsl.execute("qm31_add", &[sum, d_shift_by_ij])?[0];

    Ok(sum)
}

pub fn eval_composition_polynomial_at_point(
    dsl: &mut DSL,
    table: usize,
    alpha: usize,
    f_z: usize,
    f_gz: usize,
    f_g2z: usize,
    z_x: usize,
    z_y: usize,
    log_size: u32,
    claim: M31,
) -> Result<usize> {
    // compute the boundary constraint evaluation
    let constraint_zero_domain = Coset::subgroup(log_size);
    let p = constraint_zero_domain.at(constraint_zero_domain.size() - 1);

    // numerator
    let claim_minus_one_times_p_y_inverse = (claim - M31::one()) * p.y.inverse();
    let claim_minus_one_times_p_y_inverse_limbs = dsl.alloc_constant(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(claim_minus_one_times_p_y_inverse.0).to_vec()),
    )?;

    let mut linear = qm31_mul_m31_limbs(dsl, table, z_y, claim_minus_one_times_p_y_inverse_limbs)?;
    linear = dsl.execute("qm31_1add", &[linear])?[0];

    let numerator = dsl.execute("qm31_sub", &[f_z, linear])?[0];

    // denominator
    let denominator =
        pair_vanishing_with_constant_m31_points(dsl, table, z_x, z_y, p, CirclePoint::zero())?;

    let numerator_limbs = dsl.execute("qm31_to_limbs", &[numerator])?[0];
    let denominator_limbs = dsl.execute("qm31_to_limbs", &[denominator])?[0];
    let denominator_limbs_inverse = dsl.execute("qm31_limbs_inverse", &[denominator_limbs])?[0];

    let boundary_res = dsl.execute(
        "qm31_limbs_mul",
        &[table, numerator_limbs, denominator_limbs_inverse],
    )?[0];

    // compute the step constraint evaluation

    todo!()
}

pub fn pair_vanishing_with_constant_m31_points(
    dsl: &mut DSL,
    table: usize,
    z_x: usize,
    z_y: usize,
    excluded0: CirclePoint<M31>,
    excluded1: CirclePoint<M31>,
) -> Result<usize> {
    let excluded_1_x_minus_excluded_0_x = excluded1.x - excluded0.x;
    let excluded_1_x_minus_excluded_0_x_limbs = dsl.alloc_constant(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(excluded_1_x_minus_excluded_0_x.0).to_vec()),
    )?;

    let z_y_part = qm31_mul_m31_limbs(dsl, table, z_y, excluded_1_x_minus_excluded_0_x_limbs)?;

    let excluded_0_y_minus_excluded_1_y = excluded0.y - excluded1.y;
    let excluded_0_y_minus_excluded_1_y_limbs = dsl.alloc_constant(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(excluded_0_y_minus_excluded_1_y.0).to_vec()),
    )?;

    let z_x_part = qm31_mul_m31_limbs(dsl, table, z_x, excluded_0_y_minus_excluded_1_y_limbs)?;

    let mut sum = dsl.execute("qm31_add", &[z_x_part, z_y_part])?[0];
    let constant = dsl.alloc_constant(
        "m31",
        Element::Num((excluded0.x * excluded1.y - excluded0.y * excluded1.x).0 as i32),
    )?;
    sum = dsl.execute("qm31_add_m31", &[sum, constant])?[0];

    Ok(sum)
}

#[cfg(test)]
mod test {
    use crate::dsl::example::fiat_shamir::{
        eval_from_partial_evals, pair_vanishing_with_constant_m31_points,
    };
    use crate::dsl::load_data_types;
    use crate::dsl::load_functions;
    use crate::dsl::qm31::reformat_qm31_to_dsl_element;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::circle::{CirclePoint, M31_CIRCLE_GEN, SECURE_FIELD_CIRCLE_ORDER};
    use stwo_prover::core::constraints::pair_vanishing;
    use stwo_prover::core::fields::qm31::QM31;

    #[test]
    fn test_eval_from_partial_evals() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a = get_rand_qm31(&mut prng);
        let b = get_rand_qm31(&mut prng);
        let c = get_rand_qm31(&mut prng);
        let d = get_rand_qm31(&mut prng);

        let mut res = a;
        res += b * QM31::from_u32_unchecked(0, 1, 0, 0);
        res += c * QM31::from_u32_unchecked(0, 0, 1, 0);
        res += d * QM31::from_u32_unchecked(0, 0, 0, 1);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let a_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(a)))
            .unwrap();
        let b_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(b)))
            .unwrap();
        let c_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(c)))
            .unwrap();
        let d_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(d)))
            .unwrap();

        let res_var = eval_from_partial_evals(&mut dsl, a_var, b_var, c_var, d_var).unwrap();

        assert_eq!(
            dsl.get_many_num(res_var).unwrap(),
            reformat_qm31_to_dsl_element(res)
        );

        dsl.set_program_output("qm31", res_var).unwrap();

        test_program(
            dsl,
            script! {
                { res }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_pair_vanishing() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let z = CirclePoint::get_point(prng.gen::<u128>() % SECURE_FIELD_CIRCLE_ORDER);

        let excluded0 = M31_CIRCLE_GEN.mul(prng.gen());
        let excluded1 = M31_CIRCLE_GEN.mul(prng.gen());

        let expected = pair_vanishing(excluded0.into_ef(), excluded1.into_ef(), z);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let z_x_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(z.x)))
            .unwrap();
        let z_y_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(z.y)))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = pair_vanishing_with_constant_m31_points(
            &mut dsl, table, z_x_var, z_y_var, excluded0, excluded1,
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
}
