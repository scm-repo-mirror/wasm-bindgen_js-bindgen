# Current

- Basic runner.

# Critical Priority

- Replace `llvm-mc` with `wasm-tools`. See [bytecodealliance/wasm-tools#2405].

[bytecodealliance/wasm-tools#2405]: https://github.com/bytecodealliance/wasm-tools/issues/2405

# High Priority

- Allocate slots on the `externref` table in batches.
- Figure out what to do with the panic optimization.
- Experiment if allocation is better for build times then iterator chaining in proc-macros.
- Find a way to prevent users from accidentally using the default linker. Could be done by supplying
  an invalid object file that would be removed by our custom linker.
- Also find a way to prevent users from accidentally using our linker with something else then Wasm.
- Version all names to make packages compatible with other versions of itself.
- Embed crate version to make linker capable of detecting unsupported versions.
- Add tracking for ASM object files in the linker, so we don't re-generate them each time.
- Evaluate the output folder of our ASM objet files. Some ideas:
  - Store them next to the output file.
  - Pass an environment variable from a `build.rs` pointing to the target folder and go from there.
    This seems to have failed. No build script instruction can reach the linker on Wasm.

# Medium Priority

- Provide an absolutely minimal allocator.
- The `js_sys` proc-macro should remove the `extern "C" { ... }` part of the input on error.
- Optimize linker file interactions by using memory mapped files instead of reading and writing
  everything into memory.
- Run the assembly compiler on the proc-macro level so users see errors without having to engage the
  linker.
- Parse the JS on the proc-macro level so users see errors. E.g. `oxc-parser`.
- Use an AST for JS code generation so we don't play around with strings. E.g. `oxc-codegen`.

# Low Priority

- Linker functionality should live in its own crate so a newer linker versions can support multiple
  versions.
- We have a custom `LocalKey` replica for non-atomic or non-std builds. It differs because its
  methods don't take a `'static` lifetime. It would probably be easiest to just align actual Std's
  `LocalKey` unsafely to not require `'static`.
- Can we remove custom sections in pre-processing by modifying `.rlib`s?
- Re-evaluate caching via the linker.
- Polish LLD linker argument parsing.

# Upstream

This is a list of upstream issues that could make our lives significantly easier:

- LLVM 22 delivers support for the GC proposal, with which we can implement the `externref` table
  much more efficiently.
- Stable `asm!` support for Wasm: [rust-lang/rust#136382].
- `asm!` support with target features: [rust-lang/rust#113221]
- Verbatim `asm!` parameters: [rust-lang/rust#132083].
- Better stable proc-macro support
  - `quote!`: [rust-lang/rust#54722].
  - Diagnostics: [rust-lang/rust#54140].
  - Execution in non-proc-macro crates: [rust-lang/rust#130856].
- Elevate `wasm64-unknown-unknown` to tier 2: [rust-lang/rust#146944].
- A way to flag proc-macros as `unsafe`: [rust-lang/rfcs#3715].
- Link internal functions without exporting them: [rust-lang/rust#29603] or [rust-lang/rfcs#3834].
- Safe slice to array conversion: [rust-lang/rust#133508]

[rust-lang/rust#136382]: https://github.com/rust-lang/rust/issues/136382
[rust-lang/rust#113221]: https://github.com/rust-lang/rust/issues/113221
[rust-lang/rust#132083]: https://github.com/rust-lang/rust/issues/132083
[rust-lang/rust#54722]: https://github.com/rust-lang/rust/issues/54722
[rust-lang/rust#54140]: https://github.com/rust-lang/rust/issues/54140
[rust-lang/rust#130856]: https://github.com/rust-lang/rust/issues/130856
[rust-lang/rust#29603]: https://github.com/rust-lang/rust/issues/29603
[rust-lang/rfcs#3834]: https://github.com/rust-lang/rfcs/pull/3834
[rust-lang/rust#146944]: https://github.com/rust-lang/rust/issues/146944
[rust-lang/rust#133508]: https://github.com/rust-lang/rust/issues/133508
[rust-lang/rfcs#3715]: https://github.com/rust-lang/rfcs/pull/3715
