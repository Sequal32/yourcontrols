# Setting up the workspace
For some reason the MSVC compiler wouldn't work for wasm32, so getting the GNU toolchain and running the below command should get things running.

`rustup override set stable-x86_64-pc-windows-gnu`

# Compiling
Create a Cargo.toml file in a .cargo folder with the following contents:

```toml
[target.wasm32-wasi]
rustflags = [
  "-Clink-arg=--export-table",
  "-Clink-arg=--export=malloc",
  "-Clink-arg=--export=free",
]

[build]
target = "wasm32-wasi"
```

Run `cargo build`
`yourcontrols_gauge.wasm` will be created in `target/release/wasm32-wasi/debug/yourcontrols_gauge.wasm`