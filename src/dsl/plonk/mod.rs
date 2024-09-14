pub mod hints;

pub mod part1_fiat_shamir_noncompute;

#[cfg(test)]
mod test {
    use crate::dsl::plonk::hints::Hints;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::test_program;
    use std::collections::HashMap;

    #[test]
    fn test_generate_dsl() {
        let hints = Hints::instance();
        let mut cache = HashMap::new();

        let dsl = super::part1_fiat_shamir_noncompute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(dsl, script! {}).unwrap();
    }
}
