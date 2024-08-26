use crate::table::generate_table;
use crate::treepp::*;
use anyhow::Result;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::FunctionOutput;
use itertools::Itertools;

pub fn push_table(_: &mut DSL, _: &[usize]) -> Result<FunctionOutput> {
    let table = generate_table::<9>();

    // important: reverse the order
    let data = table.data.iter().map(|x| *x as i32).rev().collect_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("table", Element::ManyNum(data))],
        new_hints: vec![],
    })
}

pub fn push_table_gadget(_: &[usize]) -> Result<Script> {
    let table = generate_table::<9>();

    Ok(script! {
        { &table }
    })
}
