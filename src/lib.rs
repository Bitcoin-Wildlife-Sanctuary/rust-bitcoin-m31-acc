pub fn report_bitcoin_script_size(category: &str, name: &str, script_size_bytes: usize) {
    println!("{}.{} = {} bytes", category, name, script_size_bytes);
}

pub mod table;

pub mod lookup;

pub mod m31;

pub mod utils;

pub mod cm31;

pub mod qm31;

pub mod decompose;

pub mod dsl;
