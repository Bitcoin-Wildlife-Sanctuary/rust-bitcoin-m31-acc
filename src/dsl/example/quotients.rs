use anyhow::Result;
use bitcoin_script_dsl::dsl::DSL;

#[derive(Clone)]
pub struct DenominatorInversesIndices {
    pub u1: usize,
    pub u2: usize,
    pub u3: usize,
}

#[derive(Clone)]
pub struct NominatorsIndices {
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub c1: usize,
    pub c2: usize,
    pub c3: usize,
    pub c4: usize,
}

pub fn aggregation(
    dsl: &mut DSL,
    table: usize,
    left_query: (DenominatorInversesIndices, NominatorsIndices),
    right_query: (DenominatorInversesIndices, NominatorsIndices),
    power_alphas: &[usize],
) -> Result<(usize, usize)> {
    todo!()
}
