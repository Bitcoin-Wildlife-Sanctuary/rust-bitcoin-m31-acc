use bitcoin::script::write_scriptint;
use bitcoin_circle_stark::channel::{BitcoinIntegerEncodedData, DrawHints};
use bitcoin_script_dsl::dsl::{Element, MemoryEntry};

pub fn draw_hints_to_memory_entries(hint: DrawHints) -> Vec<MemoryEntry> {
    let mut new_hints = vec![];
    for hint_element in hint.0.iter() {
        let data = match hint_element {
            BitcoinIntegerEncodedData::NegativeZero => {
                vec![0x80]
            }
            BitcoinIntegerEncodedData::Other(v) => {
                let mut out = [0u8; 8];
                let len = write_scriptint(&mut out, *v);
                out[0..len].to_vec()
            }
        };
        new_hints.push(MemoryEntry::new("internal", Element::Str(data)));
    }
    if !hint.1.is_empty() {
        new_hints.push(MemoryEntry::new("internal", Element::Str(hint.1)));
    }

    new_hints
}
