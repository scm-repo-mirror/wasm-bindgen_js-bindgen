use std::env;

use ui_test::color_eyre::Result;
use ui_test::dependencies::DependencyBuilder;
use ui_test::Config;

fn main() -> Result<()> {
	let mut config = Config::rustc("tests/ui");
	config
		.comment_defaults
		.base()
		.set_custom("dependencies", DependencyBuilder::default());

	if env::var_os("BLESS").filter(|v| v == "1").is_some() {
		config.output_conflict_handling = ui_test::bless_output_files;
	}

	ui_test::run_tests(config)
}
