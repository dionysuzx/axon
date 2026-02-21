# Axon

Axon is a Rust CLI for validating and refactoring prompt filename conventions.

## Prerequisites

- [yazi](https://yazi-rs.github.io/) - terminal file manager, used to open notes
- [fzf](https://github.com/junegunn/fzf) - fuzzy finder, used by TUI search

## Install

From this repo:

```bash
cargo install --path .
```

Or build a local binary:

```bash
cargo build --release
cp target/release/axon ~/.local/bin/
```

## Usage

Invoke:

```bash
axon --help
```
