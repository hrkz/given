# Given

[![License](https://img.shields.io/github/license/hrkz/given?style=flat-square&color=blue)](https://github.com/hrkz/given/blob/main/LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/hrkz/given/build.yml?style=flat-square)](https://github.com/hrkz/given/actions/workflows/build.yml)

An experimental project based on a modern and flexible implementation of the [E-graph](https://en.wikipedia.org/wiki/E-graph) data structure.
Unlike most projects focused primarily on compiler optimization, Given is designed for symbolic computation and reasoning. Its goals include:

1. Performing symbolic computations across different mathematical areas.
2. Generating readable mathematical proofs following rules applications.
3. Exploring strategies for integrating modern (and lightweight) LLMs into mathematical reasoning workflows.

> [!WARNING]
> Given is a side project and is not expected to reach a stable state anytime soon.

For more details, contact me.

## Getting started

You can either add Given as a **local** dependency (see Cargo’s documentation on [specifying dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)), or clone the repository and run the examples directly:

```bash
# Switch to the latest release
git checkout latest
# Runs the "name" example
cargo run --example name
```

For now, Given only runs on nightly Rust but should build just fine on stable once some features are stabilized. 
The standard cargo `fmt`, `clippy` and `test` workflow is available.

## In-progress content

> [!IMPORTANT]
> **The objective is to provide a standard CLI executable and a web demo as soon as possible, but also:**

* A full documentation.
* An open markdown containing notes.
* A community channel.
