# mote shell integration for git/jj auto-snapshot
# Add this to your ~/.bashrc or ~/.zshrc

# Git wrapper - auto-snapshot on branch/state changes
git() {
    command git "$@"
    local status=$?
    case "$1" in
        checkout|switch|merge|rebase|pull|stash|reset)
            mote snapshot --auto --trigger "git-$1" 2>/dev/null || true
            ;;
    esac
    return $status
}

# jj wrapper - auto-snapshot on change operations
jj() {
    command jj "$@"
    local status=$?
    case "$1" in
        edit|new|abandon|rebase|squash|restore|undo)
            mote snapshot --auto --trigger "jj-$1" 2>/dev/null || true
            ;;
    esac
    return $status
}
