# Changelog

All notable changes to **musubi** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cargo workspace scaffold with `musubi-core`, `musubi-cli`, `musubi-wasm` crates.
- MIT License, README (Japanese), `.gitignore`, `.editorconfig`, `rustfmt.toml`.
- OSS contribution templates: `CONTRIBUTING`, `CODE_OF_CONDUCT`, `SECURITY`, issue & PR templates.
- Dependabot configuration for Cargo and GitHub Actions.
- GitHub Actions CI: rustfmt, clippy (`-D warnings`), multi-OS tests (Linux/macOS/Windows), `wasm32-unknown-unknown` build, strict rustdoc.
- Branch protection on `main`: pull-request only, required CI checks, linear history, no force pushes.
- `musubi-core`: full v1 cipher implementation — `Alphabet`, `Key` (with random + JSON I/O), `Relation`, `Ciphertext`, `encrypt`, `decrypt`. Default alphabet `default-v1` (175 chars: 五十音80 + ASCII 95).
- Formal cipher specification at [`docs/SPEC.md`](docs/SPEC.md) and theory writeup at [`docs/THEORY.md`](docs/THEORY.md).
- `musubi-cli` (binary `musubi`): `keygen` / `encrypt` / `decrypt` subcommands with stdin/stdout streaming, file I/O via `-i`/`-o`, optional `--seed` for reproducible keys, `--anchor` for explicit anchor positioning, and `--compact` for single-line ciphertext JSON.
- End-to-end CLI integration tests using `assert_cmd` covering round-trip, Japanese plaintext, error paths, and seeded determinism.

[Unreleased]: https://github.com/masaki-09/musubi/compare/...HEAD
