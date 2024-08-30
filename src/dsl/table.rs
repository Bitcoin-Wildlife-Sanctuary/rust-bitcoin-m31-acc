use crate::table::get_table;
use anyhow::Result;
use bitcoin_circle_stark::treepp::*;
use bitcoin_script_dsl::dsl::{Element, MemoryEntry, DSL};
use bitcoin_script_dsl::functions::FunctionOutput;
use itertools::Itertools;

pub fn push_table(_: &mut DSL, _: &[usize]) -> Result<FunctionOutput> {
    let table = get_table();

    // important: reverse the order
    let data = table.data.iter().map(|x| *x as i32).rev().collect_vec();

    Ok(FunctionOutput {
        new_elements: vec![MemoryEntry::new("table", Element::ManyNum(data))],
        new_hints: vec![],
    })
}

pub fn push_table_gadget(_: &[usize]) -> Result<Script> {
    let table = get_table();

    Ok(script! {
        { table }
    })
}
