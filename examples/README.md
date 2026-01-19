# mote Examples

This directory contains example files demonstrating mote usage patterns.

## Files

- `test.diff`: Example diff output showing mote's snapshot comparison format

## Usage Examples

### Basic Workflow

```bash
# Initialize mote in your project
mote init

# Create your first snapshot
mote snapshot -m "Initial state"

# Make some changes to your files
echo "new feature" >> src/feature.rs

# Create another snapshot
mote snapshot -m "Added new feature"

# View the difference
mote diff <first-snapshot-id> <second-snapshot-id>
```

### Experimental Development

```bash
# Baseline snapshot
mote snapshot -m "baseline-implementation"

# Try different approaches without git commits
# Approach 1
mote snapshot -m "approach-1-using-hashmap"

# Approach 2
mote snapshot -m "approach-2-using-btree"

# Compare the approaches
mote diff approach-1 approach-2 --content
```

### Integration with Git

```bash
# Set up shell integration
mote setup-shell zsh >> ~/.zshrc
source ~/.zshrc

# Now git operations automatically create snapshots
git checkout feature-branch  # → automatic snapshot
# ... work on feature ...
git checkout main           # → automatic snapshot

# Compare what you did on the feature branch
mote diff <feature-snapshot> <main-snapshot>
```

### Debugging Session

```bash
# Before starting debug session
mote snapshot -m "before-debug-session"

# Add debug logging, modify code
vim src/main.rs

# After debugging
mote snapshot -m "after-debug-session"

# Review exactly what changed during debugging
mote diff before-debug after-debug --content

# If debug changes are good, keep them
# If not, restore the original
mote restore before-debug
```

## Advanced Patterns

### Cross-Branch Comparison

Since mote is git-agnostic, you can compare states across different branches without merging:

```bash
# On branch feature-a
mote snapshot -m "feature-a-final"

# Switch branches
git checkout feature-b
mote snapshot -m "feature-b-final"

# Compare implementations across branches
mote diff feature-a-final feature-b-final
```

### Time-Based Analysis

```bash
# Morning snapshot
mote snapshot -m "morning-state"

# Evening snapshot
mote snapshot -m "evening-state"

# See what you accomplished today
mote diff morning-state evening-state --content
```

## Tips

- Use descriptive snapshot messages to make them easier to find later
- Leverage `mote log` to browse your snapshot history
- Use `--content` flag with diff for detailed line-by-line comparison
- Remember: snapshots are independent of git commits—feel free to snapshot frequently!
