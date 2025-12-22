use std::env;
use std::path::Path;

fn main() {
	let manifest_dir =
		env::var_os("CARGO_MANIFEST_DIR").expect("`CARGO_MANIFEST_DIR` should be present");
	js_bindgen_bootstrap::bootstrap(&Path::new(&manifest_dir).join("src").join("cache"));
}
