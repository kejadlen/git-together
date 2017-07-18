# git-together

[![Build Status](https://travis-ci.org/kejadlen/git-together.svg?branch=master)](https://travis-ci.org/kejadlen/git-together)

Following in the footsteps of [git-pair][gp] and [git-duet][gd], but without
needing to change your existing git habits.

[gp]: https://github.com/pivotal/git_scripts
[gd]: https://github.com/git-duet/git-duet

## Usage

### Configuration

Here's one way to configure `git-together`, but since it uses `git config` to
store information, there are many other ways to do it. This particular example
assumes a desire to store authors at the repo-level in a `.git-together` file.

```bash
# `git-together` is meant to be aliased as `git`
alias git=git-together

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

### Usage

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

Soloing and mobbing are set by simply passing in the right number of authors to
`git with`. `git-together` automatically rotates authors after making a commit
so that the author/committer roles are fairly spread across the pair/mob over
time.

### Technical Details

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

### Known Issues

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
