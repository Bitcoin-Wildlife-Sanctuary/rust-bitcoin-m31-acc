use crate::algorithms::utils::convert_m31_to_limbs;
use crate::dsl::building_blocks::point::{add_constant_m31_point_x_only, point_double_x};
use crate::dsl::building_blocks::qm31::qm31_mul_m31_limbs;
use crate::dsl::framework::dsl::{Element, DSL};
use anyhow::Result;
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

pub fn step_constraint_numerator_evaluation(
    dsl: &mut DSL,
    table: usize,
    f_z: usize,
    f_gz: usize,
    f_g2z: usize,
    z_x: usize,
    z_y: usize,
    log_size: u32,
) -> Result<usize> {
    let constraint_zero_domain = Coset::subgroup(log_size);

    let f_z_limbs = dsl.execute("qm31_to_limbs", &[f_z])?[0];
    let f_z_squared = dsl.execute("qm31_limbs_mul", &[table, f_z_limbs, f_z_limbs])?[0];

    let f_gz_limbs = dsl.execute("qm31_to_limbs", &[f_gz])?[0];
    let f_gz_squared = dsl.execute("qm31_limbs_mul", &[table, f_gz_limbs, f_gz_limbs])?[0];

    let mut poly = dsl.execute("qm31_add", &[f_z_squared, f_gz_squared])?[0];
    poly = dsl.execute("qm31_sub", &[poly, f_g2z])?[0];

    let pair_vanisher = pair_vanishing_with_constant_m31_points(
        dsl,
        table,
        z_x,
        z_y,
        constraint_zero_domain.at(constraint_zero_domain.size() - 2),
        constraint_zero_domain.at(constraint_zero_domain.size() - 1),
    )?;

    let poly_limbs = dsl.execute("qm31_to_limbs", &[poly])?[0];
    let pair_vanisher_limbs = dsl.execute("qm31_to_limbs", &[pair_vanisher])?[0];
    let numerator = dsl.execute("qm31_limbs_mul", &[table, poly_limbs, pair_vanisher_limbs])?[0];

    Ok(numerator)
}

pub fn step_constraint_denominator_inverse_evaluation(
    dsl: &mut DSL,
    table: usize,
    z_x: usize,
    z_y: usize,
    log_size: u32,
) -> Result<usize> {
    let constraint_zero_domain = Coset::subgroup(log_size);
    let denominator = coset_vanishing(dsl, table, z_x, z_y, constraint_zero_domain)?;

    let denominator_limbs = dsl.execute("qm31_to_limbs", &[denominator])?[0];
    let denominator_limbs_inverse =
        dsl.execute("qm31_limbs_inverse", &[table, denominator_limbs])?[0];

    Ok(denominator_limbs_inverse)
}

pub fn boundary_constraint_evaluation(
    dsl: &mut DSL,
    table: usize,
    f_z: usize,
    z_x: usize,
    z_y: usize,
    log_size: u32,
    claim: M31,
) -> Result<usize> {
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
    let denominator_limbs_inverse =
        dsl.execute("qm31_limbs_inverse", &[table, denominator_limbs])?[0];

    let res = dsl.execute(
        "qm31_limbs_mul",
        &[table, numerator_limbs, denominator_limbs_inverse],
    )?[0];

    Ok(res)
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

pub fn coset_vanishing(
    dsl: &mut DSL,
    table: usize,
    z_x: usize,
    z_y: usize,
    coset: Coset,
) -> Result<usize> {
    let shift = -coset.initial + coset.step_size.half().to_point();
    let mut res = add_constant_m31_point_x_only(dsl, table, z_x, z_y, shift)?;

    for _ in 1..coset.log_size {
        res = point_double_x(dsl, table, res)?;
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::algorithms::utils::convert_qm31_to_limbs;
    use crate::dsl::building_blocks::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::framework::dsl::{Element, DSL};
    use crate::dsl::framework::test_program;
    use crate::dsl::load_data_types;
    use crate::dsl::load_functions;
    use crate::dsl::modules::fiat_shamir::{
        boundary_constraint_evaluation, coset_vanishing, eval_from_partial_evals,
        pair_vanishing_with_constant_m31_points, step_constraint_denominator_inverse_evaluation,
        step_constraint_numerator_evaluation,
    };
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use rand::{Rng, RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::circle::{
        CirclePoint, Coset, M31_CIRCLE_GEN, SECURE_FIELD_CIRCLE_GEN, SECURE_FIELD_CIRCLE_ORDER,
    };
    use stwo_prover::core::constraints::pair_vanishing;
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::fields::qm31::QM31;
    use stwo_prover::core::fields::FieldExpOps;
    use stwo_prover::examples::fibonacci::Fibonacci;

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

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

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

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

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

    #[test]
    fn test_coset_vanishing() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let z = CirclePoint::get_point(prng.gen::<u128>() % SECURE_FIELD_CIRCLE_ORDER);

        let coset = Coset::subgroup(10);

        let expected = stwo_prover::core::constraints::coset_vanishing(coset, z);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let z_x_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(z.x)))
            .unwrap();
        let z_y_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(z.y)))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = coset_vanishing(&mut dsl, table, z_x_var, z_y_var, coset).unwrap();

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
    fn test_eval_composition_polynomial_at_point() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let point = SECURE_FIELD_CIRCLE_GEN.mul(prng.gen());
        let mask = [
            get_rand_qm31(&mut prng),
            get_rand_qm31(&mut prng),
            get_rand_qm31(&mut prng),
        ];

        let fibonacci_component = Fibonacci::new(10, BaseField::reduce(prng.next_u64()));

        let step_constraint_evaluation_expected = {
            let constraint_zero_domain = Coset::subgroup(10);
            let constraint_value = mask[0].square() + mask[1].square() - mask[2];
            let selector = pair_vanishing(
                constraint_zero_domain
                    .at(constraint_zero_domain.size() - 2)
                    .into_ef(),
                constraint_zero_domain
                    .at(constraint_zero_domain.size() - 1)
                    .into_ef(),
                point,
            );
            let num = constraint_value * selector;
            let denom =
                stwo_prover::core::constraints::coset_vanishing(constraint_zero_domain, point);

            (num, denom)
        };
        let boundary_constraint_evaluation_expected = fibonacci_component
            .air
            .component
            .boundary_constraint_eval_quotient_by_mask(point, &[mask[0]]);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let f_z_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(mask[0])),
            )
            .unwrap();
        let f_gz_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(mask[1])),
            )
            .unwrap();
        let f_g2z_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(mask[2])),
            )
            .unwrap();
        let z_x_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(point.x)),
            )
            .unwrap();
        let z_y_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(point.y)),
            )
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = step_constraint_numerator_evaluation(
            &mut dsl, table, f_z_var, f_gz_var, f_g2z_var, z_x_var, z_y_var, 10,
        )
        .unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(step_constraint_evaluation_expected.0)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { step_constraint_evaluation_expected.0 }
            },
        )
        .unwrap();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let z_x_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(point.x)),
            )
            .unwrap();
        let z_y_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(point.y)),
            )
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res =
            step_constraint_denominator_inverse_evaluation(&mut dsl, table, z_x_var, z_y_var, 10)
                .unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            convert_qm31_to_limbs(step_constraint_evaluation_expected.1.inverse())
        );

        dsl.set_program_output("qm31_limbs", res).unwrap();

        test_program(
            dsl,
            script! {
                { convert_qm31_to_limbs(step_constraint_evaluation_expected.1.inverse()).to_vec() }
            },
        )
        .unwrap();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl).unwrap();
        load_functions(&mut dsl).unwrap();

        let f_z_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(mask[0])),
            )
            .unwrap();
        let z_x_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(point.x)),
            )
            .unwrap();
        let z_y_var = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(point.y)),
            )
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = boundary_constraint_evaluation(
            &mut dsl,
            table,
            f_z_var,
            z_x_var,
            z_y_var,
            10,
            fibonacci_component.air.component.claim,
        )
        .unwrap();

        assert_eq!(
            dsl.get_many_num(res).unwrap(),
            reformat_qm31_to_dsl_element(boundary_constraint_evaluation_expected)
        );

        dsl.set_program_output("qm31", res).unwrap();

        test_program(
            dsl,
            script! {
                { boundary_constraint_evaluation_expected }
            },
        )
        .unwrap();
    }
}
