use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

use iced_x86::{CpuidFeature, Decoder, DecoderOptions};
use object::{Object, ObjectSection};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Usage: <path-to-binary-or-folder>".to_string().into());
    }
    let input_path = &args[1];
    let path = Path::new(input_path);

    let mut binaries = Vec::new();
    if path.is_file() {
        binaries.push(input_path.to_string());
    } else if path.is_dir() {
        find_executables_recursively(path, &mut binaries)?;
    } else {
        return Err(format!("Path '{}' does not exist", input_path).into());
    }

    if binaries.is_empty() {
        println!("No executables found in {}", input_path);
        return Ok(());
    }

    if path.is_file() {
        let used_features = detect_instruction_sets(&binaries[0])?;
        println!("\nBinary {input_path} uses the following CPU features:");
        print_features(&used_features);
    } else {
        let mut all_features = [false; 256];
        for file_path in &binaries {
            println!(" - analyzing {}", file_path);
            match detect_instruction_sets(file_path) {
                Ok(used_features) => {
                    for (i, used) in used_features.iter().enumerate() {
                        if *used {
                            all_features[i] = true;
                        }
                    }
                }
                Err(e) => {
                    if e.downcast_ref::<object::Error>().is_some() {
                        // Ignore non-ELF or unsupported files
                        println!("   Skipping non-ELF or unsupported file: {}", file_path);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        println!("\nAll binaries in {input_path} use the following CPU features (union):");
        print_features(&all_features);
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

fn find_executables_recursively(
    dir: &Path,
    binaries: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_executables_recursively(&path, binaries)?;
        } else if path.is_file() {
            use std::os::unix::fs::PermissionsExt;
            let meta = entry.metadata()?;
            let perm = meta.permissions();
            // Check if owner, group, or others have execute permission
            if perm.mode() & 0o111 != 0 {
                binaries.push(path.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}

fn detect_instruction_sets(path: &str) -> Result<[bool; 256], Box<dyn Error>> {
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

    let binary = fs::read(path)?;
    // Try to parse as an object file first
    match object::File::parse(&*binary) {
        Ok(obj_file) => {
            let mut features = [false; 256];
            for section in obj_file.sections() {
                process_section(&mut features, &section);
            }
            Ok(features)
        }
        Err(e) => {
            // If not an object file, try to parse as an archive (ar) file
            if let Ok(archive) = object::read::archive::ArchiveFile::parse(&*binary) {
                let mut features = [false; 256];
                for member in archive.members() {
                    let member = member?;
                    if let Ok(obj_file) = object::File::parse(member.data(&*binary)?) {
                        for section in obj_file.sections() {
                            process_section(&mut features, &section);
                        }
                    }
                }
                Ok(features)
            } else {
                Err(Box::new(e))
            }
        }
    }
}
