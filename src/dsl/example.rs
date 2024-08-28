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
use bitcoin_script_dsl::functions::FunctionOutput;

pub fn column_line_coeffs_with_hint(
    num_columns: usize,
    dsl: &mut DSL,
    inputs: &[usize],
) -> Result<FunctionOutput> {
    todo!()
}
