pub mod table;

pub mod m31;

use crate::dsl::m31::{m31_equalverify, m31_equalverify_gadget, m31_to_limbs, m31_to_limbs_gadget};
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
    dsl.add_function(
        "m31_to_limbs",
        FunctionMetadata {
            trace_generator: m31_to_limbs,
            script_generator: m31_to_limbs_gadget,
            input: vec!["m31"],
            output: vec!["m31_limbs"],
        },
    );
    dsl.add_function(
        "m31_equalverify",
        FunctionMetadata {
            trace_generator: m31_equalverify,
            script_generator: m31_equalverify_gadget,
            input: vec!["m31_limbs", "m31_limbs"],
            output: vec![],
        },
    )
}
