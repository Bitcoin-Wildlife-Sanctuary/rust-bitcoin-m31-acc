use anyhow::Result;
use bitcoin_script_dsl::dsl::DSL;

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

#[cfg(test)]
mod test {
    use crate::dsl::example::fiat_shamir::eval_from_partial_evals;
    use crate::dsl::load_data_types;
    use crate::dsl::load_functions;
    use crate::dsl::qm31::reformat_qm31_to_dsl_element;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
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
}
