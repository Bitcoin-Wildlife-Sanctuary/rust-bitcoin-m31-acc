use crate::m31::{M31Limbs, M31LimbsGadget, M31Mult, M31MultGadget};
use crate::utils::m31_to_limbs;
use anyhow::Result;
use bitcoin_circle_stark::treepp::pushable::{Builder, Pushable};
use bitcoin_circle_stark::treepp::*;
use rust_bitcoin_m31::{m31_add, m31_sub};

pub struct CM31MultGadget;

impl CM31MultGadget {
    /// Input:
    /// - CM31 element:
    /// -   a1, a2, a3, a4 (the real part)
    /// -   a5, a6, a7, a8 (the imaginary part)
    /// -   b1, b2, b3, b4 (the real part)
    /// -   b5, b6, b7, b8 (the imaginary part)
    pub fn mult(k: usize) -> Script {
        script! {
            // compute (b1, b2, b3, b4) + (b5, b6, b7, b8)
            // save to the altstack
            for _ in 0..8 {
                7 OP_PICK
            }
            { M31LimbsGadget::add_limbs() }
            for _ in 0..4 {
                OP_TOALTSTACK
            }

            // compute (a1, a2, a3, a4) + (a5, a6, a7, a8)
            for _ in 0..8 {
                15 OP_PICK
            }
            { M31LimbsGadget::add_limbs() }
            // pull the (b1, b2, b3, b4) + (b5, b6, b7, b8) back
            for _ in 0..4 {
                OP_FROMALTSTACK
            }

            // compute the corresponding c limbs and perform the reduction
            { M31MultGadget::compute_c_limbs(k + 4 * 4) }
            { M31MultGadget::reduce() }
            OP_TOALTSTACK

            // compute the imaginary part's product
            for _ in 0..4 {
                11 OP_ROLL
            }
            { M31MultGadget::compute_c_limbs(k + 2 * 4) }
            { M31MultGadget::reduce() }
            OP_TOALTSTACK

            // compute the real part's product
            { M31MultGadget::compute_c_limbs(k) }
            { M31MultGadget::reduce() }

            // stack: aR * bR
            // altstack: (aR + aI) * (bR + bI), aI * bI

            OP_FROMALTSTACK
            OP_2DUP
            m31_sub

            OP_ROT OP_ROT
            m31_add

            OP_FROMALTSTACK
            OP_SWAP
            m31_sub
        }
    }

    pub fn compute_hint(a: [u32; 2], b: [u32; 2]) -> Result<CM31MultHint> {
        let a_real = m31_to_limbs(a[0]);
        let a_imag = m31_to_limbs(a[1]);
        let b_real = m31_to_limbs(b[0]);
        let b_imag = m31_to_limbs(b[1]);

        let a_real_b_real = M31Mult::compute_c_limbs_from_limbs(&a_real, &b_real)?;
        let q3 = M31Mult::compute_q(&a_real_b_real)?;

        let a_imag_b_imag = M31Mult::compute_c_limbs_from_limbs(&a_imag, &b_imag)?;
        let q2 = M31Mult::compute_q(&a_imag_b_imag)?;

        let a_real_imag_sum = M31Limbs::add_limbs(&a_real, &a_imag);
        let b_real_imag_sum = M31Limbs::add_limbs(&b_real, &b_imag);
        let a_real_imag_b_real_imag =
            M31Mult::compute_c_limbs_from_limbs(&a_real_imag_sum, &b_real_imag_sum)?;
        let q1 = M31Mult::compute_q(&a_real_imag_b_real_imag)?;

        Ok(CM31MultHint { q1, q2, q3 })
    }
}

pub struct CM31MultHint {
    pub q1: i32,
    pub q2: i32,
    pub q3: i32,
}

impl Pushable for CM31MultHint {
    fn bitcoin_script_push(&self, mut builder: Builder) -> Builder {
        builder = self.q1.bitcoin_script_push(builder);
        builder = self.q2.bitcoin_script_push(builder);
        builder = self.q3.bitcoin_script_push(builder);
        builder
    }
}

#[cfg(test)]
mod test {
    use crate::cm31::CM31MultGadget;
    use crate::table::generate_table;
    use crate::utils::m31_to_limbs;
    use bitcoin_circle_stark::tests_utils::report::report_bitcoin_script_size;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_scriptexec::execute_script;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_mult() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        report_bitcoin_script_size("cm31", "mult", CM31MultGadget::mult(0).len());

        let table = generate_table::<9>();

        for i in 0..100 {
            let a_real = prng.gen_range(0u32..((1 << 31) - 1));
            let a_imag = prng.gen_range(0u32..((1 << 31) - 1));
            let b_real = prng.gen_range(0u32..((1 << 31) - 1));
            let b_imag = prng.gen_range(0u32..((1 << 31) - 1));

            let mut expected_real = (a_real as i64) * (b_real as i64) % ((1 << 31) - 1);
            expected_real += (1 << 31) - 1;
            expected_real -= (a_imag as i64) * (b_imag as i64) % ((1 << 31) - 1);
            expected_real %= (1 << 31) - 1;

            let mut expected_imag = (a_real as i64) * (b_imag as i64) % ((1 << 31) - 1);
            expected_imag += (b_real as i64) * (a_imag as i64) % ((1 << 31) - 1);
            expected_imag %= (1 << 31) - 1;

            let hint = CM31MultGadget::compute_hint([a_real, a_imag], [b_real, b_imag]).unwrap();

            let script = script! {
                { hint }
                { &table }
                for _ in 0..i {
                    { 1 }
                }
                { m31_to_limbs(a_real).to_vec() }
                { m31_to_limbs(a_imag).to_vec() }
                { m31_to_limbs(b_real).to_vec() }
                { m31_to_limbs(b_imag).to_vec() }
                { CM31MultGadget::mult(i) }
                { expected_imag } OP_EQUALVERIFY
                { expected_real } OP_EQUALVERIFY
                for _ in 0..i {
                    OP_DROP
                }
                for _ in 0..256 {
                    OP_2DROP
                }
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}
