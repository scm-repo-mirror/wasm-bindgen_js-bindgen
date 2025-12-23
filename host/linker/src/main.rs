use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::{env, fs};

use object::read::archive::ArchiveFile;
use wasm_encoder::{Module, RawSection, Section};
use wasmparser::{Encoding, Parser, Payload};

fn main() {
	let args: Vec<_> = env::args_os().collect();

	let arch = args
		.iter()
		.find_map(|a| {
			a.to_str().and_then(|a| {
				a.strip_prefix("-m")
					.filter(|a| !a.starts_with('='))
					.map(str::to_owned)
			})
		})
		.unwrap_or_else(|| String::from("wasm32"));
	let output_path = args
		.iter()
		.enumerate()
		.find_map(|(i, a)| (a == "-o").then_some(i))
		.and_then(|i| args.get(i + 1))
		.map(Path::new)
		.expect("output path argument should be present");

	let mut asm_args: Vec<PathBuf> = Vec::new();

	for arg in &args {
		if arg.as_encoded_bytes().starts_with(b"-") {
			continue;
		}

		if arg.as_encoded_bytes().ends_with(b".rlib") {
			let archive_path = Path::new(&arg);
			let archive_data = fs::read(archive_path).expect("object file should be readable");
			let archive =
				ArchiveFile::parse(&*archive_data).expect("`*.rlib` should be a valid archive");

			for member in archive.members() {
				let member = member.expect("archive member should be valid");

				let name = str::from_utf8(member.name())
					.expect("archive name should be valid UTF-8")
					.to_owned();

				let object = member
					.data(&*archive_data)
					.expect("archive member should be extractable");

				process_object(
					&arch,
					&mut asm_args,
					&archive_path.with_file_name(name),
					object,
				);
			}
		} else if arg.as_encoded_bytes().ends_with(b".o") {
			let object_path = Path::new(&arg);
			let object = fs::read(object_path).expect("object file should be readable");

			process_object(&arch, &mut asm_args, object_path, &object);
		}
	}

	let status = Command::new("rust-lld")
		.args(env::args_os().skip(1))
		.args(asm_args)
		.status()
		.unwrap();

	if status.success() {
		post_processing(output_path)
	}

	process::exit(status.code().unwrap_or(1));
}

fn process_object(arch: &str, asm_args: &mut Vec<PathBuf>, archive_path: &Path, object: &[u8]) {
	let mut asm_counter = 0;

	for payload in Parser::new(0).parse_all(object) {
		let payload = payload.expect("object file should be valid Wasm");

		if let Payload::CustomSection(c) = payload
			&& c.name() == "js_bindgen.assembly"
		{
			for assembly in c.data().split(|b| b == &b'\0').filter(|a| !a.is_empty()) {
				let asm_object =
					assembly_to_object(arch, assembly).expect("compiling ASM should be valid");

				let asm_path = archive_path.with_added_extension(format!("asm.{asm_counter}.o"));
				asm_counter += 1;
				fs::write(&asm_path, asm_object).expect("writing ASM object file should succeed");

				asm_args.push(asm_path);
			}
		}
	}
}

fn assembly_to_object(arch: &str, assembly: &[u8]) -> Result<Vec<u8>, Error> {
	let mut child = Command::new("llvm-mc")
		.arg(format!("-arch={arch}"))
		.arg("-mattr=+reference-types,+call-indirect-overlong")
		.arg("-filetype=obj")
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.stdin(Stdio::piped())
		.spawn()?;

	let stdin = child
		.stdin
		.as_mut()
		.ok_or_else(|| Error::other("`llvm-mc` process should have `stdin`"))?;
	stdin.write_all(assembly)?;

	let output = child.wait_with_output()?;

	if output.status.success() {
		Ok(output.stdout)
	} else {
		if !output.stdout.is_empty() {
			eprintln!(
				"------ llvm-mc stdout ------\n{}",
				String::from_utf8_lossy(&output.stdout)
			);

			if !output.stdout.ends_with(b"\n") {
				eprintln!();
			}
		}

		if !output.stderr.is_empty() {
			eprintln!(
				"------ llvm-mc stderr ------\n{}",
				String::from_utf8_lossy(&output.stderr)
			);

			if !output.stderr.ends_with(b"\n") {
				eprintln!();
			}
		}

		Err(Error::other(format!(
			"`llvm-mc` process failed with status: {}\n",
			output.status
		)))
	}
}

fn post_processing(output_path: &Path) {
	let input = fs::read(output_path).expect("output file should be readable");

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
			Payload::CustomSection(c) if c.name() == "js_bindgen.assembly" => (),
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

	fs::write(output_path, output).expect("object file should be writable");
}
