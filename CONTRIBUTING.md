# コントリビューションガイド / Contributing

musubi への貢献を歓迎します！バグ報告・機能提案・ドキュメント改善・コード貢献、いずれも大歓迎です。

> *Contributions are welcome. This document is in Japanese; feel free to open issues in English.*

## はじめに

- バグや要望は [Issues](https://github.com/masaki-09/musubi/issues) からどうぞ。
- 大きめの変更を加える前に、Issue や Discussions で方針を相談していただけると安心です。
- すべての参加者は [行動規範](CODE_OF_CONDUCT.md) に同意するものとします。

## 開発環境

| 必須 | 用途 |
|---|---|
| Rust 1.75+ (stable) | コンパイル |
| `cargo` | ビルド/テスト |
| `wasm-pack`（任意） | WebAssembly ビルド |

```bash
git clone git@github.com:masaki-09/musubi.git
cd musubi
cargo build --workspace
cargo test  --workspace
```

## 変更を提案する流れ

1. リポジトリを **Fork** してください（外部コントリビューター）。コラボレーターは直接ブランチを切ってください。
2. ブランチ名は目的を表すプレフィックス付きで：

   | プレフィックス | 用途 | 例 |
   |---|---|---|
   | `feat/`  | 機能追加         | `feat/cli-encrypt`     |
   | `fix/`   | バグ修正         | `fix/empty-input-panic` |
   | `docs/`  | ドキュメント     | `docs/spec-clarify`     |
   | `chore/` | 雑務・基盤       | `chore/ci-cache`        |
   | `refactor/` | リファクタ    | `refactor/alphabet-iter` |
   | `test/`  | テスト追加・修正 | `test/cipher-vectors`   |

3. ローカルで以下が通ることを確認：

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

4. **Conventional Commits** 形式でコミット：

   ```
   feat(core): add Alphabet::from_chars constructor

   Allow building an Alphabet from any iterator of chars rather than
   only from the built-in default set.
   ```

   - `type(scope): subject` 形式
   - `type` は `feat` / `fix` / `docs` / `chore` / `refactor` / `test` / `perf` / `style` / `ci`
   - `scope` は `core` / `cli` / `wasm` / `web` / `ci` 等

5. push して Pull Request を `main` 向けに作成：

   ```bash
   gh pr create --base main --fill
   ```

   PR テンプレートのチェックリストを埋めてください。

6. レビュー → CI 緑 → squash merge。

## コードスタイル

- Rust: `rustfmt` と `clippy::pedantic` を遵守
- 公開 API には rustdoc コメント必須（`#![warn(missing_docs)]`）
- `unsafe` は使わない（`#![forbid(unsafe_code)]`）

## テスト

- 単体テストは各モジュール内 `mod tests { ... }`
- 結合テストは `crates/<crate>/tests/`
- 暗号アルゴリズムには **テストベクタ** (固定鍵・固定平文に対する固定暗号文) を必ず追加

## 暗号としての位置づけ

musubi は **古典暗号の楽しみのためのソフトウェア** であり、本気の秘匿通信を意図していません。「現代暗号として強くする」方向の貢献は、議論の上で慎重に検討します。一方、**理論の美しさ・実装の厳格さ・UI のロマン** を高める貢献はいつでも大歓迎です。

ありがとうございます 🪢
