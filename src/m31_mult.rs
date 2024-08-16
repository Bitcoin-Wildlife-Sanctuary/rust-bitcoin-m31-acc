use bitcoin_circle_stark::OP_HINT;
use bitcoin_circle_stark::treepp::pushable::{Builder, Pushable};
use bitcoin_circle_stark::treepp::*;
use crate::lookup_8bit::Lookup8BitGadget;

pub struct M31MultGadget;

impl M31MultGadget {
    pub fn compute_hint(pairs: &[(u32, u32)]) -> M31MultHint {
        let ab: u64 = pairs.iter().map(|(a, b)| (*a as u64) * (*b as u64))
            .sum();

        let q = ab / ((1 << 31) - 1);
        let r = ab % ((1 << 31) - 1);

        let q_1 = q & ((1 << 17) - 1);
        let q_2 = q >> 17;

        let r_1 = r & ((1 << 16) - 1);
        let r_2 = r >> 16;

        M31MultHint {
            q_1: q_1 as i64,
            q_2: q_2 as i64,
            r_1: r_1 as i64,
            r_2: r_2 as i64,
        }
    }

    // Compute c from a, b.
    //
    // Input:
    // - table
    // - (k elements)
    // - a1, a2, a3, a4
    // - b1, b2, b3, b4
    //
    // Output:
    // - table
    // - (k elements)
    // - c7, c6, c5, c4, c3, c2, c1
    pub fn from_ab_to_c(k: usize) -> Script {
        script! {
            // c_1 = a1 * b1
            { 7 + 0 + 0 } OP_PICK
            { 3 + 1 + 0 } OP_PICK
            { Lookup8BitGadget::lookup(k + 8 + 0) }
            OP_TOALTSTACK

            // c_2 = a1 * b2 + a2 * b1
            { 7 + 0 + 0 } OP_PICK
            { 2 + 1 + 0 } OP_PICK
            { Lookup8BitGadget::lookup(k + 8 + 0) }
            { 6 + 0 + 1 } OP_PICK
            { 3 + 1 + 1 } OP_PICK
            { Lookup8BitGadget::lookup(k + 8 + 1) }
            OP_ADD OP_TOALTSTACK

            // c_3 = a_1 * b_3 + a_2 * b_2 + a_3 * b_1
            { 7 + 0 + 0 } OP_PICK
            { 1 + 1 + 0 } OP_PICK
            { Lookup8BitGadget::lookup(k + 8 + 0) }
            { 6 + 0 + 1 } OP_PICK
            { 2 + 1 + 1 } OP_PICK
            { Lookup8BitGadget::lookup(k + 8 + 1) }
            OP_ADD
            { 5 + 0 + 1 } OP_PICK
            { 3 + 1 + 1 } OP_PICK
            { Lookup8BitGadget::lookup(k + 8 + 1) }
            OP_ADD OP_TOALTSTACK

            // c_4 = a_1 * b_4 + a_2 * b_3 + a_3 * b_2 + a_4 * b_1
            { 7 + 0 + 0 } OP_ROLL
            { 0 + 1 + 0 } OP_PICK
            { Lookup8BitGadget::lookup(k + 7 + 0) }
            { 6 + 0 + 1 } OP_PICK
            { 1 + 1 + 1 } OP_PICK
            { Lookup8BitGadget::lookup(k + 7 + 1) }
            OP_ADD
            { 5 + 0 + 1 } OP_PICK
            { 2 + 1 + 1 } OP_PICK
            { Lookup8BitGadget::lookup(k + 7 + 1) }
            OP_ADD
            { 4 + 0 + 1 } OP_PICK
            { 3 + 1 + 1 } OP_ROLL
            { Lookup8BitGadget::lookup(k + 6 + 1) }
            OP_ADD OP_TOALTSTACK

            // - table
            // - (k elements)
            // - a2, a3, a4
            // - b2, b3, b4

            // c_5 = a_2 * b_4 + a_3 * b_3 + a_4 * b_2
            { 5 + 0 + 0 } OP_ROLL
            { 0 + 1 + 0 } OP_PICK
            { Lookup8BitGadget::lookup(k + 5 + 0) }
            { 4 + 0 + 1 } OP_PICK
            { 1 + 1 + 1 } OP_PICK
            { Lookup8BitGadget::lookup(k + 5 + 1) }
            OP_ADD
            { 3 + 0 + 1 } OP_PICK
            { 2 + 1 + 1 } OP_ROLL
            { Lookup8BitGadget::lookup(k + 4 + 1) }
            OP_ADD OP_TOALTSTACK

            // - table
            // - (k elements)
            // - a3, a4
            // - b3, b4

            // c_6 = a_3 * b_4 + a_4 * b_3
            { 3 + 0 + 0 } OP_ROLL
            { 0 + 1 + 0 } OP_PICK
            { Lookup8BitGadget::lookup(k + 3 + 0) }
            { 2 + 0 + 1 } OP_PICK
            { 1 + 1 + 1 } OP_ROLL
            { Lookup8BitGadget::lookup(k + 2 + 1) }
            OP_ADD OP_TOALTSTACK

            // c_7 = a_4 * b_4
            { Lookup8BitGadget::lookup(k + 0) }

            OP_FROMALTSTACK
            OP_FROMALTSTACK
            OP_FROMALTSTACK
            OP_FROMALTSTACK
            OP_FROMALTSTACK
            OP_FROMALTSTACK
        }
    }

    pub fn zero_test_with_hint() -> Script {
        script! {
            // pull q_2
            OP_HINT

            // save 2 * q_2 to altstack
            OP_DUP OP_DUP OP_ADD OP_TOALTSTACK

            // stack:
            //   c7 c6 c5 c4 c3 c2 c1
            //   q2

            7 OP_ROLL OP_SUB

            // stack:
            //   c6 c5 c4 c3 c2 c1
            //   q2-c7

            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD

            6 OP_ROLL OP_SUB

            // stack:
            //   c5 c4 c3 c2 c1
            //   (q2-c7)*256-c6

            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD

            5 OP_ROLL OP_SUB

            // stack:
            //   c4 c3 c2 c1
            //   ((q2-c7)*256-c6)*256-c5

            OP_DUP OP_ADD

            // pull q_1 and save a copy
            OP_HINT OP_DUP OP_TOALTSTACK
            OP_ADD

            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD

            4 OP_ROLL OP_SUB

            // stack:
            //   c3 c2 c1
            //   ((((q2-c7)*256-c6)*256-c5)*2+q1)*128-c4

            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD

            // pull r_2
            OP_HINT OP_ADD

            // find 2 * q_2
            OP_FROMALTSTACK OP_FROMALTSTACK OP_SWAP OP_TOALTSTACK
            OP_SUB

            3 OP_ROLL OP_SUB

            // stack:
            //   c2 c1
            //   (((((q2-c7)*256-c6)*256-c5)*2+q1)*128-c4)*256+r2-q2*2-c3

            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD

            2 OP_ROLL OP_SUB

            // stack:
            //   c1
            //   (((((((q2-c7)*256-c6)*256-c5)*2+q1)*128-c4)*256+r2-q2*2-c3)*256-c2

            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
            OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD

            OP_FROMALTSTACK OP_SUB

            // pull r_1
            OP_HINT OP_ADD

            // stack:
            //   c1
            //   (((((((q2-c7)*256-c6)*256-c5)*2+q1)*128-c4)*256+r2-q2*2-c3)*256-c2)*256-q1+r1

            OP_EQUALVERIFY
        }
    }
}

pub struct M31MultTask {
    pub a_1: i64,
    pub a_2: i64,
    pub a_3: i64,
    pub a_4: i64,
    pub b_1: i64,
    pub b_2: i64,
    pub b_3: i64,
    pub b_4: i64,
}

pub struct M31MultHint {
    pub q_1: i64,
    pub q_2: i64,
    pub r_1: i64,
    pub r_2: i64,
}

impl Pushable for &M31MultHint {
    fn bitcoin_script_push(&self, mut builder: Builder) -> Builder {
        builder = self.q_2.bitcoin_script_push(builder);
        builder = self.q_1.bitcoin_script_push(builder);
        builder = self.r_2.bitcoin_script_push(builder);
        builder = self.r_1.bitcoin_script_push(builder);
        builder
    }
}

// Given
// M31 a = a_1 + (a_2 << 8) + (a_3 << 16) + (a_4 << 24) where a_1, a_2, a_3 are 8-bit and a_4 is 7-bit.
// M31 b = b_1 + (b_2 << 8) + (b_3 << 16) + (b_4 << 24) similarly.
//
// a * b
// = c_1 + (c_2 << 8) + (c_3 << 16) + (c_4 << 24) + (c_5 << 32) + (c_6 << 40) + (c_7 << 48)
// where c_1...c_6 are 16-bit and c_7 is at most 14-bit
//
// c_1 = a_1 * b_1
// c_2 = a_1 * b_2 + a_2 * b_1
// c_3 = a_1 * b_3 + a_2 * b_2 + a_3 * b_1
// c_4 = a_1 * b_4 + a_2 * b_3 + a_3 * b_2 + a_4 * b_1
// c_5 = a_2 * b_4 + a_3 * b_3 + a_4 * b_2
// c_6 = a_3 * b_4 + a_4 * b_3
// c_7 = a_4 * b_4
//
// in total 16 8-bit mult
//
// Now, reduction phase
//
// given hints: q and r
// Compute q * (1 << 31 - 1) + r = q * (1 << 31) + r - q
//
// let q = q_1 + (q_2 << 17) = q_1 + ((q_2 << 1) << 16)
//
// q * (1 << 31)
// = (q_1 << 31) + (q_2 << 48)
//
// let r = r_1 + (r_2 << 16)
//
// then, it is a zero-test

#[cfg(test)]
mod test_single {
    use bitcoin_circle_stark::tests_utils::report::report_bitcoin_script_size;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use crate::m31_mult::M31MultGadget;
    use crate::utils::m31_to_limbs;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_scriptexec::execute_script;
    use crate::table::generate_table;

    #[test]
    fn test_m31_mult() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a = prng.gen_range(0u32..((1 << 31) - 1));
            let b = prng.gen_range(0u32..((1 << 31) - 1));

            let a_limbs = m31_to_limbs(a);
            let b_limbs = m31_to_limbs(b);

            let mut c_limbs = vec![0i64; 7];

            for i in 0..4 {
                for j in 0..4 {
                    c_limbs[i + j] += a_limbs[i] * b_limbs[j];
                }
            }

            let mut c_actual = 0i64;
            for i in 0..7 {
                c_actual <<= 8;
                c_actual += c_limbs[6 - i];
            }

            let hint = M31MultGadget::compute_hint(&[(a, b)]);
            assert!(!hint.q_1.is_negative());
            assert!(!hint.q_2.is_negative());
            assert!(!hint.r_1.is_negative());
            assert!(!hint.r_2.is_negative());

            assert!(hint.q_1 < (1 << 17));
            assert!(hint.q_2 < (1 << 14));
            assert!(hint.r_1 < (1 << 16));
            assert!(hint.r_2 < (1 << 16));

            let q = c_actual / ((1 << 31) - 1);
            assert_eq!(q, hint.q_1 + (hint.q_2 << 17));

            let mut another_limbs = vec![0i64; 7];
            another_limbs[0] += hint.r_1;
            another_limbs[2] += hint.r_2;

            another_limbs[0] -= hint.q_1;
            another_limbs[2] -= hint.q_2 * 2;

            another_limbs[3] += hint.q_1 * 128;
            another_limbs[6] += hint.q_2;

            let mut gap = 0i64;
            for i in 0..7 {
                gap <<= 8;
                gap += c_limbs[6 - i] - another_limbs[6 - i];
                assert!(gap.abs() < (1 << 30));
            }
            assert_eq!(gap, 0i64);
        }
    }

    #[test]
    fn test_from_ab_to_c() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        report_bitcoin_script_size(
            "m31_mult",
            "from_ab_to_c",
            M31MultGadget::from_ab_to_c(0).len()
        );

        for i in 0..100 {
            let a = prng.gen_range(0u32..((1 << 31) - 1));
            let b = prng.gen_range(0u32..((1 << 31) - 1));

            let a_limbs = m31_to_limbs(a);
            let b_limbs = m31_to_limbs(b);

            let mut c_limbs = vec![0i64; 7];

            for i in 0..4 {
                for j in 0..4 {
                    c_limbs[i + j] += a_limbs[i] * b_limbs[j];
                }
            }

            let table = generate_table::<9>();

            let script = script! {
                { &table }
                for _ in 0..i {
                    { 1 }
                }
                { a_limbs.to_vec() }
                { b_limbs.to_vec() }
                { M31MultGadget::from_ab_to_c(i) }
                for c_limb in c_limbs.iter() {
                    { *c_limb }
                    OP_EQUALVERIFY
                }
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

    #[test]
    fn test_zero_test_with_hint() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        report_bitcoin_script_size(
            "m31_mult",
            "zero_test_with_hint",
            M31MultGadget::zero_test_with_hint().len()
        );

        for _ in 0..100 {
            let a = prng.gen_range(0u32..((1 << 31) - 1));
            let b = prng.gen_range(0u32..((1 << 31) - 1));

            let a_limbs = m31_to_limbs(a);
            let b_limbs = m31_to_limbs(b);

            let mut c_limbs = vec![0i64; 7];

            for i in 0..4 {
                for j in 0..4 {
                    c_limbs[i + j] += a_limbs[i] * b_limbs[j];
                }
            }

            let hint = M31MultGadget::compute_hint(&[(a, b)]);

            let script = script! {
                { &hint }
                for c_limb in c_limbs.iter().rev() {
                    { *c_limb }
                }
                { M31MultGadget::zero_test_with_hint() }
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}

#[cfg(test)]
mod test_double {
    use bitcoin_script::script;
    use bitcoin_scriptexec::execute_script;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use crate::m31_mult::M31MultGadget;
    use crate::utils::m31_to_limbs;
    use bitcoin_circle_stark::treepp::*;

    #[test]
    fn test_zero_test_with_hint() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a = prng.gen_range(0u32..((1 << 31) - 1));
            let b = prng.gen_range(0u32..((1 << 31) - 1));

            let a_limbs = m31_to_limbs(a);
            let b_limbs = m31_to_limbs(b);

            let d = prng.gen_range(0u32..((1 << 31) - 1));
            let e = prng.gen_range(0u32..((1 << 31) - 1));

            let d_limbs = m31_to_limbs(d);
            let e_limbs = m31_to_limbs(e);

            let mut c_limbs = vec![0i64; 7];

            for i in 0..4 {
                for j in 0..4 {
                    c_limbs[i + j] += a_limbs[i] * b_limbs[j];
                    c_limbs[i + j] += d_limbs[i] * e_limbs[j];
                }
            }

            let hint = M31MultGadget::compute_hint(&[(a, b), (d, e)]);

            let script = script! {
                { &hint }
                for c_limb in c_limbs.iter().rev() {
                    { *c_limb }
                }
                { M31MultGadget::zero_test_with_hint() }
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}