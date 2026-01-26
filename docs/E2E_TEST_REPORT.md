# Mote E2E テストレポート

**実行日**: 2026-01-26
**バージョン**: v0.1.2 (feature/project-context-management)
**テスト環境**: macOS, `/tmp/mote-e2e-test`

## テスト結果サマリー

| ユースケース | 結果 | 備考 |
|---|---|---|
| UC1: 新規プロジェクトでの初回セットアップ | ✅ PASS | プロジェクト自動作成機能が正常動作 |
| UC2: 複数コンテキストの管理 | ✅ PASS | コンテキスト間の独立性を確認 |
| UC3: カスタムストレージディレクトリ | ✅ PASS | --storage-dirオプションが正常動作 |
| UC4: プロジェクト自動検出 | ✅ PASS | cwdベースの自動検出が動作 |
| UC5: 既存プロジェクトからの移行 | ✅ PASS | .moteディレクトリからの移行成功 |
| UC6: コンテキストの削除と一覧表示 | ✅ PASS | default保護機能を含む |

**全体結果**: ✅ **6/6 テストケース成功**

---

## UC1: 新規プロジェクトでの初回セットアップ

### シナリオ
初めてmoteを使うユーザーが、新しいプロジェクトでコンテキストを作成する

### 実行コマンド

```bash
cd /tmp/mote-e2e-test/project-alpha
mote --project project-alpha context new default --cwd "$PWD"
```

### 期待される動作
- プロジェクトが存在しない場合、自動的に作成される
- デフォルトコンテキストが作成される
- ストレージディレクトリ構造が正しく作成される

### 実行結果

```
✓ Created project 'project-alpha'
✓ Created context 'default' for project 'project-alpha'
```

**ディレクトリ構造:**
```
~/.config/mote/projects/project-alpha/
├── config.toml
└── contexts/
    └── default/
        ├── config.toml
        ├── ignore
        └── storage/
            ├── objects/
            └── snapshots/
```

**スナップショット作成:**
```bash
echo "console.log('hello');" > app.js
echo "# Project Alpha" > README.md
mote --project project-alpha snapshot -m "Initial commit"
```

```
✓ Created snapshot 84eaccd (2 files)
  Message: Initial commit
```

### 検証項目
- ✅ プロジェクト自動作成
- ✅ コンテキスト作成
- ✅ ストレージディレクトリ構造
- ✅ ignoreファイル作成
- ✅ スナップショット作成

---

## UC2: 複数コンテキストの管理

### シナリオ
1つのプロジェクトで開発環境・ステージング・本番環境など複数コンテキストを使い分ける

### 実行コマンド

```bash
# staging コンテキスト作成
mote --project project-alpha context new staging --cwd "$PWD"

# production コンテキスト作成
mote --project project-alpha context new production --cwd "$PWD"

# コンテキスト一覧
mote --project project-alpha context list
```

### 実行結果

```
Contexts for project 'project-alpha':
  staging
  default (default)
  production
```

**各コンテキストでの独立したスナップショット:**

```bash
# defaultコンテキストで変更
echo "console.log('development');" >> app.js
mote --project project-alpha snapshot -m "Dev changes"

# stagingコンテキストで別のスナップショット
mote --project project-alpha -c staging snapshot -m "Staging snapshot"
```

**defaultコンテキストのログ:**
```
snapshot 879b02d - Dev changes (2 files)
snapshot 84eaccd - Initial commit (2 files)
```

**stagingコンテキストのログ:**
```
snapshot d4863df - Staging snapshot (2 files)
```

### 検証項目
- ✅ 複数コンテキスト作成
- ✅ コンテキスト一覧表示
- ✅ コンテキスト間の独立性（各コンテキストで別のスナップショット履歴）
- ✅ defaultコンテキストの表示（"(default)"マーク）

---

## UC3: カスタムストレージディレクトリ

### シナリオ
デフォルトと異なる場所にストレージを配置したい

### 実行コマンド

```bash
cd /tmp/mote-e2e-test/project-beta
mote --project project-beta context new default --cwd "$PWD" --storage-dir custom_data
```

### 実行結果

```
✓ Created project 'project-beta'
✓ Created context 'default' for project 'project-beta'
```

**ストレージディレクトリ:**
```
~/.config/mote/projects/project-beta/contexts/default/custom_data/
├── objects/
└── snapshots/
```

**スナップショット作成:**
```bash
echo "data = [1, 2, 3]" > data.py
mote --project project-beta snapshot -m "Initial data"
```

```
✓ Created snapshot 2c5edf8 (1 files)
  Message: Initial data
```

### 検証項目
- ✅ カスタムストレージディレクトリの指定
- ✅ カスタムディレクトリ内でのスナップショット作成
- ✅ オブジェクトとスナップショットディレクトリの作成

---

## UC4: プロジェクト自動検出

### シナリオ
プロジェクトディレクトリ内で作業中、`--project`オプションなしでコマンドを実行する

### 実行コマンド

```bash
cd /tmp/mote-e2e-test/project-alpha

# --projectオプションなしで実行
mote log --limit 3
```

### 実行結果

```
snapshot 879b02d
Date:    2026-01-26 02:11:42 UTC
Message: Dev changes
Files:   2

snapshot 84eaccd
Date:    2026-01-26 02:11:42 UTC
Message: Initial commit
Files:   2
```

**サブディレクトリからの実行:**
```bash
mkdir -p src/components
cd src/components
echo "export const Header = () => {};" > Header.jsx
mote snapshot -m "Add Header component"
```

```
✓ Created snapshot 64a8ca6 (1 files)
  Message: Add Header component
```

### 検証項目
- ✅ プロジェクトルートからの自動検出
- ✅ サブディレクトリからの自動検出
- ✅ cwdベースのプロジェクトマッチング

---

## UC5: 既存プロジェクトからの移行

### シナリオ
`.mote`ディレクトリを使っていた既存プロジェクトを新構造に移行する

### 実行コマンド

```bash
cd /tmp/mote-e2e-test/project-legacy

# 従来の init で .mote ディレクトリを作成
/path/to/mote init

# スナップショット作成
/path/to/mote snapshot -m "Legacy snapshot"

# 新構造へ移行
mote migrate
```

### 実行結果

**init:**
```
✓ Initialized mote in /tmp/mote-e2e-test/project-legacy/.mote
  Created .moteignore for ignore patterns
```

**legacy snapshot:**
```
✓ Created snapshot 43641ba (2 files)
  Message: Legacy snapshot
```

**migration:**
```
Migrating .mote/ to new structure...
  Project name: project-legacy
  Source: /tmp/mote-e2e-test/project-legacy/.mote
  Destination: ~/.config/mote/projects/project-legacy/contexts/default/storage
  Copied .moteignore to context

✓ Migration complete!
  You can now remove the old .mote/ directory
  Use: -p project-legacy -c default for future commands
```

**移行後の確認:**
```bash
mote --project project-legacy log --limit 5
```

```
snapshot 43641ba
Date:    2026-01-26 02:12:00 UTC
Message: Legacy snapshot
Files:   2
```

### 検証項目
- ✅ 既存.moteディレクトリの検出
- ✅ スナップショットデータの移行
- ✅ .moteignoreファイルの移行
- ✅ プロジェクト設定の自動作成
- ✅ 移行後のスナップショット履歴の保持

---

## UC6: コンテキストの削除と一覧表示

### シナリオ
不要になったコンテキストを削除し、現在のコンテキスト一覧を確認する

### 実行コマンド

```bash
# 削除前の一覧
mote --project project-alpha context list

# stagingコンテキストを削除
mote --project project-alpha context delete staging

# 削除後の一覧
mote --project project-alpha context list

# defaultコンテキストの削除試行（失敗するはず）
mote --project project-alpha context delete default
```

### 実行結果

**削除前:**
```
Contexts for project 'project-alpha':
  staging
  default (default)
  production
```

**削除:**
```
✓ Deleted context 'staging' from project 'project-alpha'
```

**削除後:**
```
Contexts for project 'project-alpha':
  default (default)
  production
```

**default削除試行:**
```
error: Failed to read config: Cannot delete default context
✓ Correctly prevented default context deletion
```

### 検証項目
- ✅ コンテキスト一覧表示
- ✅ コンテキスト削除
- ✅ defaultコンテキストの削除保護
- ✅ 削除後のディレクトリ構造更新

---

## 発見された改善点

### 1. プロジェクト自動作成機能の追加 ✅

**問題**: `context new`実行時にプロジェクトが存在しないとエラーになる

**解決策**:
- `ResolveOptions`に`allow_missing_project`フラグを追加
- `context new`コマンド実行時のみ`true`に設定
- プロジェクトが存在しない場合、自動的に作成

**変更ファイル**:
- `src/config/resolver.rs`: allow_missing_projectフラグ追加
- `src/main.rs`: context newの場合のみフラグをtrue に設定
- `src/main_new_commands.rs`: プロジェクト自動作成ロジック追加

### 2. init-projectコマンドの削除 ✅

**理由**: `context new`だけで全て完結するため冗長

**影響**:
- コマンド数削減
- ユーザー体験のシンプル化
- ドキュメント更新

---

## テスト環境詳細

**環境変数:**
```bash
MOTE_BIN=/Users/shaba/workspace/mote/target/release/mote
MOTE_CONFIG_DIR=/tmp/mote-e2e-test/.config/mote
```

**テストプロジェクト:**
- `project-alpha`: 複数コンテキスト管理のテスト
- `project-beta`: カスタムストレージディレクトリのテスト
- `project-legacy`: 移行機能のテスト

**成果物:**
- テストスクリプト: `/tmp/mote-e2e-test/run-e2e-tests.sh`
- テスト計画: `/tmp/mote-e2e-test/test-plan.md`
- 設定ディレクトリ: `/tmp/mote-e2e-test/.config/mote/`

---

## 結論

すべてのユースケースが正常に動作することを確認しました。

### 主要機能の検証完了:
1. ✅ プロジェクト自動作成
2. ✅ コンテキスト管理（作成・削除・一覧）
3. ✅ カスタムストレージディレクトリ
4. ✅ プロジェクト自動検出
5. ✅ 既存プロジェクトからの移行
6. ✅ デフォルトコンテキスト保護

### 新しいワークフロー:

```bash
# 1. 新規プロジェクトの開始（プロジェクトとコンテキストを同時に作成）
mote --project my-app context new default --cwd "$PWD"

# 2. 追加コンテキストの作成
mote --project my-app context new staging
mote --project my-app context new production

# 3. スナップショット作成（プロジェクトは自動検出）
mote snapshot -m "Initial commit"

# 4. コンテキスト切り替え
mote -c staging snapshot -m "Staging changes"

# 5. 既存プロジェクトの移行
mote migrate
```

すべてのテストケースが成功し、新しいコマンド構造が正常に動作することを確認しました。
