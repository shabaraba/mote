# mote shell integration for git/jj auto-snapshot
# Add this to your ~/.config/fish/config.fish

function git --wraps git
    command git $argv
    set -l status_code $status
    switch $argv[1]
        case checkout switch merge rebase pull stash reset
            mote snapshot --auto --trigger "git-$argv[1]" 2>/dev/null; or true
    end
    return $status_code
end

function jj --wraps jj
    command jj $argv
    set -l status_code $status
    switch $argv[1]
        case edit new abandon rebase squash restore undo
            mote snapshot --auto --trigger "jj-$argv[1]" 2>/dev/null; or true
    end
    return $status_code
end
