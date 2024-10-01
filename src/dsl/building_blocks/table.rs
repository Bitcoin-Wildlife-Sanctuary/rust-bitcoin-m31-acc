use crate::algorithms::table::get_table;
use crate::dsl::framework::dsl::{Element, MemoryEntry, DSL};
use crate::dsl::framework::functions::FunctionOutput;
use anyhow::Result;
use bitcoin_circle_stark::treepp::*;
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
