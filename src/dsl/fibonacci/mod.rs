pub mod hints;

pub mod part1_fiat_shamir_plus_precomputed_merkle_noncompute;

pub mod part2_fiat_shamir_step_numerator_compute;

pub mod part3_fiat_shamir_boundary_compute;

pub mod part4_fiat_shamir_step_denominator_compute;

pub mod part5_oods_point_compute;

pub mod part6_column_line_coeff_trace_compute;

pub mod part7_column_linear_combination_compute;

pub mod part8_prepared_oods_and_alphas_compute;

pub mod part9_sort_queries_first_3_noncompute;

pub mod part10_sort_queries_last_5_noncompute;

pub mod per_query_part1_reorganize_noncompute;

pub mod per_query_part2_folding_compute;

pub mod per_query_part3_folding_compute;

pub mod per_query_part4_num_denom1_compute;

pub mod per_query_part5_num2_compute;

pub mod per_query_part6_aggregation2_compute;

pub mod per_query_part7_aggregation3_compute;

pub mod per_query_part8_aggregation4_compute;

pub mod covenant;

#[cfg(test)]
mod test {
    use crate::dsl::fibonacci::hints::Hints;
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

        let dsl =
            super::part9_sort_queries_first_3_noncompute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("shared_information").unwrap().hash }
                { cache.get("query1").unwrap().hash }
                { cache.get("query2").unwrap().hash }
                { cache.get("query3").unwrap().hash }
                { cache.get("unsorted").unwrap().hash }
            },
        )
        .unwrap();

        let dsl =
            super::part10_sort_queries_last_5_noncompute::generate_dsl(&hints, &mut cache).unwrap();
        test_program(
            dsl,
            script! {
                { cache.get("global_state").unwrap().hash }
            },
        )
        .unwrap();

        for query_idx in 1..=8 {
            let dsl = super::per_query_part1_reorganize_noncompute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_fri_folding1", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl =
                super::per_query_part2_folding_compute::generate_dsl(&hints, &mut cache, query_idx)
                    .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_fri_folding2", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl =
                super::per_query_part3_folding_compute::generate_dsl(&hints, &mut cache, query_idx)
                    .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_num_denom1", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part4_num_denom1_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_num_denom1_result", query_idx)).unwrap().hash }
                    { cache.get(&format!("query{}_num2", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl =
                super::per_query_part5_num2_compute::generate_dsl(&hints, &mut cache, query_idx)
                    .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_aggregation1_result", query_idx)).unwrap().hash }
                    { cache.get(&format!("query{}_aggregation2", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part6_aggregation2_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_aggregation3", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part7_aggregation3_compute::generate_dsl(
                &hints, &mut cache, query_idx,
            )
            .unwrap();
            test_program(
                dsl,
                script! {
                    { cache.get("global_state").unwrap().hash }
                    { cache.get(&format!("query{}_aggregation4", query_idx)).unwrap().hash }
                },
            )
            .unwrap();

            let dsl = super::per_query_part8_aggregation4_compute::generate_dsl(
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
