# Wurth Telesto

Rust support for the Wurth Electronics Telesto modules (I, II and III) as well as a handy CLI tool.

## Using the Rust crate

To use the library in your own project.

```shell
cargo add wurth-telesto
```

### Features

- `defmt-03` enabled defmt traits for this crate and dependencies that support it.

## Installing the CLI

```shell
cargo install --git https://github.com/team-arrow-racing/wurth-telesto.git --features="cli"
```

To run use the CLI with `cargo run` use:

```shell
cargo run --features cli -- <args here>
```
