use std::io::Write;
use std::path::Path;
use std::process::{self, Command, Stdio};
use std::{env, fs};

use wasm_encoder::{Module, RawSection, Section};
use wasmparser::{Encoding, Parser, Payload};

fn main() {
	let mut asm_args = Vec::new();

	for arg in env::args() {
		let object_path = if !arg.starts_with('-') && arg.ends_with(".o") {
			Path::new(&arg)
		} else {
			continue;
		};

		let mut asm_counter = 0;

		let input = fs::read(object_path).expect("object file should be readable");
		let mut output = Vec::new();

		for payload in Parser::new(0).parse_all(&input) {
			let payload = payload.expect("object file should be valid Wasm");

			match payload {
				Payload::Version { encoding, .. } => output.extend_from_slice(match encoding {
					Encoding::Module => &Module::HEADER,
					Encoding::Component => {
						unimplemented!("objects with components are not supported")
					}
				}),
				Payload::CustomSection(c) if c.name() == "js_bindgen.assembly" => {
					for assembly in c.data().split(|b| b == &b'\0').filter(|a| !a.is_empty()) {
						let asm_object = assembly_to_object(assembly);

						let asm_path =
							object_path.with_added_extension(format!("asm.{asm_counter}.o"));
						asm_counter += 1;
						fs::write(&asm_path, asm_object)
							.expect("writing ASM object file should succeed");

						asm_args.push(asm_path);
					}
				}
				Payload::CodeSectionEntry(_) | Payload::End(_) => (),
				_ => {
					if let Some((id, range)) = payload.as_section() {
						RawSection {
							id,
							data: &input[range],
						}
						.append_to(&mut output);
					} else {
						unimplemented!("encountered unknown Wasm payload: {payload:?}")
					}
				}
			}
		}

		fs::write(object_path, output).expect("object file should be writable");
	}

	let status = Command::new("rust-lld")
		.args(env::args_os().skip(1))
		.args(asm_args)
		.status()
		.unwrap();

	process::exit(status.code().unwrap_or(1));
}

fn assembly_to_object(assembly: &[u8]) -> Vec<u8> {
	let mut child = Command::new("llvm-mc")
		.arg("-arch=wasm32")
		.arg("-mattr=+reference-types")
		.arg("-filetype=obj")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.stdin(Stdio::piped())
		.spawn()
		.expect("calling `llvm-mc` should succeed");

	let stdin = child
		.stdin
		.as_mut()
		.expect("`llvm-mc` process should have `stdin`");
	stdin
		.write_all(assembly)
		.expect("copying to `llvm-mc`'s `stdin` should succeed");

	let output = child.wait_with_output().expect("`llvm-mc` should succeed");

	if output.status.success() {
		output.stdout
	} else {
		let mut error = format!("`llvm-mc` process failed with status: {}\n", output.status);

		if !output.stdout.is_empty() {
			error.push_str("\n------ llvm-mc stdout ------\n");
			error.push_str(&String::from_utf8_lossy(&output.stdout));

			if !output.stdout.ends_with(b"\n") {
				error.push('\n');
			}
		}

		if !output.stderr.is_empty() {
			error.push_str("\n------ llvm-mc stderr ------\n");
			error.push_str(&String::from_utf8_lossy(&output.stderr));

			if !output.stderr.ends_with(b"\n") {
				error.push('\n');
			}
		}

		panic!("{error}");
	}
}
