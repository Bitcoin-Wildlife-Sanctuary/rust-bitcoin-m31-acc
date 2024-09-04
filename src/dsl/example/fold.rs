use crate::dsl::building_blocks::qm31::qm31_mul_m31_limbs;
use anyhow::Result;
use bitcoin_script_dsl::dsl::DSL;

pub fn ibutterfly(
    dsl: &mut DSL,
    table: usize,
    v0: usize,
    v1: usize,
    itwid: usize,
) -> Result<(usize, usize)> {
    let new_v0 = dsl.execute("qm31_add", &[v0, v1])?[0];

    let diff = dsl.execute("qm31_sub", &[v0, v1])?[0];
    let itwid_limbs = dsl.execute("m31_to_limbs", &[itwid])?[0];

    let new_v1 = qm31_mul_m31_limbs(dsl, table, diff, itwid_limbs)?;

    Ok((new_v0, new_v1))
}

#[cfg(test)]
mod test {
    use crate::dsl::building_blocks::qm31::reformat_qm31_to_dsl_element;
    use crate::dsl::example::fold::ibutterfly;
    use crate::dsl::load_data_types;
    use crate::dsl::load_functions;
    use bitcoin_circle_stark::treepp::*;
    use bitcoin_circle_stark::utils::get_rand_qm31;
    use bitcoin_script_dsl::dsl::{Element, DSL};
    use bitcoin_script_dsl::test_program;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use stwo_prover::core::fields::m31::M31;

    #[test]
    fn test_ibutterfly() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let v0 = get_rand_qm31(&mut prng);
        let v1 = get_rand_qm31(&mut prng);
        let itwid = M31::reduce(prng.next_u64());

        let expected = {
            let mut t0 = v0.clone();
            let mut t1 = v1.clone();

            stwo_prover::core::fft::ibutterfly(&mut t0, &mut t1, itwid);

            (t0, t1)
        };

        let mut dsl = DSL::new();

        load_data_types(&mut dsl);
        load_functions(&mut dsl);

        let v0_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(v0)))
            .unwrap();
        let v1_var = dsl
            .alloc_input("qm31", Element::ManyNum(reformat_qm31_to_dsl_element(v1)))
            .unwrap();
        let itwid = dsl
            .alloc_input("m31", Element::Num(itwid.0 as i32))
            .unwrap();

        let table = dsl.execute("push_table", &[]).unwrap()[0];

        let (new_v0_var, new_v1_var) = ibutterfly(&mut dsl, table, v0_var, v1_var, itwid).unwrap();

        assert_eq!(
            dsl.get_many_num(new_v0_var).unwrap(),
            reformat_qm31_to_dsl_element(expected.0)
        );
        assert_eq!(
            dsl.get_many_num(new_v1_var).unwrap(),
            reformat_qm31_to_dsl_element(expected.1)
        );

        dsl.set_program_output("qm31", new_v0_var).unwrap();
        dsl.set_program_output("qm31", new_v1_var).unwrap();

        test_program(
            dsl,
            script! {
                { expected.0 }
                { expected.1 }
            },
        )
        .unwrap()
    }
}
