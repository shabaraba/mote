# mote

A fine-grained snapshot management tool for projects - like dust accumulates to become a mountain.

**mote**（微粒子）は、git commitより細かい粒度でプロジェクトのスナップショットを管理するCLIツールです。「塵も積もれば山となる」のコンセプトに基づき、小さな変更を積み重ねて履歴を形成します。

## Features

- **Fine-grained snapshots**: Create snapshots more frequently than git commits
- **Content-addressable storage**: Efficient deduplication using SHA256 + zstd compression
- **Independent of git**: Works alongside git without conflicts
- **Flexible storage location**: Store in `.mote/` or inside `.git/mote/` based on your preference
- **Auto-backup on restore**: Automatically creates a backup before restoring

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

## Commands

### `mote init`
Initialize mote in the current directory. Creates `.mote/` directory and `.moteignore` file.

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

## Configuration

Global configuration file: `~/.config/mote/config.toml`

```toml
[storage]
# Storage location strategy
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

## Use Cases

### Git/jj Integration (Recommended)

mote is designed to work alongside git or jj without conflicts. Using shell integration, you can automatically create snapshots when switching branches or performing other VCS operations.

**Setup:**
```bash
# Add to your shell config
mote setup-shell zsh >> ~/.zshrc
source ~/.zshrc
```

**How it works:**
- When you run `git checkout`, `git merge`, `jj edit`, etc., mote automatically takes a snapshot
- The snapshot captures the state right after the VCS operation
- You can then work freely, and take another snapshot before the next VCS operation
- This gives you a diff of "what changed between VCS operations"

**Supported commands:**
- **git**: checkout, switch, merge, rebase, pull, stash, reset
- **jj**: edit, new, abandon, rebase, squash, restore, undo

**Example workflow:**
```bash
git checkout feature-branch    # → auto snapshot (state A)
# ... make changes ...
git checkout main              # → auto snapshot (state B)
mote diff <A> <B>              # → see what you changed on feature-branch
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

## License

MIT
