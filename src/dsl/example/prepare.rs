// Fibonacci Prepare Gadget
//
// Input:
// - points
//   * 3 masked points
//   * oods point
// - trace oods values
//   * 3 values
// - composition oods values
//   * 4 values
// - random_coeffs2
//
// Output:
//    (a, b), (a, b), (a, b) for trace (3 * 2 cm31 elements)
//    (a, b), (a, b), (a, b), (a, b) for composition (4 * 2 cm31 elements)
//    prepared points
//    coeff^6, coeff^5, coeff^4, coeff^3, coeff^2, coeff

use anyhow::Result;
use bitcoin_script_dsl::dsl::DSL;

/// Compute the parameters of `column_line_coeffs` without applying alpha.
///
/// Input:
/// - `table`, by reference
/// - `p.y, f1(p), f2(p), ..., fn(p)`, all of which are QM31 (not yet decomposed)
///
/// Output:
/// - `(a1, b1), (a2, b2), (a3, b3), ..., (an, bn)`
/// where all of them are cm31.
/// - `ai = Im(f(P)) / Im(p.y)`
/// - `bi = Im(f(P)) / Im(p.y) Re(p.y) - Re(f(P))`
///
pub fn column_line_coeffs(
    dsl: &mut DSL,
    table: usize,
    y: usize,
    evals: &[usize],
) -> Result<Vec<(usize, usize)>> {
    let y_imag = dsl.execute("qm31_second", &[y])?[0];
    let y_imag_limbs = dsl.execute("cm31_to_limbs", &[y_imag])?[0];
    let y_imag_inv_limbs = dsl.execute("cm31_limbs_inverse", &[table, y_imag_limbs])?[0];

    let y_real = dsl.execute("qm31_first", &[y])?[0];
    let y_real_limbs = dsl.execute("cm31_to_limbs", &[y_real])?[0];
    let y_real_times_y_imag_inv =
        dsl.execute("cm31_limbs_mul", &[table, y_real_limbs, y_imag_inv_limbs])?[0];
    let y_real_times_y_imag_inv_limbs =
        dsl.execute("cm31_to_limbs", &[y_real_times_y_imag_inv])?[0];

    let mut ab = vec![];

    for &eval in evals.iter() {
        let eval_imag = dsl.execute("qm31_second", &[eval])?[0];
        let eval_imag_limbs = dsl.execute("cm31_to_limbs", &[eval_imag])?[0];

        let eval_real = dsl.execute("qm31_first", &[eval])?[0];

        let a = dsl.execute(
            "cm31_limbs_mul",
            &[table, eval_imag_limbs, y_imag_inv_limbs],
        )?[0];
        let mut b = dsl.execute(
            "cm31_limbs_mul",
            &[table, eval_imag_limbs, y_real_times_y_imag_inv_limbs],
        )?[0];
        b = dsl.execute("cm31_sub", &[b, eval_real])?[0];

        ab.push((a, b));
    }

    Ok(ab)
}

pub fn prepare_pair_vanishing(
    dsl: &mut DSL,
    table: usize,
    x: usize,
    y: usize,
) -> Result<(usize, usize)> {
    // note: there are some overlapping regarding the extraction of `y_imag` and `y_real` between
    // this function and `column_line_coeffs` and they can be combined.

    let y_imag = dsl.execute("qm31_second", &[y])?[0];
    let y_imag_limbs = dsl.execute("cm31_to_limbs", &[y_imag])?[0];
    let y_imag_inv_limbs = dsl.execute("cm31_limbs_inverse", &[table, y_imag_limbs])?[0];

    let x_imag = dsl.execute("qm31_second", &[x])?[0];
    let x_imag_limbs = dsl.execute("cm31_to_limbs", &[x_imag])?[0];

    let x_imag_div_y_imag =
        dsl.execute("cm31_limbs_mul", &[table, x_imag_limbs, y_imag_inv_limbs])?[0];
    let x_imag_div_y_imag_limbs = dsl.execute("cm31_to_limbs", &[x_imag_div_y_imag])?[0];

    let y_real = dsl.execute("qm31_first", &[y])?[0];
    let y_real_limbs = dsl.execute("cm31_to_limbs", &[y_real])?[0];

    let mut cross_term = dsl.execute(
        "cm31_limbs_mul",
        &[table, x_imag_div_y_imag_limbs, y_real_limbs],
    )?[0];
    let x_real = dsl.execute("qm31_first", &[x])?[0];
    cross_term = dsl.execute("cm31_sub", &[cross_term, x_real])?[0];

    Ok((x_imag_div_y_imag, cross_term))
}

pub fn power_alpha_six(dsl: &mut DSL, table: usize, alpha: usize) -> Result<Vec<usize>> {
    let alpha_limbs = dsl.execute("qm31_to_limbs", &[alpha])?[0];
    let alpha_2 = dsl.execute("qm31_limbs_mul", &[table, alpha_limbs, alpha_limbs])?[0];
    let alpha_2_limbs = dsl.execute("qm31_to_limbs", &[alpha_2])?[0];
    let alpha_4 = dsl.execute("qm31_limbs_mul", &[table, alpha_2_limbs, alpha_2_limbs])?[0];
    let alpha_4_limbs = dsl.execute("qm31_to_limbs", &[alpha_4])?[0];

    let alpha_3 = dsl.execute("qm31_limbs_mul", &[table, alpha_limbs, alpha_2_limbs])?[0];
    let alpha_5 = dsl.execute("qm31_limbs_mul", &[table, alpha_limbs, alpha_4_limbs])?[0];
    let alpha_6 = dsl.execute("qm31_limbs_mul", &[table, alpha_2_limbs, alpha_4_limbs])?[0];

    Ok(vec![alpha_6, alpha_5, alpha_4, alpha_3, alpha_2, alpha])
}

#[cfg(test)]
mod test {
    use crate::dsl::example::prepare::{
        column_line_coeffs, power_alpha_six, prepare_pair_vanishing,
    };
    use crate::dsl::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::{load_data_types, load_functions};
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use itertools::Itertools;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::fields::FieldExpOps;

    #[test]
    fn test_column_line_coeffs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let y = get_rand_qm31(&mut prng);

        let evals = (0..4).map(|_| get_rand_qm31(&mut prng)).collect_vec();

        let expected = {
            let y_real = y.0;
            let y_imag_inverse = y.1.inverse();

            let mut result = vec![];

            for eval in evals.iter() {
                let eval_imag = eval.1;
                let eval_real = eval.0;

                let fp_imag_div_y_imag = eval_imag * y_imag_inverse;
                let cross_term = y_real * fp_imag_div_y_imag - eval_real;

                result.push((fp_imag_div_y_imag, cross_term))
            }

            result
        };

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let y_val = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(y)))
            .unwrap();

        let mut evals_val = vec![];
        for &eval in evals.iter() {
            evals_val.push(
                dsl.alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(eval)))
                    .unwrap(),
            );
        }

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = column_line_coeffs(&mut dsl, table, y_val, &evals_val).unwrap();

        for res_entry in res.iter() {
            dsl.set_program_output("cm31", res_entry.0).unwrap();
            dsl.set_program_output("cm31", res_entry.1).unwrap();
        }

        test_program(
            dsl,
            script! {
                for expected_entry in expected.iter() {
                    { expected_entry.0 }
                    { expected_entry.1 }
                }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_prepare_pair_vanishing() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let x = get_rand_qm31(&mut prng);
        let y = get_rand_qm31(&mut prng);

        let expected = {
            let x_imag_div_y_imag = x.1 * y.1.inverse();
            let cross_term = x_imag_div_y_imag * y.0 - x.0;

            (x_imag_div_y_imag, cross_term)
        };

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let x_val = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(x)))
            .unwrap();
        let y_val = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(y)))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let res = prepare_pair_vanishing(&mut dsl, table, x_val, y_val).unwrap();

        dsl.set_program_output("cm31", res.0).unwrap();
        dsl.set_program_output("cm31", res.1).unwrap();

        test_program(
            dsl,
            script! {
                { expected.0 }
                { expected.1 }
            },
        )
        .unwrap();
    }

    #[test]
    fn test_power_alpha_six() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let alpha = get_rand_qm31(&mut prng);

        let expected = {
            [
                alpha.pow(6),
                alpha.pow(5),
                alpha.pow(4),
                alpha.pow(3),
                alpha.pow(2),
                alpha,
            ]
        };

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let alpha_val = dsl
            .alloc_input(
                "qm31",
                Element::ManyNum(reformat_qm31_to_dsl_element(alpha)),
            )
            .unwrap();
        let table = dsl.execute("push_table", &[]).unwrap()[0];
        let alpha_powers = power_alpha_six(&mut dsl, table, alpha_val).unwrap();

        for &alpha_power in alpha_powers.iter() {
            dsl.set_program_output("qm31", alpha_power).unwrap();
        }

        test_program(
            dsl,
            script! {
                for expected_entry in expected.iter() {
                    { *expected_entry }
                }
            },
        )
        .unwrap();
    }
}
