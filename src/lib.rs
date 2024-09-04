pub fn report_bitcoin_script_size(category: &str, name: &str, script_size_bytes: usize) {
    println!("{}.{} = {} bytes", category, name, script_size_bytes);
}

pub mod algorithms;
pub mod dsl;
