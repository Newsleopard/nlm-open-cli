# Contributing to nlm-cli

We welcome contributions! Here's how to get started.

## Development Setup

```bash
git clone https://github.com/Newsleopard/nlm-open-cli.git
cd nlm-open-cli
cargo build
cargo test
```

**Rust 1.75+** is required.

## Code Quality

```bash
cargo fmt -- --check    # Formatting
cargo clippy -- -D warnings  # Linting
cargo test              # Tests
```

All three checks run in CI on every push and PR.

## Pull Request Process

1. Fork the repo and create a branch from `main`.
2. Add tests for new functionality.
3. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass.
4. Write a clear PR description.

## Reporting Issues

- Use the bug report or feature request issue templates.
- Include `nl --version` output and your OS when reporting bugs.
- Use `--dry-run` to capture the request details if relevant.

## License

By contributing, you agree that your contributions will be dual-licensed under [MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE).
