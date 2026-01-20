A Rust port of the **[Bound Book Format (BBF)](https://github.com/ef1500/libbbf)** library.

> **Credits:** This project is a direct port of the original `libbbf` by [ef1500](https://github.com/ef1500). It might differ slightly in failure modes.

## Features

* **Safety First:** Built-in protection against Zip-Bombs, arithmetic overflows, and FFI panic boundaries.
* **Portable:** Pure Rust implementation with no C++ runtime dependencies.
* **WASM Ready:** Generic I/O traits (`Read + Seek`) allow for easy integration with WebAssembly.
* **Zero-Copy:** Utilizes `zerocopy` for safe, high-speed parsing of binary structures.
* **C ABI:** Compiles to a static/dynamic library (`cdylib`) for seamless integration with C/C++ projects.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libbbf-rs = { git = "https://github.com/thmasq/sqlite-wasm-reader", tag = "v0.1.0" } # Or whatever is the latest release
```

## License

Distributed under the MIT License. See `LICENSE` for more information.
