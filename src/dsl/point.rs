use crate::dsl::qm31::{
    qm31_mul_m31_limbs, reformat_qm31_from_dsl_element, reformat_qm31_to_dsl_element,
};
use crate::utils::convert_m31_to_limbs;
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, DSL};
use num_traits::One;
use std::ops::{Add, Mul, Neg};
use stwo_prover::core::circle::CirclePoint;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::QM31;
use stwo_prover::core::fields::{Field, FieldExpOps};

pub fn get_random_point(
    dsl: &mut DSL,
    table: usize,
    channel_digest: usize,
) -> Result<(usize, usize, usize)> {
    // compute the results
    let res_draw_felt = dsl.execute("draw_felt", &[channel_digest])?;
    let new_channel_digest = res_draw_felt[0];
    let t_var = res_draw_felt[1];

    let t = reformat_qm31_from_dsl_element(dsl.get_many_num(t_var)?);
    let one_plus_tsquared_inv = t.square().add(QM31::one()).inverse();

    let x = QM31::one().add(t.square().neg()).mul(one_plus_tsquared_inv);
    let y = t.double().mul(one_plus_tsquared_inv);

    let x_var = dsl.alloc_hint("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(x)))?;
    let y_var = dsl.alloc_hint("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(y)))?;

    // check the correctness
    let t_limbs = dsl.execute("qm31_to_limbs", &[t_var])?[0];
    let t_squared = dsl.execute("qm31_limbs_mul", &[table, t_limbs, t_limbs])?[0];
    let t_squared_minus_1 = dsl.execute("qm31_1sub", &[t_squared])?[0];
    let t_squared_plus_1 = dsl.execute("qm31_1add", &[t_squared])?[0];

    let x_limbs = dsl.execute("qm31_to_limbs", &[x_var])?[0];
    let t_squared_plus_1_limbs = dsl.execute("qm31_to_limbs", &[t_squared_plus_1])?[0];
    let should_be_1_minus_t_squared =
        dsl.execute("qm31_limbs_mul", &[table, x_limbs, t_squared_plus_1_limbs])?[0];
    let should_be_t_squared_minus_1 = dsl.execute("qm31_neg", &[should_be_1_minus_t_squared])?[0];

    let _ = dsl.execute(
        "qm31_equalverify",
        &[should_be_t_squared_minus_1, t_squared_minus_1],
    )?;

    let y_limbs = dsl.execute("qm31_to_limbs", &[y_var])?[0];
    let should_be_2t = dsl.execute("qm31_limbs_mul", &[table, y_limbs, t_squared_plus_1_limbs])?[0];

    let double_t_var = dsl.execute("qm31_add", &[t_var, t_var])?[0];

    let _ = dsl.execute("qm31_equalverify", &[should_be_2t, double_t_var])?;

    Ok((new_channel_digest, x_var, y_var))
}

pub fn add_constant_m31_point(
    dsl: &mut DSL,
    table: usize,
    current_x: usize,
    current_y: usize,
    constant: CirclePoint<M31>,
) -> Result<(usize, usize)> {
    // new x: x0 · x1 − y0 · y1
    // new y: x0 · y1 + y0 · x1
    // use Karatsuba

    let x1_limbs = dsl.alloc_constant(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(constant.x.0).to_vec()),
    )?;
    let x0x1 = qm31_mul_m31_limbs(dsl, table, current_x, x1_limbs)?;

    let y1_limbs = dsl.alloc_constant(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(constant.y.0).to_vec()),
    )?;
    let y0y1 = qm31_mul_m31_limbs(dsl, table, current_y, y1_limbs)?;

    let x0_plus_y0 = dsl.execute("qm31_add", &[current_x, current_y])?[0];
    let x1_plus_y1 = constant.x + constant.y;
    let x1_plus_y1_limbs = dsl.alloc_constant(
        "m31_limbs",
        Element::ManyNum(convert_m31_to_limbs(x1_plus_y1.0).to_vec()),
    )?;

    let all_terms = qm31_mul_m31_limbs(dsl, table, x0_plus_y0, x1_plus_y1_limbs)?;
    let mut cross_terms = dsl.execute("qm31_sub", &[all_terms, x0x1])?[0];
    cross_terms = dsl.execute("qm31_sub", &[cross_terms, y0y1])?[0];

    let new_x = dsl.execute("qm31_sub", &[x0x1, y0y1])?[0];
    let new_y = cross_terms;

    Ok((new_x, new_y))
}

#[cfg(test)]
mod test {
    use crate::dsl::point::{add_constant_m31_point, get_random_point};
    use crate::dsl::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::channel::Sha256Channel;
    use stwo_prover::core::circle::{CirclePoint, M31_CIRCLE_GEN, SECURE_FIELD_CIRCLE_GEN};
    use stwo_prover::core::vcs::sha256_hash::Sha256Hash;

    #[test]
    fn test_get_random_point() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let mut init_state = [0u8; 32];
        init_state.iter_mut().for_each(|v| *v = prng.gen());

        let init_state = Sha256Hash::from(init_state.to_vec());

        let mut channel = Sha256Channel::default();
        channel.update_digest(init_state);

        let oods_res = CirclePoint::get_random_point(&mut channel);

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let old_channel_digest_var = dsl
            .alloc_input("hash", Element::Str(init_state.as_ref().to_vec()))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = get_random_point(&mut dsl, table, old_channel_digest_var).unwrap();
        dsl.set_program_output("hash", res.0).unwrap();
        dsl.set_program_output("qm31", res.1).unwrap();
        dsl.set_program_output("qm31", res.2).unwrap();

        test_program(
            dsl,
            script! {
                { channel.digest() }
                { oods_res.x }
                { oods_res.y }
            },
        )
        .unwrap()
    }

    #[test]
    fn test_add_constant_m31_point() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let current_point = SECURE_FIELD_CIRCLE_GEN.mul(prng.gen());
        let constant_point = M31_CIRCLE_GEN.mul(prng.gen());

        let expected = current_point + constant_point.into_ef();

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let current_point_x = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(current_point.x)),
            )
            .unwrap();
        let current_point_y = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(current_point.y)),
            )
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let (res_x, res_y) = add_constant_m31_point(
            &mut dsl,
            table,
            current_point_x,
            current_point_y,
            constant_point,
        )
        .unwrap();

        assert_eq!(
            dsl.get_many_num(res_x).unwrap(),
            reformat_qm31_to_dsl_element(expected.x)
        );
        assert_eq!(
            dsl.get_many_num(res_y).unwrap(),
            reformat_qm31_to_dsl_element(expected.y)
        );

        dsl.set_program_output("qm31", res_x).unwrap();
        dsl.set_program_output("qm31", res_y).unwrap();

        test_program(
            dsl,
            script! {
                { expected.x }
                { expected.y }
            },
        )
        .unwrap()
    }
}
