@echo off
cargo run --manifest-path "..\host\linker\Cargo.toml" -- %*
exit /b %ERRORLEVEL%
