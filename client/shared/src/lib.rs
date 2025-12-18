use std::env;
use std::path::Path;

#[derive(Clone, Copy)]
enum Architecture {
    Wasm32,
    Wasm64,
}

pub fn link_wasm_objects() {
    let arch = match env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap().as_str() {
        "32" => Architecture::Wasm32,
        "64" => Architecture::Wasm64,
        _ => unreachable!(),
    };
    let src = Path::new(&env::var_os("CARGO_MANIFEST_PATH").unwrap())
        .parent()
        .unwrap()
        .join("src");

    search_directory(arch, &src);
}

fn search_directory(arch: Architecture, directory: &Path) {
    for entry in directory.read_dir().unwrap() {
        let entry = entry.unwrap();
        let r#type = entry.file_type().unwrap();

        if r#type.is_dir() {
            search_directory(arch, &entry.path());
        } else if r#type.is_file() {
            let file = entry.file_name();
            let file = Path::new(&file);

            if let Some(extension) = file.extension() {
                if extension != "wasm" {
                    continue;
                }
            }

            if let Some(extension) = Path::new(file.file_stem().unwrap()).extension() {
                match arch {
                    Architecture::Wasm32 => {
                        if extension != "32" {
                            continue;
                        }
                    }
                    Architecture::Wasm64 => {
                        if extension != "64" {
                            continue;
                        }
                    }
                }
            }

            println!("cargo:rustc-link-arg-cdylib={}", entry.path().display());
        }
    }
}
