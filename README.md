# mote

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> A git-agnostic snapshot manager that tracks changes independently of version control

**mote** (微粒子, meaning "fine particles") is a lightweight CLI tool for capturing and comparing project states. Unlike traditional VCS tools, mote operates independently—enabling you to **diff any two points in your project timeline**, regardless of git commits or staging area.

## 🎯 The Core Advantage

**Traditional VCS**: Only compare committed states
**mote**: Compare ANY two snapshots, even across uncommitted changes

This independence means you can:
- Track experimental work without cluttering git history
- Compare states before/after debugging sessions
- Review changes across multiple git operations
- Maintain parallel exploration without branches

## ✨ Key Features

- **🔄 Git/jj Independent**: Coexists peacefully with any VCS—no interference, no conflicts
- **📸 Snapshot-Based Diffing**: Compare ANY two snapshots, regardless of commit/staging state
- **⚡ Lightweight & Fast**: Content-addressable storage with SHA256 + zstd compression
- **🎯 Flexible Comparison**: Diff between snapshots, working directory, or across VCS operations
- **🗂️ Smart Storage**: `.mote/` or `.git/mote/`—your choice
- **🛡️ Safe Restoration**: Auto-backup before restore operations
- **📁 Context Separation**: Multiple storage directories for organizing different workflows

## Installation

### Homebrew (macOS / Linux)

```bash
# Add tap
brew tap shabaraba/tap

# Install mote
brew install mote

# Verify installation
mote --version
```

### Cargo

```bash
cargo install mote
```

### From Source

```bash
# Clone repository
git clone https://github.com/shabaraba/mote.git
cd mote

# Build and install
cargo install --path .
```

### Pre-built Binaries

Download pre-built binaries from [GitHub Releases](https://github.com/shabaraba/mote/releases).

## Quick Start

```bash
# Initialize mote in your project
mote init

# Create a snapshot
mote snapshot -m "Before refactoring"

# View snapshot history
mote log

# Show differences with current working directory
mote diff <snapshot-id>

# Restore a specific file
mote restore <snapshot-id> --file src/main.rs

# Restore entire snapshot
mote restore <snapshot-id> --force
```

## Global Options

All commands support these global options:

### Project & Context Management

Mote uses a 3-layer configuration system (v0.2.0+):

1. **Global**: Base settings for all projects (`~/.config/mote/config.toml`)
2. **Project**: Project-specific settings (`~/.config/mote/projects/<name>/`)
3. **Context**: Context-specific settings (highest priority)

#### `--project <name>` / `-p <name>`

Specify or auto-detect the project. Projects group multiple contexts together:

```bash
# Auto-detect project from current directory
mote snapshot -m "work"

# Explicitly specify project
mote --project my-app snapshot -m "work"
mote -p my-app log
```

#### `--context <name>` / `-c <name>`

Use a specific context within a project. Each context has its own:
- Snapshot history (storage)
- Ignore patterns
- Configuration overrides

```bash
# Use default context
mote snapshot -m "main work"

# Use feature-specific context
mote --context feature-x snapshot -m "feature X iteration"
mote -c feature-x log

# Use experimental context
mote -c experiment snapshot -m "trying new approach"
```

**Typical workflow:**
```bash
# Create a new context for a feature
mote context new feature-auth

# Switch to that context for all operations
mote -c feature-auth snapshot -m "baseline"
mote -c feature-auth log
mote -c feature-auth diff

# Return to default context
mote snapshot -m "back to main work"
```

**Use cases:**
- **Feature development**: Separate history per feature without cluttering main history
- **Experiments**: Disposable snapshots that won't pollute your main timeline
- **Team workflows**: Different contexts for personal vs. shared work
- **Long-term vs. temporary**: Keep important snapshots separate from debugging noise

### `--storage-dir <path>` (Legacy)

Use a custom storage directory instead of managed contexts. This is the legacy approach from v0.1.x:

```bash
# Feature-specific history (legacy style)
mote --storage-dir .mote-feature-x snapshot -m "feature X iteration"
mote --storage-dir .mote-feature-x log
```

**Note**: For new projects, prefer using `--project` and `--context` for better organization.

### Other Global Options

- `--project-root <path>`: Specify project root directory (default: current directory)
- `--ignore-file <path>`: Use custom ignore file (overrides context/project ignore)
- `--config-dir <path>` / `-d <path>`: Use custom config directory (default: `~/.config/mote`)

## Commands

### `mote init`

Initialize mote in the current directory. Creates `.mote/` directory and `.moteignore` file.

```bash
mote init
```

### `mote snapshot`

Create a new snapshot of tracked files.

```bash
mote snapshot                           # Create snapshot
mote snapshot -m "Description"          # With message
mote snapshot --trigger "claude-hook"   # With trigger source
mote snapshot --auto                    # Auto mode (silent, skip if no changes)
```

### `mote setup-shell`

Print shell integration script for git/jj auto-snapshot.

```bash
mote setup-shell zsh    # For zsh/bash
mote setup-shell fish   # For fish shell

# Add to your shell config:
mote setup-shell zsh >> ~/.zshrc
```

### `mote log`

Show snapshot history.

```bash
mote log                # Show recent snapshots
mote log --limit 50     # Show more snapshots
mote log --oneline      # Compact format
```

### `mote show`

Show details of a specific snapshot.

```bash
mote show abc123d       # Use short ID
```

### `mote diff`

Show differences between snapshots or working directory.

```bash
mote diff abc123d              # Compare with working directory
mote diff abc123d def456a      # Compare two snapshots
mote diff abc123d --content    # Show file content diff
```

### `mote restore`

Restore files from a snapshot.

```bash
mote restore abc123d --file src/main.rs   # Restore single file
mote restore abc123d                       # Restore all (creates backup first)
mote restore abc123d --force               # Force restore without backup
mote restore abc123d --dry-run             # Preview what would be restored
```

### `mote context`

Manage contexts within a project. Each context maintains separate snapshot history and configuration.

```bash
# List all contexts for current/specified project
mote context list
mote -p my-app context list

# Create a new context
mote context new feature-auth
mote -p my-app context new feature-auth

# Create context with custom directory
mote context new feature-auth --context-dir ~/mote-contexts/feature-auth

# Create context with custom working directory
mote context new feature-auth --cwd /path/to/project

# Delete a context (cannot delete 'default')
mote context delete feature-auth
mote -p my-app context delete feature-auth
```

**Context naming rules:**
- Must start with ASCII letter or underscore
- Can contain ASCII letters, digits, hyphens, underscores, and dots
- 1-255 characters
- Cannot use Windows reserved names (CON, PRN, AUX, etc.)
- Cannot contain path separators or control characters

## Configuration

Mote uses a 3-layer configuration hierarchy:

```
~/.config/mote/
├── config.toml                    # Global config (base defaults)
└── projects/
    └── <project-name>/
        ├── config.toml            # Project config (overrides global)
        └── contexts/
            └── <context-name>/
                ├── config.toml    # Context config (highest priority)
                ├── ignore         # Context-specific ignore patterns
                └── storage/       # Context-specific snapshots
                    ├── objects/
                    └── snapshots/
```

### Global Configuration

File: `~/.config/mote/config.toml`

```toml
[storage]
# Storage location strategy (for legacy --storage-dir usage)
# "root": Always use .mote/ in project root (default)
# "vcs": Always use .git/mote/ or .jj/mote/
# "auto": Use VCS directory if available, otherwise root
location_strategy = "root"
compression_level = 3

[snapshot]
auto_cleanup = true
max_snapshots = 1000
max_age_days = 30

[ignore]
ignore_file = ".moteignore"
```

### Project Configuration

File: `~/.config/mote/projects/<name>/config.toml`

```toml
# Project working directory
path = "/path/to/project"

# Optional: Map of contexts with custom directories
# [contexts]
# feature-x = "/custom/path/to/feature-x-context"

# Project-specific overrides (inherits from global)
[storage]
compression_level = 5

[snapshot]
max_snapshots = 500
```

### Context Configuration

File: `~/.config/mote/projects/<name>/contexts/<context>/config.toml`

```toml
# Optional: Context working directory (if different from project)
# cwd = "/path/to/subdirectory"

# Optional: Custom context directory location
# context_dir = "/custom/path"

# Context-specific overrides (highest priority)
[snapshot]
max_snapshots = 100
max_age_days = 7
```

**Configuration priority**: Context > Project > Global

## .moteignore

Uses gitignore syntax to specify files to ignore:

```
# Dependencies
node_modules/
vendor/

# Build outputs
target/
dist/

# IDE files
.idea/
.vscode/
```

## 📖 Common Use Cases

### Use Case 1: Feature Development with Isolated Context

**Scenario**: 複数の機能を並行開発しており、それぞれの作業履歴を分離したい

```bash
# 1. プロジェクトの初期化（初回のみ）
cd /path/to/my-app
mote init

# 2. 認証機能用のコンテキスト作成
mote context new feature-auth
# ✓ Created context 'feature-auth' for project 'my-app'

# 3. 認証機能の開発開始（ベースライン作成）
mote -c feature-auth snapshot -m "Start authentication feature"
# Snapshot created: abc123d

# 4. 実装作業
# ... コードを編集 ...

# 5. 途中経過を記録
mote -c feature-auth snapshot -m "Add login form"
# Snapshot created: def456a

# 6. さらに実装
# ... コードを編集 ...

# 7. 途中の差分確認
mote -c feature-auth diff def456a
# Shows: 現在の作業ディレクトリとスナップショットdef456aの差分

# 8. 実装完了時のスナップショット
mote -c feature-auth snapshot -m "Complete authentication feature"
# Snapshot created: ghi789b

# 9. feature-authコンテキストの全履歴確認
mote -c feature-auth log
# Shows: feature-auth専用のスナップショット履歴

# 10. 別の機能に切り替え（デフォルトコンテキストに戻る）
mote snapshot -m "Back to main development"
# Snapshot created: jkl012c (別のコンテキスト)

# 11. いつでもfeature-authの履歴に戻れる
mote -c feature-auth log
```

**メリット**:
- 機能ごとに独立したスナップショット履歴
- メイン開発ラインを汚さない
- 複数機能の並行開発が容易

### Use Case 2: Experimental Work (Disposable Context)

**Scenario**: 新しいアプローチを試したいが、失敗したら簡単に削除したい

```bash
# 1. 実験用コンテキスト作成
mote context new experiment-refactor
# ✓ Created context 'experiment-refactor'

# 2. 実験開始前のベースライン
mote -c experiment-refactor snapshot -m "Before refactoring experiment"
# Snapshot created: exp001a

# 3. 大胆なリファクタリング実施
# ... 大幅なコード変更 ...

# 4. 途中経過を記録
mote -c experiment-refactor snapshot -m "Try new architecture pattern"
# Snapshot created: exp002b

# 5. 結果が良くない場合は元に戻す
mote -c experiment-refactor restore exp001a
# Restored from snapshot exp001a

# 6. 実験が失敗したらコンテキストごと削除
mote context delete experiment-refactor
# ✓ Deleted context 'experiment-refactor'
# → 実験の痕跡が完全に消える（メインの履歴は無傷）

# 7. 実験が成功した場合は、そのまま継続開発
mote -c experiment-refactor snapshot -m "New architecture works!"
# → このコンテキストを本番に昇格させることも可能
```

**メリット**:
- 失敗しても簡単にクリーンアップ
- メインの履歴を汚さない
- 複数の実験を同時並行可能

### Use Case 3: Debugging Session Tracking

**Scenario**: バグ調査中の変更を追跡し、必要に応じて元に戻したい

```bash
# 1. デバッグ用コンテキスト作成
mote context new debug-issue-42
# ✓ Created context 'debug-issue-42'

# 2. バグ発生時の状態を記録
mote -c debug-issue-42 snapshot -m "Initial bug state"
# Snapshot created: bug001a

# 3. デバッグ用のログ追加
# ... console.log, デバッガ設定など ...
mote -c debug-issue-42 snapshot -m "Add debug logging"
# Snapshot created: bug002b

# 4. 仮説1を試す
# ... コード変更 ...
mote -c debug-issue-42 snapshot -m "Hypothesis 1: async timing issue"
# Snapshot created: bug003c

# 5. 仮説1が外れたので仮説2を試す
mote -c debug-issue-42 restore bug002b  # ログ追加直後に戻る
mote -c debug-issue-42 snapshot -m "Hypothesis 2: race condition"
# Snapshot created: bug004d

# 6. 原因特定！修正を適用
# ... 修正コード ...
mote -c debug-issue-42 snapshot -m "Fix identified: mutex needed"
# Snapshot created: bug005e

# 7. バグ修正前後の差分を確認
mote -c debug-issue-42 diff bug001a bug005e --content
# Shows: 最初の状態と修正後の完全な差分

# 8. デバッグログをクリーンアップ（最初の状態に戻す）
mote -c debug-issue-42 restore bug001a --file src/problematic-module.js
# → デバッグログだけ削除、修正は保持

# 9. デバッグ完了後、コンテキストを削除または保存
mote context delete debug-issue-42  # 削除
# または
mote -c debug-issue-42 snapshot -m "Final clean state"  # 記録として保存
```

**メリット**:
- デバッグの試行錯誤を完全に追跡
- いつでも過去の状態に戻れる
- 原因特定後のクリーンアップが容易

### Use Case 4: Team Workflow - Personal vs Shared

**Scenario**: チーム開発で個人作業と共有作業を分離したい

```bash
# 1. 個人作業用コンテキスト（デフォルト）
mote snapshot -m "Personal exploration"
# Snapshot created: per001a (default context)

# 2. チーム共有用コンテキスト作成
mote context new team-shared --cwd /path/to/team/workspace
# ✓ Created context 'team-shared'

# 3. チーム作業時のみ共有コンテキストを使用
mote -c team-shared snapshot -m "Team sprint 1 start"
# Snapshot created: team001a

# 4. ペアプログラミング中の変更を記録
mote -c team-shared snapshot -m "Pair programming session"
# Snapshot created: team002b

# 5. 個人作業に戻る（コンテキストを切り替えるだけ）
mote snapshot -m "Personal refactoring ideas"
# Snapshot created: per002b (別の履歴)

# 6. チーム作業の履歴確認
mote -c team-shared log
# Shows: チーム作業のみの履歴

# 7. 個人作業の履歴確認
mote log
# Shows: 個人作業のみの履歴
```

**メリット**:
- 個人とチームの作業履歴を明確に分離
- コンテキスト切り替えだけで作業モード変更
- それぞれの履歴が混ざらない

### Use Case 5: Long-term vs Temporary Snapshots

**Scenario**: 重要なマイルストーンと日々のデバッグを分けて管理したい

```bash
# 1. 重要マイルストーン用コンテキスト
mote context new milestones
# ✓ Created context 'milestones'

# 2. 一時的なデバッグ用コンテキスト（短期保存設定）
mote context new temp-debug
# ✓ Created context 'temp-debug'

# 3. マイルストーンを記録
mote -c milestones snapshot -m "v1.0.0 release candidate"
# Snapshot created: mile001a

# 4. 日々のデバッグはtemp-debugで
mote -c temp-debug snapshot -m "Debug session 2024-01-28"
# Snapshot created: temp001a

# 5. 定期的にtemp-debugをクリーンアップ
mote context delete temp-debug
mote context new temp-debug  # 新規作成で履歴リセット

# 6. マイルストーンは長期保存
mote -c milestones log
# Shows: 重要な節目のみの履歴（見やすい）
```

**メリット**:
- 重要なスナップショットとノイズを分離
- 一時的なコンテキストは気軽に削除可能
- 長期的な履歴が見やすい

## 💡 Why mote?

### The Fundamental Difference

| Aspect | Traditional VCS | mote |
|--------|----------------|------|
| **Comparison Scope** | Only committed states | Any two snapshots |
| **Staging Required** | Yes (git add) | No |
| **Commit Required** | Yes | No |
| **Branch Overhead** | Heavy | Lightweight |
| **Parallel Exploration** | Branch management | Just take snapshots |

### Perfect Use Cases

**🧪 Experimental Development**
```bash
mote snapshot -m "baseline"
# Try approach A
mote snapshot -m "approach-a"
# Try approach B
mote snapshot -m "approach-b"
mote diff approach-a approach-b  # Compare without any commits
```

**🐛 Debugging Sessions**
```bash
mote snapshot -m "before-debug"
# Add logging, modify code, test...
mote snapshot -m "after-debug"
mote diff before-debug after-debug  # See exactly what changed
```

**📊 Cross-VCS Analysis**
```bash
git checkout feature-1    # → auto snapshot
# work on feature-1
git checkout feature-2    # → auto snapshot
# work on feature-2
mote diff <feature-1-snapshot> <feature-2-snapshot>  # Compare work across branches
```

## 🔗 Integration

### Git/jj Integration (Recommended)

mote shines when integrated with your VCS workflow. Automatically capture snapshots on VCS operations:

**Setup:**
```bash
mote setup-shell zsh >> ~/.zshrc
source ~/.zshrc
```

**Supported commands:**
- **git**: checkout, switch, merge, rebase, pull, stash, reset
- **jj**: edit, new, abandon, rebase, squash, restore, undo

**Workflow:**
```bash
git checkout feature-branch    # → auto snapshot (state A)
# ... make changes ...
git checkout main              # → auto snapshot (state B)
mote diff <A> <B>              # → diff across git operations
```

### Claude Code Hook Integration

Add to your `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": "mote snapshot --trigger claude-hook"
  }
}
```

### vibing.nvim Integration

```lua
require('vibing').setup({
  on_ai_edit = function()
    vim.fn.system('mote snapshot --trigger vibing.nvim')
  end
})
```

## Architecture

```
.mote/
├── objects/           # Content-addressable storage (SHA256 hash → zstd compressed)
│   ├── ab/
│   │   └── cdef1234...
│   └── ...
└── snapshots/         # Snapshot metadata (JSON)
    └── 20260119_002700_abc123.json
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Documentation

- [Testing Guide](docs/testing/TESTING.md)
- [Development Setup](docs/development/HOMEBREW_SETUP.md)
- [Release Process](docs/development/RELEASE.md)
