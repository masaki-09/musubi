<h1 align="center">musubi（結び）</h1>

<p align="center">
  <em>A relational cipher for the romantically inclined.</em><br>
  <em>関係性暗号 — 文字を「結ぶ」ことで意味を伝える、独自理論の古典暗号。</em>
</p>

<p align="center">
  <a href="https://github.com/masaki-09/musubi/actions/workflows/ci.yml"><img src="https://github.com/masaki-09/musubi/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/masaki-09/musubi" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust 1.75+">
</p>

---

## これは何？

**musubi**（結び）は、独自の暗号理論「**関係性暗号** (relational cipher)」を実装した古典暗号ライブラリです。

通常の暗号が「文字を別の文字に置き換える（置換）」か「文字の順序を入れ替える（転置）」のに対し、関係性暗号は **文字そのものを書きません**。代わりに「文字同士の関係」だけを記述し、たった一つの **アンカー文字** から芋づる式に平文を復元します。

> **例**：平文「あいしてる」
>
> 暗号「3番目は『し』。1番目はそこから2つ後。2番目は1番目の1つ前。…」

## ⚠ ロマン暗号としての位置づけ

本ソフトウェアは **ロマンと好奇心のための玩具暗号** です。本気の秘匿通信や情報保護には絶対に使わないでください。現代の暗号解析にとっては紙のように脆弱です。恋文・パズル・自作古典暗号の楽しみのためにご利用ください。

## クイックスタート

### CLI

```bash
cargo install --git https://github.com/masaki-09/musubi musubi-cli
musubi keygen > my.key
echo "あいしてる" | musubi encrypt -k my.key
```

### Web

ブラウザで <https://masaki-09.github.io/musubi/> を開いてください。サーバ不要、すべてローカルで完結します。

## 理論と仕様

- 形式仕様: [`docs/SPEC.md`](docs/SPEC.md)
- 理論解説: [`docs/THEORY.md`](docs/THEORY.md)

## 開発

```bash
git clone git@github.com:masaki-09/musubi.git
cd musubi
cargo build --workspace
cargo test  --workspace
```

### ワークスペース構成

| クレート | 役割 |
|---|---|
| [`musubi-core`](crates/core)   | 暗号アルゴリズム本体（純粋ロジック） |
| [`musubi-cli`](crates/cli)     | コマンドラインインターフェース（バイナリ名: `musubi`） |
| [`musubi-wasm`](crates/wasm)   | WebAssembly バインディング |

## コントリビュート

Issues / PR を歓迎します。詳細は [CONTRIBUTING](CONTRIBUTING.md) と [行動規範](CODE_OF_CONDUCT.md) を参照。

## ライセンス

MIT License — 詳細は [LICENSE](LICENSE) を参照。
