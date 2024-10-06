use bitcoin_script_dsl::builtins::cm31::CM31Var;
use bitcoin_script_dsl::builtins::m31::M31Var;
use bitcoin_script_dsl::builtins::table::TableVar;

pub fn apply_twin(
    table: &TableVar,
    z_y: &M31Var,
    queried_value_for_z: &M31Var,
    queried_value_for_conjugated_z: &M31Var,
    a: &CM31Var,
    b: &CM31Var,
) -> (CM31Var, CM31Var) {
    //let a_times_z_y = a * (table, z_y);
    todo!()
}
