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

### `--storage-dir <path>`

Use a custom storage directory instead of the default `.mote/`. This enables context separation—you can maintain independent snapshot histories for different workflows:

```bash
# Default: project-wide history
mote snapshot -m "general work"

# Feature-specific history
mote --storage-dir .mote-feature-x snapshot -m "feature X iteration"
mote --storage-dir .mote-feature-x log

# Experimental history (can be deleted anytime)
mote --storage-dir .mote-experiment snapshot -m "trying new approach"
```

**Use cases:**
- **Feature development**: Separate history per feature without cluttering main history
- **Experiments**: Disposable snapshots that won't pollute your main timeline
- **Team workflows**: Different storage for personal vs. shared snapshots
- **Long-term vs. temporary**: Keep important snapshots separate from debugging noise

### Other Global Options

- `--project-root <path>`: Specify project root directory (default: current directory)
- `--ignore-file <path>`: Use custom ignore file (default: `.moteignore`)
- `--config <path>`: Use custom config file

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
