use crate::treepp::*;
use crate::utils::OP_256MUL;
use stwo_prover::core::fields::cm31::CM31;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::QM31;

pub trait Decompose {
    fn decompose_to_limbs(&self) -> Vec<i32>;
}

impl Decompose for M31 {
    #[inline(always)]
    fn decompose_to_limbs(&self) -> Vec<i32> {
        let v = self.0;
        vec![
            (v & 0xff) as i32,
            ((v >> 8) & 0xff) as i32,
            ((v >> 16) & 0xff) as i32,
            ((v >> 24) & 0xff) as i32,
        ]
    }
}

impl Decompose for CM31 {
    fn decompose_to_limbs(&self) -> Vec<i32> {
        let mut res = self.0.decompose_to_limbs();
        res.extend(self.1.decompose_to_limbs());
        res
    }
}

impl Decompose for QM31 {
    fn decompose_to_limbs(&self) -> Vec<i32> {
        let mut res = self.0.decompose_to_limbs();
        res.extend(self.1.decompose_to_limbs());
        res
    }
}

pub struct DecomposeGadget;

impl DecomposeGadget {
    // Input:
    //   4 limbs representing an M31
    //
    // Output:
    //   the corresponding M31 element (if the limbs are valid)
    pub fn recompose() -> Script {
        script! {
            OP_DUP 128 OP_LESSTHAN OP_VERIFY
            OP_256MUL

            OP_SWAP
            OP_DUP 256 OP_LESSTHAN OP_VERIFY
            OP_ADD
            OP_256MUL

            OP_SWAP
            OP_DUP 256 OP_LESSTHAN OP_VERIFY
            OP_ADD
            OP_256MUL

            OP_SWAP
            OP_DUP 256 OP_LESSTHAN OP_VERIFY
            OP_ADD
        }
    }
}
