use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

use iced_x86::{CpuidFeature, Decoder, DecoderOptions};
use object::{read::archive::ArchiveFile, FileKind, Object, ObjectSection};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: <path-to-binary-or-folder>".to_string().into());
    }
    let input_path = &args[1];
    let path = Path::new(input_path);

    let mut features = [false; 256];
    if path.is_file() {
        detect_instruction_sets(path, &mut features)?;
        println!("\nBinary {input_path} uses the following CPU features:");
        print_features(&features);
    } else if path.is_dir() {
        println!("Processing directory: {}", input_path);
        process_code_files_recursively(path, &mut features)?;
        println!("\nAll binaries in {input_path} use the following CPU features:");
        print_features(&features);
    } else {
        return Err(format!("Path '{}' is neither a file nor a directory", input_path).into());
    }
   
    Ok(())
}

fn print_features(features: &[bool; 256]) {
    for (i, &used) in features.iter().enumerate() {
        if used {
            let feature = unsafe { std::mem::transmute::<u8, CpuidFeature>(i as u8) } ;
            println!("  {:?}", feature);
        }
    }
}

fn process_code_files_recursively(
    dir: &Path,
    features: &mut [bool; 256],
) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            process_code_files_recursively(&path, features)?;
        } else if path.is_file() {
            detect_instruction_sets(path, features)?;
        }
    }
    Ok(())
}

fn detect_instruction_sets(path: impl AsRef<Path>, features: &mut [bool; 256]) -> Result<(), Box<dyn Error>> {
    println!("Processing binary: {:?}", path.as_ref());
    let binary = fs::read(path)?;

    match FileKind::parse(&binary[..]) {
        Ok(FileKind::Elf32 | FileKind::Elf64 | FileKind::DyldCache) => {
            detect_features_in_object(&binary, features)
        }
        Ok(FileKind::Archive) => {
            detect_features_in_archive(&binary, features)
        }
        _ => {
            // Ignore unknown files;
            Ok(())
        }
    }
}

fn detect_features_in_object(binary: &[u8], features: &mut [bool; 256]) -> Result<(), Box<dyn Error>> {
    let obj_file = object::File::parse(binary)?;
    for section in obj_file.sections() {
        process_section(features, &section);
    }
    Ok(())
}

fn detect_features_in_archive(
    binary: &[u8],
    features: &mut [bool; 256]
) -> Result<(), Box<dyn Error>> {
    let archive = ArchiveFile::parse(binary)?;
    for member in archive.members() {
        let member = member?;
        if let Ok(obj_file) = object::File::parse(member.data(binary)?) {
            for section in obj_file.sections() {
                process_section(features, &section);
            }
        }
    }
    Ok(())
}

fn process_section(features: &mut [bool; 256], section: &object::Section) {
    if section.kind() != object::SectionKind::Text {
        return;
    }
    let addr = section.address();
    let bytes = match section.data() {
        Ok(b) => b,
        Err(_) => return,
    };
    let mut decoder = Decoder::with_ip(64, bytes, addr, DecoderOptions::NONE);
    while decoder.can_decode() {
        let instr = decoder.decode();
        for feature in instr.cpuid_features() {
            features[unsafe { std::mem::transmute::<CpuidFeature, u8>(*feature) } as usize] = true;
        }
    }
}
