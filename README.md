# git-together

Following in the footsteps of [git-pair][gp] and [git-duet][gd], but using `git
config` to hold the authors and without needing to change your existing git
habits.

[gp]: https://github.com/pivotal/git_scripts
[gd]: https://github.com/git-duet/git-duet

## Usage

```bash
# `git-together` is meant to be aliased as `git`
alias git=git-together

# Setting the default domain
git config --add git-together.domain rocinante.com

# Adding a couple authors
git config --add git-together.authors.jh 'James Holden; jholden'
git config --add git-together.authors.nn 'Naomi Nagata; nnagata'

# Adding an author with a different domain
git config --add git-together.authors.ca 'Chrisjen Avasarala; avasarala@un.gov'

# Pairing
git with jh nn
# ...
git commit
```

Soloing and mobbing are set by simply passing in the right number of authors to
`git with`.

TODO: rotation, mobbing

### Technical Details

Under the hood, `git-together` will only act on the following `git` subcommands:

- `with`
- `commit`
- `merge`
- `revert`
- `rebase`

All other subcommands are passed straight through to `git`.

TODO: This is accomplished by passing `--signoff` to `commit`, `merge`, and `revert`
with the `GIT_COMMITTER_NAME` and `GIT_COMMITTER_EMAIL` environment variables
set appropriately. This is all done by `git-together` so you don't have to
think about it.

TODO: Interaction with other aliases, scripts, hub, etc.