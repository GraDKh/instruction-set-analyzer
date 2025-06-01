use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;

use iced_x86::{CpuidFeature, Decoder, DecoderOptions};
use object::{Object, ObjectSection};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: <path-to-binary>".to_string().into());
    }
    let file_path = &args[1];

    let used_features = detect_instruction_sets(file_path)?;

    println!("\nBinary uses the following CPU features:");
    for f in &used_features {
        println!("- {:?}", f);
    }
    Ok(())
}

fn detect_instruction_sets(path: &str) -> Result<HashSet<CpuidFeature>, Box<dyn Error>> {
    let binary = fs::read(path)?;
    let obj_file = object::File::parse(&*binary)?;

    let mut features = HashSet::new();

    for section in obj_file.sections() {
        if section.kind() != object::SectionKind::Text {
            continue;
        }

        let addr = section.address();
        let bytes = match section.data() {
            Ok(b) => b,
            Err(_) => continue,
        };

        let mut decoder = Decoder::with_ip(64, bytes, addr, DecoderOptions::NONE);
        while decoder.can_decode() {
            let instr = decoder.decode();
            features.extend(instr.cpuid_features().iter());
        }
    }

    Ok(features)
}
