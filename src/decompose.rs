use crate::utils::OP_256MUL;
use bitcoin_circle_stark::treepp::*;
use rust_bitcoin_m31::{cm31_fromaltstack, cm31_swap, cm31_toaltstack};
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
    pub fn recompose_m31() -> Script {
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

    // Input:
    //   8 limbs representing a CM31
    //
    // Output:
    //   the corresponding CM31 element (taking 2 elements)
    pub fn recompose_cm31() -> Script {
        script! {
            { Self::recompose_m31() }
            OP_TOALTSTACK
            { Self::recompose_m31() }
            OP_FROMALTSTACK

            // the ordering is different between limbs and the CM31
            OP_SWAP
        }
    }

    // Input:
    //   16 limbs representing a QM31
    //
    // Output:
    //   the corresponding QM31 element (taking 4 elements)
    pub fn recompose_qm31() -> Script {
        script! {
            { Self::recompose_cm31() }
            cm31_toaltstack
            { Self::recompose_cm31() }
            cm31_fromaltstack

            // the ordering is different between limbs and the QM31
            cm31_swap
        }
    }
}

#[cfg(test)]
mod test {
    use crate::decompose::DecomposeGadget;
    use crate::utils::convert_m31_to_limbs;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_scriptexec::execute_script;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use rust_bitcoin_m31::{cm31_equalverify, qm31_equalverify};

    #[test]
    fn test_recompose_m31() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a = prng.gen_range(0..((1 << 31) - 1));
            let a_limbs = convert_m31_to_limbs(a);

            let script = script! {
                { a_limbs.to_vec() }
                { DecomposeGadget::recompose_m31() }
                { a }
                OP_EQUAL
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_recompose_cm31() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a_real = prng.gen_range(0..((1 << 31) - 1));
            let a_imag = prng.gen_range(0..((1 << 31) - 1));

            let a_real_limbs = convert_m31_to_limbs(a_real);
            let a_imag_limbs = convert_m31_to_limbs(a_imag);

            let script = script! {
                { a_real_limbs.to_vec() }
                { a_imag_limbs.to_vec() }
                { DecomposeGadget::recompose_cm31() }
                { a_imag }
                { a_real }
                cm31_equalverify
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_recompose_qm31() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a_first_real = prng.gen_range(0..((1 << 31) - 1));
            let a_first_imag = prng.gen_range(0..((1 << 31) - 1));
            let a_second_real = prng.gen_range(0..((1 << 31) - 1));
            let a_second_imag = prng.gen_range(0..((1 << 31) - 1));

            let a_first_real_limbs = convert_m31_to_limbs(a_first_real);
            let a_first_imag_limbs = convert_m31_to_limbs(a_first_imag);
            let a_second_real_limbs = convert_m31_to_limbs(a_second_real);
            let a_second_imag_limbs = convert_m31_to_limbs(a_second_imag);

            let script = script! {
                { a_first_real_limbs.to_vec() }
                { a_first_imag_limbs.to_vec() }
                { a_second_real_limbs.to_vec() }
                { a_second_imag_limbs.to_vec() }
                { DecomposeGadget::recompose_qm31() }
                { a_second_imag }
                { a_second_real }
                { a_first_imag }
                { a_first_real }
                qm31_equalverify
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}
