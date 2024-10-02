pub mod building_blocks;
pub mod framework;
pub mod modules;
pub mod plonk;
pub mod tools;
pub mod utils;

use crate::dsl::framework::dsl::{ElementType, DSL};
use crate::dsl::framework::functions::FunctionMetadata;
use anyhow::Result;
use building_blocks::table::{push_table, push_table_gadget};
use building_blocks::{cm31, m31, merkle_tree, pow, qm31, sha256};

pub fn load_data_types(dsl: &mut DSL) -> Result<()> {
    dsl.add_data_type("table", ElementType::ManyNum((1 << 9) + 1))?;
    dsl.add_data_type("m31_limbs", ElementType::ManyNum(4))?;
    dsl.add_data_type("cm31_limbs", ElementType::ManyNum(8))?;
    dsl.add_data_type("qm31_limbs", ElementType::ManyNum(16))?;

    dsl.add_data_type("m31", ElementType::Num)?;
    dsl.add_data_type("cm31", ElementType::ManyNum(2))?;
    dsl.add_data_type("qm31", ElementType::ManyNum(4))?;
    dsl.add_data_type("hash", ElementType::Str)?;

    dsl.add_data_type("position", ElementType::Num)?;
    dsl.add_data_type("internal", ElementType::Str)?;

    Ok(())
}

pub fn load_functions(dsl: &mut DSL) -> Result<()> {
    dsl.add_function(
        "push_table",
        FunctionMetadata {
            trace_generator: push_table,
            script_generator: push_table_gadget,
            input: vec![],
            output: vec!["table"],
        },
    )?;

    m31::load_functions(dsl)?;
    cm31::load_functions(dsl)?;
    qm31::load_functions(dsl)?;
    sha256::load_functions(dsl)?;
    pow::load_functions(dsl)?;
    merkle_tree::load_functions(dsl)?;
    tools::load_functions(dsl)?;
    building_blocks::utils::load_functions(dsl)?;

    Ok(())
}
