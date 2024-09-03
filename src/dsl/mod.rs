pub mod table;

pub mod m31;

pub mod cm31;

pub mod qm31;

pub mod sha256;

pub mod point;

pub mod pow;

pub mod merkle_tree;

pub mod example;

pub mod utils;

use crate::dsl::table::{push_table, push_table_gadget};
use bitcoin_script_dsl::dsl::{ElementType, DSL};
use bitcoin_script_dsl::functions::FunctionMetadata;

pub fn load_data_types(dsl: &mut DSL) {
    dsl.add_data_type("table", ElementType::ManyNum((1 << 9) + 1));
    dsl.add_data_type("m31_limbs", ElementType::ManyNum(4));
    dsl.add_data_type("cm31_limbs", ElementType::ManyNum(8));
    dsl.add_data_type("qm31_limbs", ElementType::ManyNum(16));

    dsl.add_data_type("m31", ElementType::Num);
    dsl.add_data_type("cm31", ElementType::ManyNum(2));
    dsl.add_data_type("qm31", ElementType::ManyNum(4));
    dsl.add_data_type("hash", ElementType::Str);

    dsl.add_data_type("position", ElementType::Num);
    dsl.add_data_type("internal", ElementType::Str);
}

pub fn load_functions(dsl: &mut DSL) {
    dsl.add_function(
        "push_table",
        FunctionMetadata {
            trace_generator: push_table,
            script_generator: push_table_gadget,
            input: vec![],
            output: vec!["table"],
        },
    );

    m31::load_functions(dsl);
    cm31::load_functions(dsl);
    qm31::load_functions(dsl);
    sha256::load_functions(dsl);
    pow::load_functions(dsl);
    merkle_tree::load_functions(dsl);
}
