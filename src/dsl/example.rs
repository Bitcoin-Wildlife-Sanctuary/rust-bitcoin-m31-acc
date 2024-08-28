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

use crate::utils::convert_qm31_from_limbs;
use anyhow::Result;
use bitcoin_script_dsl::dsl::DSL;
use bitcoin_script_dsl::functions::FunctionOutput;
use stwo_prover::core::fields::FieldExpOps;

/// Compute the parameters of `column_line_coeffs` without applying alpha.
///
/// Hint:
/// - `y_imag_inv`
///
/// Input:
/// - `table`, by reference
/// - `p.y, f1(p), f2(p), ..., fn(p)`, all of which are QM31
///
/// Output:
/// - `(a1, b1), (a2, b2), (a3, b3), ..., (an, bn)`
/// where all of them are cm31.
/// - `ai = Im(f(P)) / Im(p.y)`
/// - `bi = Im(f(P)) / Im(p.y) Re(p.y) - Re(f(P))`
///
pub fn column_line_coeffs(
    num_columns: usize,
    dsl: &mut DSL,
    inputs: &[usize],
) -> Result<FunctionOutput> {
    let y = dsl.get_many_num(inputs[1])?;
    let y_qm31 = convert_qm31_from_limbs(y);

    let y_imag_inv = y_qm31.1.inverse();

    todo!()
}
