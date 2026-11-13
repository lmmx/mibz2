# Installation

## Python

mibz2 needs Python 3.10 or later. It has no Python dependencies.

```sh
pip install mibz2
```

or with uv:

```sh
uv add mibz2
```

Wheels are built for Linux (x86-64, aarch64), macOS (x86-64, arm64) and Windows (x64). Each uses the stable ABI (abi3), so one wheel covers every Python version from 3.10.

## Rust

```sh
cargo add mibz2
```

The library depends on std only. Its minimum supported Rust version is 1.80. The `python` feature builds the Python extension module, and is only for maturin.
