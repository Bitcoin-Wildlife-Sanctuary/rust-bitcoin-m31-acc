use bitcoin_circle_stark::treepp::pushable::{Builder, Pushable};
use std::ops::Index;
use std::sync::OnceLock;

pub static TABLE: OnceLock<Table> = OnceLock::new();

#[derive(Clone)]
pub struct Table {
    pub data: Vec<i64>,
}

impl Index<usize> for Table {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl Pushable for &Table {
    fn bitcoin_script_push(&self, mut builder: Builder) -> Builder {
        for &i in self.data.iter().rev() {
            builder = builder.push_int(i);
        }
        builder
    }
}

pub fn generate_table<const N: usize>() -> Table {
    assert!(N >= 1);
    assert!(N <= 9);

    let mut v = vec![0i64; (1 << N) + 1];

    for i in 0..((1 << N) + 1) {
        v[i] = ((i * i) / 4) as i64;
    }

    Table { data: v }
}

pub fn get_table() -> &'static Table {
    TABLE.get_or_init(|| generate_table::<9>())
}

#[cfg(test)]
mod test {
    use crate::algorithms::table::get_table;
    use crate::report_bitcoin_script_size;
    use bitcoin_circle_stark::treepp::*;
    pub use bitcoin_scriptexec::execute_script;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use std::mem::swap;

    #[test]
    fn test_hypothesis() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let table = get_table();

        for _ in 0..100 {
            let mut a = prng.gen_range(0usize..(1 << 8));
            let mut b = prng.gen_range(0usize..(1 << 8));

            let expected = (a * b) as i64;

            if b > a {
                swap(&mut a, &mut b);
            }

            let a_plus_b_squared_div_4 = table[a + b];
            let a_minus_b_squared_div_4 = table[a - b];

            assert_eq!(expected, a_plus_b_squared_div_4 - a_minus_b_squared_div_4);
        }
    }

    #[test]
    fn test_pushable() {
        let table = get_table();

        report_bitcoin_script_size(
            "table",
            "push_table",
            script! {
                { table }
            }
            .len(),
        );

        let script = script! {
            { table }

            for _ in 0..256 {
                OP_2DROP
            }
            OP_DROP
            OP_TRUE
        };

        let exec_result = execute_script(script);
        assert!(exec_result.success);
    }
}
