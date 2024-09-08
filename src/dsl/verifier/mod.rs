pub mod hints;

pub mod part1_fiat_shamir_plus_precomputed_merkle_noncompute;

pub mod part2_fiat_shamir_step_numerator_compute;

pub mod part3_fiat_shamir_boundary_compute;

pub mod part4_fiat_shamir_step_denominator_compute;

pub mod part5_oods_point_compute;

pub mod part6_column_line_coeff_trace_compute;

pub mod part7_column_linear_combination_compute;

pub mod part8_prepared_oods_and_alphas_compute;

pub mod part9_sort_queries_noncompute;

#[cfg(test)]
mod test {
    use crate::dsl::verifier::hints::Hints;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::test_program;
    use std::collections::HashMap;

    #[test]
    fn test_generate_dsl() {
        let hints = Hints::instance();
        let mut cache = HashMap::new();

        let dsl = super::part1_fiat_shamir_plus_precomputed_merkle_noncompute::generate_dsl(
            &hints, &mut cache,
        )
        .unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir_verify1").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part2_fiat_shamir_step_numerator_compute::generate_dsl(&hints, &mut cache)
            .unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir_verify2").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part3_fiat_shamir_boundary_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir_verify3").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part4_fiat_shamir_step_denominator_compute::generate_dsl(&hints, &mut cache)
                .unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir_verify4").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part5_oods_point_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("column_line_coeff1").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part6_column_line_coeff_trace_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("column_line_coeff2").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part7_column_linear_combination_compute::generate_dsl(&hints, &mut cache)
            .unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("prepared_oods1").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part8_prepared_oods_and_alphas_compute::generate_dsl(&hints, &mut cache)
            .unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("prepared_oods2").unwrap().hash }
                { cache.get("after_fiat_shamir").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part9_sort_queries_noncompute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("shared_information").unwrap().hash }
            },
        )
        .unwrap();
    }
}
