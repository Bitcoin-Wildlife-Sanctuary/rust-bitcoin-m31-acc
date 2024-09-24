pub mod hints;

pub mod covenant;

pub mod part1_fiat_shamir1_noncompute;

pub mod part2_fiat_shamir2_noncompute;

pub mod part3_constraint_algebra_logup_ab_compute;

pub mod part4_constraint_logup_c_compute;

pub mod part5_constraint_oods_coset_vanishing_compute;

pub mod part6_constraint_equalverify_compute;

pub mod part7_constraint_oods_shifted_and_prepared_compute;

pub mod part8_alphas_compute;

pub mod part9_column_line_coeffs1_compute;

pub mod part10_column_line_coeffs2_compute;

pub mod part11_column_line_coeffs3_compute;

pub mod part12_column_line_coeffs4_compute;

pub mod part13_sort_queries1_noncompute;

pub mod part14_sort_queries2_noncompute;

pub mod per_query_part1_folding_compute;

pub mod per_query_part2_num_trace_compute;

pub mod per_query_part3_num_interaction1_compute;

pub mod per_query_part4_num_interaction2_compute;

pub mod per_query_part5_num_interaction3_compute;

pub mod per_query_part6_num_constant_compute;

pub mod per_query_part7_num_composition_compute;

pub mod per_query_part8_denom_and_equalverify_compute;

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
            super::part3_constraint_algebra_logup_ab_compute::generate_dsl(&hints, &mut cache)
                .unwrap();
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
            super::part4_constraint_logup_c_compute::generate_dsl(&hints, &mut cache).unwrap();
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
            super::part5_constraint_oods_coset_vanishing_compute::generate_dsl(&hints, &mut cache)
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
            super::part6_constraint_equalverify_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("constraint_oods_point").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part7_constraint_oods_shifted_and_prepared_compute::generate_dsl(
            &hints, &mut cache,
        )
        .unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("prepared_oods").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part8_alphas_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("fiat_shamir1_result").unwrap().hash }
                { cache.get("fiat_shamir2_result").unwrap().hash }
                { cache.get("prepared_oods").unwrap().hash }
                { cache.get("alphas").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part9_column_line_coeffs1_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("column_line_coeffs1").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part10_column_line_coeffs2_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("column_line_coeffs2").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part11_column_line_coeffs3_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("column_line_coeffs3").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part12_column_line_coeffs4_compute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("shared_information").unwrap().hash }
                { cache.get("sort_queries1").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part13_sort_queries1_noncompute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("shared_information").unwrap().hash }
                { cache.get("sort_queries1").unwrap().hash }
                { cache.get("folding_intermediate_1").unwrap().hash }
                { cache.get("folding_intermediate_2").unwrap().hash }
                { cache.get("folding_intermediate_3").unwrap().hash }
                { cache.get("folding_intermediate_4").unwrap().hash }
                { cache.get("folding_intermediate_5").unwrap().hash }
                { cache.get("folding_intermediate_6").unwrap().hash }
                { cache.get("folding_intermediate_7").unwrap().hash }
                { cache.get("folding_intermediate_8").unwrap().hash }
            },
        )
        .unwrap();

        let dsl = super::part14_sort_queries2_noncompute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("global_state").unwrap().hash }
            },
        )
        .unwrap();
        println!("this");

        for query_idx in 1..=8 {
            let dsl =
                super::per_query_part1_folding_compute::generate_dsl(&hints, &mut cache, query_idx)
                    .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query_post_folding_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part2_num_trace_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("trace_results_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part3_num_interaction1_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("interaction1_results_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part4_num_interaction2_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("interaction2_results_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part5_num_interaction3_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("interaction3_results_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part6_num_constant_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("constant_results_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part7_num_composition_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("composition_results_{}", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part8_denom_and_equalverify_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                },
            )
            .unwrap();
        }
    }
}
