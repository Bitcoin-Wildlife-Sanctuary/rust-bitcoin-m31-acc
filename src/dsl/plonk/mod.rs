pub mod hints;

pub mod part1_fiat_shamir1_noncompute;

#[cfg(test)]
mod test {
    use crate::dsl::plonk::hints::Hints;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::test_program;
    use bitcoin_script_dsl::worm::WORMMemory;

    #[test]
    fn test_generate_dsl() {
        let hints = Hints::instance();
        let mut worm = WORMMemory::new();

        let cs = super::part1_fiat_shamir1_noncompute::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();
    }
}
