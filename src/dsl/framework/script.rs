use crate::dsl::framework::dsl::MemoryEntry;
use bitcoin_circle_stark::treepp::Script;

pub struct CompiledProgram {
    pub input: Vec<MemoryEntry>,
    pub script: Script,
    pub hint: Vec<MemoryEntry>,
}
