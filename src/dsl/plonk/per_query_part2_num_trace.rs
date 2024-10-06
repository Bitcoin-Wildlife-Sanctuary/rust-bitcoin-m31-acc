use crate::dsl::plonk::hints::Hints;
use anyhow::Result;
use bitcoin_script_dsl::builtins::m31::M31Var;
use bitcoin_script_dsl::constraint_system::{ConstraintSystem, ConstraintSystemRef};
use bitcoin_script_dsl::worm::WORMMemory;

pub fn generate_cs(
    _: &Hints,
    worm: &mut WORMMemory,
    query_idx: usize,
) -> Result<ConstraintSystemRef> {
    let cs = ConstraintSystem::new_ref();
    worm.init(&cs)?;

    let y: M31Var = worm.read(format!("circle_point_y_{}", query_idx))?;

    worm.save()?;
    Ok(cs)
}
