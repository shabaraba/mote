# Claude Code引き継ぎ用コンテキスト

## プロジェクト概要

**git commitより細粒度のスナップショット管理ツール**をCLIとして作成する。

### 背景・動機

- jujutsuのような細粒度管理をgitと併用したい
- vibing.nvim（AI vibe coding用Neovimプラグイン）から利用する
- Claude Codeのhookに登録して、ファイル変更時に自動スナップショットを取りたい
- 既存ツール（dura, undotree, VSCode Local History）はいずれも「プロジェクト全体のスナップショット」を満たさない

## 決定事項

| 項目 | 決定内容 |
|------|----------|
| トリガー | Claude Codeのhook経由 + CLIコマンドとしても提供 |
| git連携 | **独立管理**。gitとは別レイヤーで動作、競合しない |
| 配布形態 | 独立CLI。vibing.nvimからは外部コマンドとして呼ぶ |
| プロジェクト名 | small-git (ディレクトリ名より) |

## 未決定事項（要相談・提案付き）

### 1. MVPスコープ

**推奨: b) snapshot取得 + log表示 + diff表示**

理由：
- 復元機能なしでも、履歴確認とdiffで十分価値がある
- 実装複雑度が中程度で、段階的拡張が容易
- Claude Codeのhookからの利用に最適

選択肢：
- a) snapshot取得 + log表示のみ（最小）
- **b) a + diff表示（推奨）**
- c) b + 特定スナップショットへの復元
- d) c + プロジェクト全体の一括復元

### 2. ストレージ形式

**推奨: a) 独自形式（content-addressable storage）**

理由：
- gitと完全独立、競合リスクなし
- ファイル重複排除で容量効率が良い
- 実装がシンプル（SHA256 + 圧縮）

選択肢：
- **a) 独自形式（content-addressable storage）（推奨）**
  ```
  .snapshots/
  ├── objects/
  │   ├── ab/cdef1234...  # SHA256ハッシュでファイル内容を保存
  │   └── ...
  └── snapshots/
      └── 20260119_002700_abc123.json  # メタデータ
  ```
- b) gitのobject format流用（libgit2依存、オーバーキル）
- c) 単純ファイルコピー（容量効率悪い）

### 3. 追跡対象

**推奨: a) .gitignoreを尊重**

理由：
- ユーザー期待値に合致
- node_modules等の無駄なスナップショット回避
- 追加設定不要

選択肢：
- **a) .gitignoreを尊重（推奨）**
- b) 独自.ignoreファイル（追加設定が面倒）
- c) 全ファイル（容量爆発）

### 4. 実装言語

**推奨: Rust**

理由：
- シングルバイナリ配布が容易
- ファイルI/O性能が高い
- クロスプラットフォーム対応が容易
- durやjujutsuもRust製で参考実装が豊富

代替案：
- Go（シンプル、ビルド速度速い）
- TypeScript（Denoで配布、Node.js依存）

### 5. ツール名

**候補（優先順）:**
1. **grain** - 粒（小さな単位）、覚えやすい
2. **kizami** - 日本語「刻み」、細かい単位
3. **tick** - 時間の最小単位
4. **speck** - 微粒子

**推奨: grain**
- 短い、覚えやすい、タイプしやすい
- 「細粒度」のコンセプトに合致
- コマンド衝突リスク低い

## 想定アーキテクチャ（MVP版）

```
プロジェクトルート/
└── .grain/                    # ストレージディレクトリ
    ├── config.toml            # 設定ファイル
    ├── objects/               # content-addressable storage
    │   ├── ab/
    │   │   └── cdef1234...    # SHA256ハッシュのファイル内容（zstd圧縮）
    │   └── ...
    └── snapshots/             # スナップショットメタデータ
        ├── 20260119_002700_abc123.json
        └── ...
```

### スナップショットメタデータ形式

```json
{
  "id": "abc123...",
  "timestamp": "2026-01-19T00:27:00Z",
  "message": "optional snapshot message",
  "files": [
    {
      "path": "src/main.rs",
      "hash": "abcdef1234...",
      "size": 1234,
      "mode": "0644"
    }
  ],
  "trigger": "claude-code-hook",
  "git_commit": "abc123..." // optional, 現在のgit commitハッシュ
}
```

## 想定コマンド（MVP版）

```bash
# 初期化
grain init

# スナップショット作成（hookから呼ばれる）
grain snapshot [--message "optional"]

# 履歴表示
grain log [--limit 20] [--oneline]

# 詳細表示
grain show <snapshot-id>

# diff表示
grain diff <snapshot-id-1> [snapshot-id-2]
# snapshot-id-2省略時は現在のワーキングディレクトリとdiff

# （将来）復元
# grain restore <snapshot-id> [--file <path>]
```

## Claude Codeフック連携

### hook設定例（~/.claude/settings.json）

```json
{
  "hooks": {
    "after_tool": {
      "Edit": "grain snapshot --message 'Auto: after Edit'",
      "Write": "grain snapshot --message 'Auto: after Write'"
    }
  }
}
```

### vibing.nvim連携

```lua
-- vibing.nvim設定例
require('vibing').setup({
  on_ai_edit = function()
    vim.fn.system('grain snapshot --message "Auto: vibing.nvim edit"')
  end
})
```

## 参考にすべき既存ツール

### dura (tjdevries/dura)
- Rust製バックグラウンドgit commit自動化ツール
- **参考ポイント:**
  - Watcherの実装（notify crateを使用）
  - プロジェクト検出ロジック
  - 非同期スナップショット処理

### jujutsu (martinvonz/jj)
- 次世代VCS、operation log機能
- **参考ポイント:**
  - Content-addressable storageの設計
  - Operation logのメタデータ構造
  - ユーザーフレンドリーなCLIデザイン

### Git Local History系
- VSCode Local History拡張
- vim-undotree
- **参考ポイント:**
  - ユーザーが期待する機能セット
  - UIでの履歴表示方法

## 技術スタック提案（Rust版）

```toml
[dependencies]
clap = "4.4"              # CLI引数パース
serde = "1.0"             # JSONシリアライズ
serde_json = "1.0"
chrono = "0.4"            # タイムスタンプ
sha2 = "0.10"             # SHA256ハッシュ
zstd = "0.13"             # 圧縮
walkdir = "2.4"           # ディレクトリトラバース
ignore = "0.4"            # .gitignore処理
anyhow = "1.0"            # エラーハンドリング
colored = "2.1"           # 色付き出力
```

## 実装優先順位（MVP）

### フェーズ1: 基本機能
1. `grain init` - ストレージディレクトリ初期化
2. `grain snapshot` - ファイルスナップショット作成
3. `grain log` - スナップショット履歴表示

### フェーズ2: 比較機能
4. `grain show` - スナップショット詳細表示
5. `grain diff` - スナップショット間のdiff表示

### フェーズ3: 統合
6. Claude Codeフック連携テスト
7. vibing.nvim連携テスト
8. ドキュメント整備

## 成功基準

- [ ] 100ms以内でスナップショット作成完了（小規模プロジェクト）
- [ ] .gitignore尊重が正しく動作
- [ ] 既存git操作と競合しない
- [ ] Claude Codeフックから安定動作
- [ ] vibing.nvimから外部コマンドとして呼び出し可能

## 次のステップ

1. **未決定事項の確認**
   - MVPスコープ確定（推奨: snapshot + log + diff）
   - ツール名確定（推奨: grain）
   - 実装言語確定（推奨: Rust）

2. **プロジェクト初期化**
   ```bash
   cargo init --name grain
   ```

3. **基本構造実装**
   - CLI引数パース（clap）
   - 設定ファイル読み込み
   - ストレージ初期化

4. **コア機能実装**
   - ファイルハッシュ計算
   - content-addressable storage
   - スナップショットメタデータ生成

引き継ぎを受ける方へ：上記の推奨事項で進めて良いか確認してから実装を開始してください。
