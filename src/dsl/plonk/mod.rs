pub mod hints;

pub mod part1_fiat_shamir1_noncompute;

pub mod part2_fiat_shamir2_noncompute;

pub mod part3_constraint_algebra_compute;

pub mod part4_constraint_logup_ab_compute;

pub mod part5_constraint_logup_c_compute;

pub mod part6_constraint_oods_coset_vanishing_compute;

pub mod part7_constraint_equalverify_compute;

pub mod part8_constraint_oods_shifted_and_prepared_compute;

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

        let dsl = super::part1_fiat_shamir1_noncompute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir2").unwrap().hash }
                { cache.get("fiat_shamir1_result").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part2_fiat_shamir2_noncompute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part3_constraint_algebra_compute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("constraint_algebra_result").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part4_constraint_logup_ab_compute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("constraint_logup_ab_result").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part5_constraint_logup_c_compute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("constraint_logup_c_result").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part6_constraint_oods_coset_vanishing_compute::generate_dsl(&hints, &mut cache)
                .unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("constraint_oods_coset_vanishing_result").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part7_constraint_equalverify_compute::generate_dsl(&hints, &mut cache).unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("constraint_oods_point").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part8_constraint_oods_shifted_and_prepared_compute::generate_dsl(
            &hints, &mut cache,
        )
        .unwrap();

        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("oods_with_shifted_01").unwrap().hash }
            },
        )
        .unwrap();
    }
}
