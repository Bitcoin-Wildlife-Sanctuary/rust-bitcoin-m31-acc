pub mod hints;

pub mod part1_fiat_shamir1;
pub mod part2_fiat_shamir2_and_constraint_num;
pub mod part3_constraint_denom;
pub mod part4_pair_vanishing_and_alphas;
pub mod part5_column_line_coeffs1;
pub mod part6_column_line_coeffs2;
pub mod part7_column_line_coeffs3;

pub mod per_query_part1_folding;
pub mod per_query_part2_num_trace;

pub mod part8_cleanup;

#[cfg(test)]
mod test {
    use crate::dsl::plonk::hints::Hints;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_script_dsl::test_program;
    use bitcoin_script_dsl::worm::WORMMemory;
    use stwo_prover::core::prover::N_QUERIES;

    #[test]
    fn test_generate_dsl() {
        let hints = Hints::instance();
        let mut worm = WORMMemory::new();

        let cs = super::part1_fiat_shamir1::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        let cs =
            super::part2_fiat_shamir2_and_constraint_num::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        let cs = super::part3_constraint_denom::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        let cs = super::part4_pair_vanishing_and_alphas::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        let cs = super::part5_column_line_coeffs1::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        let cs = super::part6_column_line_coeffs2::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        let cs = super::part7_column_line_coeffs3::generate_cs(&hints, &mut worm).unwrap();
        test_program(
            cs,
            script! {
                { worm.write_hash_var.as_ref().unwrap().value.clone() }
                { worm.read_hash_var.as_ref().unwrap().value.clone() }
            },
        )
        .unwrap();

        for i in 0..N_QUERIES {
            let cs = super::per_query_part1_folding::generate_cs(&hints, &mut worm, i).unwrap();
            test_program(
                cs,
                script! {
                    { worm.write_hash_var.as_ref().unwrap().value.clone() }
                    { worm.read_hash_var.as_ref().unwrap().value.clone() }
                },
            )
            .unwrap();
        }

        let cs = super::part8_cleanup::generate_cs(&hints, &mut worm).unwrap();
        test_program(cs, script! {}).unwrap();
    }
}
