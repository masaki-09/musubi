# Proposal: musubi v0.2 — 「織り (ori)」

**Status:** Draft
**Author:** masaki-09
**Target release:** v0.2.0
**Compatibility:** Backward-compatible with v0.1.0 ciphertexts; `FORMAT_VERSION` stays at `1`.

---

## 0. Abstract

v0.1 の musubi は、アンカーから1ステップ近い隣接文字を参照する **正準 (canonical) エンコーダ** だけを規定した。本提案では、暗号文の表現力とロマンを拡張する2つの非互換のないエンコーダ拡張を導入する。

- **案A:「多重結び」(chain encoder)**
  全ての文字が必ず隣接文字を参照するのではなく、アンカーから出発する **任意のランダム木** に沿って関係を結ぶ。一本の赤い糸が文字から文字へ縫い進むイメージ。
- **案B:「迷い糸」(noise injection)**
  暗号文に、平文に含まれない **ダミー文字** とその関係をいくつか織り込む。鍵を持つ受信者はアンカーから到達可能なノードだけを辿るため正しく復号できるが、鍵を持たない者からは平文の長さすら判別できない。

両者は独立に有効化でき、組み合わせ可能。デコーダはどちらの場合も既存のトポロジカルソート (§7 of SPEC) でそのまま動く。

## 1. 動機

### 1.1 現状の限界

正準エンコーダの出力は、

- 関係グラフが **アンカーを中心としたパス** に固定される（`a-1 → a`, `a-2 → a-1`, …, `a+1 → a`, `a+2 → a+1`, …）。
- 暗号文の `length` がそのまま平文の文字数を漏らす。
- 「`relations[i].reference == i±1`」という構造が一目でわかるため、解析者にとって参照グラフが自明。

### 1.2 ロマン視点

> 「あなた（アンカー）から始まり、人づて（文字づて）に想いが繋がっていく」

正準エンコーダはこの隠喩を **直線的にしか** 表現できない。多重結びは赤い糸を蛇行させ、迷い糸は「真実を見つけ出す」というメッセージを暗号文の構造そのものに織り込む。

## 2. 案A: 多重結び (Chain Encoder)

### 2.1 仕様

新しいエンコーダ `encrypt_chain(plaintext, key, anchor_position, rng)` を追加する。

入力:
- `plaintext, key, anchor_position` — `encrypt` と同一。
- `rng` — `RngCore` を実装する乱数源。シード可能。

出力: `Ciphertext`（既存の構造体、`version=1` のまま）。

アルゴリズム:

1. `n = plaintext.chars().count()` を計算し、入力検証は §6 of SPEC と同様。
2. `resolved = {anchor_position}`、`pending = {0..n} \ {anchor_position}` とする。
3. `pending` が空でない間:
   - `pending` から1つランダムに `i` を選ぶ。
   - `resolved` から1つランダムに `r_i` を選ぶ。
   - `relations[i] = chooseRelation(m_i, m_{r_i}, r_i)` を §6 の規則で算出。
   - `i` を `resolved` に移す。
4. `relations[anchor_position] = None`。

これにより、アンカーを根とする **一様ランダム木** が生成される (Wilson's algorithm の単純化版に相当)。

### 2.2 既存デコーダとの互換性

SPEC §5.2 の reference invariant は既に「アンカーから到達可能な任意の有向非巡回グラフ」を許容している。`decrypt` は無改修で本エンコーダの出力を受理する。

### 2.3 決定論性

`rng` 引数を取ることで、CLI から `--seed` を渡したときの再現性を保つ。テストはシード固定の round-trip。

### 2.4 利用例 (CLI)

```bash
musubi encrypt --strategy chain --seed 42 -k key.json
```

`--strategy` は `canonical`（既定）/ `chain` を取る。

## 3. 案B: 迷い糸 (Noise Injection)

### 3.1 仕様

新しいパラメータ `noise: usize` を `encrypt`/`encrypt_chain` の両方に追加する。`noise = 0` のとき動作は既存と同一。

`noise > 0` のとき:

1. 通常通り平文に対する関係グラフ G を構築する（正準でも chain でも可）。`G` の頂点数は `n`。
2. 暗号文の長さを `N = n + noise` に拡張する。`relations` 配列を長さ `N` にリサイズし、新規スロットは一旦 `None`。
3. ダミー位置 `D = {n, n+1, …, N-1}` を平文ノードに散らすため、`D ∪ {0..n}` を一様ランダムにシャッフルし、結果を新しい配列順序とする（インデックスの付け替え）。`anchor_position` も追従。
4. 各 `j ∈ D'`（シャッフル後のダミー位置）に対し:
   - `r_j` を `{0..N-1} \ {j}` の中で **既に解決順序上 `j` より前にある** 位置から選ぶ（DAGを保つため）。
   - ダミー文字 `d_j` をアルファベット Σ から一様ランダムに選び、`relations[j] = chooseRelation(d_j, m'_{r_j}, r_j)` を算出。
5. 最終的に `length = N`、`anchor.position` はシャッフル後の位置、`relations[anchor.position] = None`。

### 3.2 鍵を持つ側から見た復号

§7 of SPEC のデコーダはアンカーから波及的にノードを埋めるが、§7 ステップ7のループは「全ての位置が埋まるまで」回る。これが迷い糸では崩れるため、デコーダ側に **拡張モード** を追加する：

- `decrypt_with_noise(cipher, key)` — アンカーから到達可能なノードのみ復号し、それらを **元の挿入順** ではなく「シャッフル前の平文順」に並べ直して返す。

これを実現するため、暗号文に新しい任意フィールドを追加する：

```json
{
  "version": 1,
  "alphabet": "default-v1",
  "length": 8,
  "anchor": { "position": 4, "character": "し" },
  "relations": [ ... 8 entries ... ],
  "ext": {
    "plaintext_indices": [2, 5, 4, 1, 7]
  }
}
```

- `ext` は **省略可能** な拡張オブジェクト。v0.1 の暗号文には存在しない。
- `ext.plaintext_indices` — 暗号文上の位置から元平文の順序へのマッピング。`plaintext_indices[k] = j` は「平文の `k` 文字目は暗号文上のスロット `j` にある」を意味する。
- このフィールドの追加は **後方互換**。既存パーサは未知のフィールドを無視する（`serde` の既定挙動）。
- `ext.plaintext_indices.len()` がノイズなしのときの平文長 `n`。

### 3.3 鍵を持たない側から見た景色

- アンカー文字は v0.1 同様に1文字露出する（これは仕様上「アンカー」の定義）。
- `length = N` だけが見え、真の平文長 `n` はわからない。
- グラフは1本の弧状の木 (chain encoder の場合) として見えるが、どのノードが平文でどれがダミーかは鍵なしには判別できない。
- 真のアンカーから DAG を辿っても、ダミーノードは平文ノードを参照している場合があるため、「アンカーから到達可能なノード」≠「平文ノード」となるよう設計する必要がある。

> **決定的設計判断**：ダミーノード `d ∈ D` は **平文ノードまたは他のダミーノード** を参照してよいが、**平文ノードがダミーノードを参照することは禁止**する。これにより「平文ノードのみからなる部分グラフ」がアンカーを根として閉じる。受信者は §3.4 のアルゴリズムで平文ノードを識別する。

### 3.4 拡張デコーダ

```
decrypt_with_noise(C, π):
  1. v0.1 の手続きで anchor から到達可能なノードを全て復号する。
     - ただし参照先が未復号で、かつそのノードが C.ext.plaintext_indices に
       含まれていなければ、そのノードはスキップしてよい（ダミーは復号不要）。
  2. C.ext.plaintext_indices = [j_0, j_1, …, j_{n-1}] に従い、
     復号文字を平文順に並べ直す。
  3. plaintext_indices に `anchor.position` が含まれることを検証する。
  4. 結果を返す。
```

実装上は「全部復号してから plaintext_indices で抽出」が最も単純で、分岐も少なく安全。

### 3.5 利用例 (CLI)

```bash
musubi encrypt --strategy chain --noise 3 --seed 42 -k key.json
musubi decrypt -k key.json   # ext.plaintext_indices があれば自動でノイズ除去
```

## 4. 互換性まとめ

| 形式 | 旧デコーダ (v0.1) | 新デコーダ (v0.2) |
|---|---|---|
| 旧暗号文 (canonical, no noise) | ✅ | ✅ |
| 新暗号文 (chain, no noise) | ✅（ext なし、グラフが任意DAGなだけ） | ✅ |
| 新暗号文 (chain, noise > 0) | ❌（reachability が壊れて malformed と誤判定） | ✅ |

`FORMAT_VERSION` は据え置き 1 のままだが、`ext.plaintext_indices` を見て v0.1 デコーダはそれを無視するため、**ノイズ入り暗号文は v0.1 デコーダで復号失敗する**。これは仕様上許容する：「機能拡張のオプトイン」。

将来 v0.3 以降で破壊的変更を入れる場合に `FORMAT_VERSION` を 2 へバンプする。

## 5. CLI 拡張

`musubi encrypt` に追加するフラグ:

| フラグ | 値 | 既定 |
|---|---|---|
| `--strategy <s>` | `canonical` \| `chain` | `canonical` |
| `--noise <N>` | 非負整数 | `0` |
| `--seed <u64>` | 64bit 整数（chain/noise 用 RNG シード） | OS 乱数 |

`musubi decrypt` は `ext.plaintext_indices` の有無で自動的に拡張デコーダへ分岐。新規フラグなし。

## 6. WASM / WebUI 拡張

`musubi-wasm`:

- 既存 `encrypt(plaintext, key_json, anchor)` を維持。
- 新規 `encrypt_woven(plaintext, key_json, anchor, options)` — `options = { strategy: "chain", noise: 3, seed: 42 }` を JS オブジェクトで受ける。

WebUI は当面オプションなしで `encrypt_woven` のデフォルト引数を使い、上級者向けに「織りモード」トグルを足す（v0.2.0 のスコープ内では UI は最小限）。

## 7. テスト計画

- 新規 unit test: chain encoder の round-trip (シード固定 × 多数の anchor 位置)。
- 新規 unit test: noise > 0 での round-trip、`length > n` の確認、`plaintext_indices.len() == n`。
- 新規 unit test: chain + noise の組み合わせ。
- 新規 unit test: ダミーノードが正しく DAG を保ち、巡回しないこと。
- 既存 unit test: 全て無改修で通過することを確認。

## 8. オープンクエスチョン

1. **ノイズの上限**: `noise` の上限を仕様で定めるべきか？ 実装上は `usize` だが、`length = n + noise` がメモリを食うので CLI 側で `--noise` の妥当な上限警告を出すかどうか。
2. **ダミー文字選択**: 平文に含まれる文字をダミーで選ぶことを許すか？ 許すと頻度解析が困難になりロマンが増すが、§3.4 の plaintext_indices なしでは識別不能なので問題なし。**許可** で進める提案。
3. **アンカー位置**: chain エンコーダで anchor が末端になりやすい木を作るか、中央寄りを優先するか？ 一様ランダム木で十分な提案。
4. **`ext` の名前空間**: 将来の拡張に備えて `ext.musubi_v0_2.plaintext_indices` のように名前空間を切るか？ シンプルさ優先で素の `ext.plaintext_indices` を提案。

---

## 9. 実装ロードマップ

```
PR #A  feat(core): chain encoder
       - encrypt_chain() 実装
       - 既存 encrypt() と並置、API 互換
       - SPEC.md §6.5 に節を追加

PR #B  feat(core): noise injection
       - encrypt() / encrypt_chain() に noise 引数
       - decrypt() を拡張デコーダに格上げ
       - Ciphertext::ext フィールド追加 (Option<CiphertextExt>)
       - SPEC.md §6.6 と §7.1 に節を追加

PR #C  feat(cli): --strategy / --noise / --seed フラグ
       - clap の derive 拡張
       - 統合テスト追加

PR #D  feat(wasm): encrypt_woven バインディング
       - WebUI に「織りモード」アコーディオン

PR #E  chore(release): v0.2.0
       - CHANGELOG 更新
       - tag & release notes
```

各 PR はそれぞれ独立してレビュー・マージ可能。
