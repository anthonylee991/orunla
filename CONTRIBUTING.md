# Contributing to Orunla

Thanks for your interest in contributing. This project is maintained on a best-effort basis.

## Development Setup

### Prerequisites

- Rust toolchain (stable)
- Node.js 22+
- ONNX Runtime 1.20.0 (see CI workflow for download steps)

### Building

```bash
# CLI and MCP server
cargo build --release --bin orunla_cli
cargo build --release --bin orunla_mcp

# Desktop app (Tauri)
npm ci
npm run tauri build
```

### Running Tests

```bash
cargo test
```

## Code Style

- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Keep functions focused and small
- Avoid unnecessary abstractions

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Run `cargo test`
6. Commit with a clear message describing the change
7. Open a pull request against `master`

## Pull Request Guidelines

- Keep PRs focused on a single change
- Include a description of what changed and why
- Add tests for new functionality where practical
- Ensure CI passes

## Reporting Issues

Open a GitHub issue with:
- Steps to reproduce
- Expected vs actual behavior
- Platform (Windows/macOS/Linux) and version

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
