use std::env;
use std::process::{self, Command};

fn main() {
	let status = Command::new("rust-lld")
		.args(env::args_os().skip(1))
		.status()
		.unwrap();

	process::exit(status.code().unwrap_or(1));
}
