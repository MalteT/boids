# [WIP] [Boids](http://www.red3d.com/cwr/boids/) in [WebAssembly](https://webassembly.org/)


## Compiling & Running

You'll need the [Rust](https://www.rust-lang.org/) compiler (and `cargo`), preferably installed using [rustup.rs](https://rustup.rs), [`wasm-pack`](https://lib.rs/crates/wasm-pack) and a server (i.e. [`miniserve`](https://lib.rs/crates/miniserve)).
You can install `wasm-pack` and `miniserve` using `cargo`:
```console
$ cargo install wasm-pack
$ cargo install miniserve
```

Then compile the code to WebAssembly using:
```console
$ wasm-pack build --target web --out-name wasm --out-dir ./docs/wasm/
```

Serve everything using:
```console
$ miniserve static
```

You should find the Boids at [localhost:8080/index.html](localhost:8080/index.html) now.
