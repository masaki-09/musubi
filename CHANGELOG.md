# Changelog

All notable changes to **musubi** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] — 2026-04-28 — 「織り (ori)」

The "weaving" release. Two backward-compatible encoder extensions
introduce richer ciphertext shapes while keeping `version: 1` on disk.

### Added
- **多重結び (chain encoder)** — new `musubi_core::encrypt_chain(plaintext, key, anchor, rng)`. Produces a uniformly random spanning tree rooted at the anchor instead of the canonical 1-step adjacency graph. Decoder unchanged.
- **迷い糸 (noise injection)** — new `musubi_core::encrypt_woven(plaintext, key, anchor, noise, rng)`. Interleaves `noise` dummy characters with the real plaintext, hiding the true plaintext length from anyone without the key.
- **`Ciphertext::ext` / `CiphertextExt::plaintext_indices`** — optional v0.2 extension field. Omitted from JSON when absent, so noise-free ciphertexts stay v0.1-compatible.
- **`musubi encrypt --strategy <canonical|chain>`** — pick the encoder.
- **`musubi encrypt --noise <N>`** — inject `N` dummy characters.
- **`musubi encrypt --seed <u64>`** — reproducible chain/noise RNG.
- **`musubi-wasm`** export `encryptWoven(plaintext, keyJson, anchor?, noise?, seed?)`.
- **WebUI 織りモード** — collapsible "advanced" panel with a 多重結び checkbox and 迷い糸の本数 input.
- **Spec sections** — `docs/SPEC.md` §5.3 (`ext` field), §6.5 (chain encoder), §6.6 (woven encoder), §7.1 (decryption with `plaintext_indices`).

### Changed
- `musubi_core::decrypt` now auto-detects `ext.plaintext_indices` and re-assembles the plaintext in the recorded order. v0.1-shaped ciphertexts decode unchanged.

### Compatibility
- `FORMAT_VERSION` stays at `1`. v0.1 ciphertexts decode unchanged with v0.2.
- v0.2 ciphertexts produced **without** `--noise` are byte-compatible with v0.1 decoders.
- v0.2 ciphertexts produced **with** `--noise > 0` carry an `ext` field and require a v0.2 decoder.

## [0.1.0] — 2026-04-27

First public release. The cipher, the CLI, and the browser UI all ship together.

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
- `musubi-wasm`: WebAssembly bindings exposing `keygen` / `encrypt` / `decrypt` to JavaScript via `wasm-bindgen`. Uses `crypto.getRandomValues` in the browser via `getrandom`'s `js` feature.
- `web/`: zero-server static frontend (`index.html` + `app.js` + `style.css`) wiring the WASM module to a tabbed UI for keygen / encrypt / decrypt.
- `.github/workflows/pages.yml`: builds the WASM bundle with `wasm-pack` and deploys `web/` to GitHub Pages on every push to `main`.
- CI's `wasm32 build` job now uses `wasm-pack` so PR validation matches the production deploy pipeline.

[Unreleased]: https://github.com/masaki-09/musubi/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/masaki-09/musubi/releases/tag/v0.2.0
[0.1.0]: https://github.com/masaki-09/musubi/releases/tag/v0.1.0
