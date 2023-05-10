# git-together

Following in the footsteps of [git-pair][gp] and [git-duet][gd], but without
needing to change your existing git habits.

This project is a fork of Pivotal Lab's [git-together][gt].

`git-together-ssh` is a bolt-on modification to `git-together`, adding functionality to select and use an SSH cert based on the user currently active. All config is compatible with `git-together`, except for the aliasing in `~/.zshrc`.

[gp]: https://github.com/pivotal/git_scripts
[gd]: https://github.com/git-duet/git-duet
[gt]: https://github.com/kejadlen/git-together

## Installation

```bash
brew tap --force-auto-update section-31/tap https://gitlab.com/section-31/homebrew-tap
brew install section-31/tap/git-together-ssh
```

## Configuration

Here's one way to configure `git-together-ssh`, but since it uses `git config` to
store information, there are many other ways to do it. This particular example
assumes a desire to store authors at the repo-level in a `.git-together` file.

```bash
# `git-together-ssh` is meant to be aliased as `git`
alias git=git-together-ssh

# Use .git-together per project for author configuration 
git config --add include.path ../.git-together
# Or use one .git-together for all projects
git config --global --add include.path ~/.git-together

# Setting the default domain
git config --file .git-together --add git-together.domain rocinante.com

# Adding a couple authors
git config --file .git-together --add git-together.authors.jh 'James Holden; jholden'
git config --file .git-together --add git-together.authors.nn 'Naomi Nagata; nnagata'

# Adding an author with a different domain
git config --file .git-together --add git-together.authors.ca 'Chrisjen Avasarala; avasarala@un.gov'
```

For completion with zsh, you'll need to update your `.zshrc` to copy the existing completion rules
from the main git binary

```zsh
# initialize the compinit system if not already
autoload -U compinit
compinit

# tell zsh to use the completion setup for the git when using git-together
compdef git-together-ssh=git
```

## Usage

```bash
# Pairing
git with jh nn
# ...
git commit

# Soloing
git with nn
# ...
git commit

# Mobbing
git with jh nn ca
# ...
git commit
```

Soloing and mobbing are automatically set by the number of authors passed to
`git with`. `git-together-ssh` rotates authors by default after making a commit so
that the author/committer roles are fairly spread across the pair/mob over
time.

Aliases are supported as well. You can make git-together do its thing when you
use an alias for a committing command by configuring a comma-separated list of
aliases:

```bash
git config git-together.aliases ci,rv,m
# ...
git ci
```

By default, `git-together` sets and rotates pairs for a single local
repository. If you are working across multiple repos with a pair on a regular
basis, this can be difficult to set across all of them. The `--global` flag can
be passed along to set a global pair. `git-together` will still default to a
local repository, so if you'd like to reset from local to global, you can use
the `--clear` flag.

```bash
# Set for all repos
git with --global jh nn

# Override in single repo
git with nn

# Clear local and move back to global
git with --clear
```

## Technical Details

Because repo-level authors are common and there's no good way of configuring
`git config` on cloning a repo, `git-together` will automatically include
`.git-together` to `git config` if it exists. (See `GitConfig::auto_include`
for details.) This allows `git-together` to work immediately on cloning a repo
without manual configuration.

Under the hood, `git-together` sets `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`,
`GIT_COMMITTER_NAME`, and `GIT_COMMITTER_EMAIL` for the `commit`, `merge`, and
`revert` subcommands so that git commits have the correct attribution..
`git-together` also adds the `--signoff` argument to the `commit` and `revert`
subcommands so that the commit message includes the `Signed-off-by: ` line.

## Known Issues

`git-together` works by aliasing `git` itself, so there are going to be issues
with git's in-built aliases as well as other utilities (such as [Hub][hub])
that work in the same manner.

[hub]: https://hub.github.com/

## Development

### Rust version

Install rust using the [rustup][rustup] tool. Installing from homebrew won't work
because some nightly features of rust are needed to build.

Then, switch to the nightly with

```bash
rustup default nightly
```

### Bats

[Bats][bats] is a bash testing framework, used here for integration tests. This
can be installed with homebrew.

```bash
brew install bats
```

[rustup]: https://www.rustup.rs/
[bats]: https://github.com/sstephenson/bats

### Testing

```bash
cargo test
./bats/integration.bats
```
