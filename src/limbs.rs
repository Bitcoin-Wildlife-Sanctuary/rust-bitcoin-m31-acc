use bitcoin_circle_stark::treepp::*;

pub struct M31LimbsGadget;

impl M31LimbsGadget {
    // a1, ..., a4
    // b1, ..., b4
    pub fn add_limbs() -> Script {
        script! {
            7 OP_ROLL
            { 3 + 1 } OP_ROLL
            OP_ADD
            OP_DUP 256 OP_GREATERTHANOREQUAL
            OP_IF
                256 OP_SUB
                { 1 }
            OP_ELSE
                { 0 }
            OP_ENDIF

            // a2, a3, a4
            // b2, b3, b4
            // c1, carry

            { 5 + 2 } OP_ROLL
            { 2 + 1 + 2 } OP_ROLL
            OP_ADD OP_ADD
            OP_DUP 256 OP_GREATERTHANOREQUAL
            OP_IF
                256 OP_SUB
                { 1 }
            OP_ELSE
                { 0 }
            OP_ENDIF

            // a3, a4
            // b3, b4
            // c1, c2, carry

            { 3 + 3 } OP_ROLL
            { 1 + 1 + 3 } OP_ROLL
            OP_ADD OP_ADD
            OP_DUP 256 OP_GREATERTHANOREQUAL
            OP_IF
                256 OP_SUB
                { 1 }
            OP_ELSE
                { 0 }
            OP_ENDIF

            // a4
            // b4
            // c1, c2, c3, carry
            { 1 + 4 } OP_ROLL
            { 0 + 1 + 4 } OP_ROLL
            OP_ADD OP_ADD
            OP_DUP 128 OP_GREATERTHANOREQUAL
            OP_IF
                128 OP_SUB
                OP_2SWAP OP_SWAP OP_1ADD OP_SWAP OP_2SWAP
            OP_ENDIF

            // c1, c2, c3, c4
            // note: c4 could be a little bit larger, but our program can handle it
        }
    }
}

#[cfg(test)]
mod test {
    use bitcoin_circle_stark::tests_utils::report::report_bitcoin_script_size;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use crate::limbs::M31LimbsGadget;

    #[test]
    fn test_add_limbs() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        report_bitcoin_script_size(
            "short_limbs",
            "add_limbs",
            M31LimbsGadget::add_limbs().len()
        );
    }
}