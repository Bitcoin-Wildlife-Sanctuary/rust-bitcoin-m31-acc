pub mod table;

pub mod m31;

pub mod cm31;

use crate::dsl::table::{push_table, push_table_gadget};
use bitcoin_script_dsl::dsl::{ElementType, DSL};
use bitcoin_script_dsl::functions::FunctionMetadata;

pub fn load_data_types(dsl: &mut DSL) {
    dsl.add_ref_only_data_type("table", ElementType::ManyNum((1 << 9) + 1));
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

    m31::load_functions(dsl);
    cm31::load_functions(dsl);
}
