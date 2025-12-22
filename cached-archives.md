# Cached Archives

This describes how ``cached_embed_asm!` works. Its function is to cache assembly to archive conversion right into the crate files to reduce compile times.

## Bootstrap Mode

While developing a crate, archive files have to be continuously updated to adapt to code changes. To that effect we introduce a `cfg(js_bindgen_bootstrap = "1")` flag that enables a bootstrap mode with this behavior.

When using `cached_embed_asm!` the first time without enabling bootstrap mode, the user will encounter build errors.

If bootstrap mode is neglected archive files might become out-of-date. To prevent this we check each files modified time against its source file.

During development Rust Analyzer causes race conditions that can cause checks to fail. Until we can resolve this you can opt-out of those checks via the environment variable `JS_BINDGEN_CACHE_DISABLE_CHECK = 1`.

## Implementation

### `build.rs`

Registers archive directory with the linker depending on the target and target features.

#### Bootstrap Mode

Deletes all generated archives to prepare for the proc-macro to generate new ones.
It also provides a path to the archive directory to the proc-macro through setting an environment variable. The environment variable has to be keyed with the crate name and version to prevent conflict with other crate's `build.rs`.

### Proc-Macro

Links the archive file.

#### Bootstrap Mode

Generate archive files from the input and stores them in the archive directory. The directory is provided by `build.rs` and passed through an environment variable.

Generated archive files modified times are set to the same modified time as the source file to allow for up-to-date checks in `build.rs`.

## **Untested** Split Archives Into Crates

Archives can be split into separate crates that can be individually depended on to reduce crate size.
`build.rs` implementations must live in those crates to be able to generate the correct archive directory path.

However, for bootstrap mode to work the crate name and version have to be kept in sync with the crate the proc-macro is being called from.

## TODOs

- Measure how much time archive caching is really saving us.
- Measure how expensive it is to check if archive caches are up-to-date.
  Potentially find alternative solutions or disable entirely.
- Test if `build.rs` in dedicated archive dependencies is properly re-run after a change in the crate actually calling the proc-macros.
- Consider fallback to non-cached behavior when archives are not present and emit a warning.
- Proc-macro can't receive bootstrapping `cfg` when cross-compiling. This is a major issue!
