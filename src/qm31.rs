use crate::cm31::{CM31Limbs, CM31LimbsGadget, CM31Mult, CM31MultGadget, CM31MultHint};
use crate::treepp::pushable::{Builder, Pushable};
use crate::treepp::*;
use crate::utils::convert_cm31_to_limbs;
use anyhow::Result;
use rust_bitcoin_m31::{
    cm31_add, cm31_copy, cm31_double, cm31_dup, cm31_fromaltstack, cm31_rot, cm31_sub, cm31_swap,
    cm31_toaltstack, m31_neg,
};

pub struct QM31MultGadget;

impl QM31MultGadget {
    // Input:
    // - QM31 elements:
    //   a1...a8
    //   a9...a16
    //   b1...b8
    //   b9...b16
    //
    // Note that for QM31, say
    //   (a + ib) + j(c + id)
    //   (e + if) + j(g + ih)
    //
    // The product is:
    //      [(a + ib) (e + if) + (2 + i) (c + id) (g + ih)]
    //  + j [(a + ib)(g + ih) + (e + if)(c + id)]
    //
    // (a + ib) (e + if) + (2 + i) (c + id) (g + ih)
    pub fn mult(k: usize) -> Script {
        script! {
            // compute (b1...b8) + (b9...b16)
            for _ in 0..16 {
                15 OP_PICK
            }
            { CM31LimbsGadget::add_limbs() }
            for _ in 0..8 {
                OP_TOALTSTACK
            }

            // compute (a1...a8) + (a9...a16)
            for _ in 0..16 {
                31 OP_PICK
            }
            { CM31LimbsGadget::add_limbs() }
            // pull the (b1...b8) + (b9...b16) back
            for _ in 0..8 {
                OP_FROMALTSTACK
            }

            // compute the corresponding cm31 (2 elements)
            { CM31MultGadget::mult(k + 8 * 4) }
            cm31_toaltstack

            // compute the j part's product
            for _ in 0..8 {
                23 OP_ROLL
            }
            { CM31MultGadget::mult(k + 8 * 2) }
            cm31_toaltstack

            // compute the non-j part's product
            { CM31MultGadget::mult(k) }

            // stack:
            //  r1 r2
            // altstack:
            //  m1 m2
            //  i1 i2

            cm31_fromaltstack
            cm31_dup
            { cm31_copy(2) }
            cm31_add
            cm31_toaltstack

            // stack:
            //  r1 r2
            //  i1 i2
            // altstack:
            //  m1 m2
            //  s1 s2

            cm31_dup
            cm31_double
            cm31_rot
            cm31_add
            cm31_swap
            OP_SWAP m31_neg
            cm31_add

            // stack:
            //  (r1 r2) + (2 + i) (i1 i2)
            // altstack:
            //  m1 m2
            //  s1 s2

            cm31_fromaltstack
            cm31_fromaltstack
            cm31_swap
            cm31_sub

            // follow the qm31 format: j first, non-j second
            cm31_swap
        }
    }
}

pub struct QM31Mult;

impl QM31Mult {
    pub fn compute_hint_from_limbs(
        a_first: &[i32],
        a_second: &[i32],
        b_first: &[i32],
        b_second: &[i32],
    ) -> Result<QM31MultHint> {
        assert_eq!(a_first.len(), 8);
        assert_eq!(a_second.len(), 8);
        assert_eq!(b_first.len(), 8);
        assert_eq!(b_second.len(), 8);

        let a_first_b_first = CM31Mult::compute_hint_from_limbs(
            &a_first[0..4],
            &a_first[4..8],
            &b_first[0..4],
            &b_first[4..8],
        )?;

        let a_second_b_second = CM31Mult::compute_hint_from_limbs(
            &a_second[0..4],
            &a_second[4..8],
            &b_second[0..4],
            &b_second[4..8],
        )?;

        let a_first_second_sum = CM31Limbs::add_limbs(&a_first, &a_second);
        let b_first_second_sum = CM31Limbs::add_limbs(&b_first, &b_second);
        let a_sum_b_sum = CM31Mult::compute_hint_from_limbs(
            &a_first_second_sum[0..4],
            &a_first_second_sum[4..8],
            &b_first_second_sum[0..4],
            &b_first_second_sum[4..8],
        )?;

        Ok(QM31MultHint {
            h1: a_sum_b_sum,
            h2: a_second_b_second,
            h3: a_first_b_first,
        })
    }

    pub fn compute_hint(a: &[u32], b: &[u32]) -> Result<QM31MultHint> {
        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 4);

        let a_first = convert_cm31_to_limbs(a[0], a[1]);
        let a_second = convert_cm31_to_limbs(a[2], a[3]);
        let b_first = convert_cm31_to_limbs(b[0], b[1]);
        let b_second = convert_cm31_to_limbs(b[2], b[3]);

        Self::compute_hint_from_limbs(&a_first, &a_second, &b_first, &b_second)
    }
}

pub struct QM31MultHint {
    pub h1: CM31MultHint,
    pub h2: CM31MultHint,
    pub h3: CM31MultHint,
}

impl Pushable for QM31MultHint {
    fn bitcoin_script_push(&self, mut builder: Builder) -> Builder {
        builder = self.h1.bitcoin_script_push(builder);
        builder = self.h2.bitcoin_script_push(builder);
        builder = self.h3.bitcoin_script_push(builder);
        builder
    }
}

#[cfg(test)]
mod test {
    use crate::qm31::{QM31Mult, QM31MultGadget};
    use crate::report_bitcoin_script_size;
    use crate::table::get_table;
    use crate::treepp::*;
    use crate::utils::convert_m31_to_limbs;
    use bitcoin_scriptexec::execute_script;
    use itertools::Itertools;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use rust_bitcoin_m31::qm31_equalverify;
    use stwo_prover::core::fields::qm31::QM31;

    #[test]
    fn test_mult() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        report_bitcoin_script_size("qm31", "mult", QM31MultGadget::mult(0).len());

        let table = get_table();

        for i in 0..100 {
            let a = (0..4)
                .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
                .collect_vec();
            let b = (0..4)
                .map(|_| prng.gen_range(0u32..((1 << 31) - 1)))
                .collect_vec();

            let a_qm31 = QM31::from_u32_unchecked(a[0], a[1], a[2], a[3]);
            let b_qm31 = QM31::from_u32_unchecked(b[0], b[1], b[2], b[3]);

            let expected = a_qm31 * b_qm31;

            let hint = QM31Mult::compute_hint(&a, &b).unwrap();

            let script = script! {
                { hint }
                { table }
                for _ in 0..i {
                    { 1 }
                }
                for &a_element in a.iter() {
                    { convert_m31_to_limbs(a_element).to_vec() }
                }
                for &b_element in b.iter() {
                    { convert_m31_to_limbs(b_element).to_vec() }
                }
                { QM31MultGadget::mult(i) }
                { expected.1.1.0 }
                { expected.1.0.0 }
                { expected.0.1.0 }
                { expected.0.0.0 }
                qm31_equalverify
                for _ in 0..i {
                    OP_DROP
                }
                for _ in 0..256 {
                    OP_2DROP
                }
                OP_DROP
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}
