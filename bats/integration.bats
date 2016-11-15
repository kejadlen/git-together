#!/usr/bin/env bats

@test "soloing" {
  git-together with jh
  touch foo
  git add foo
  git-together commit -m "add foo"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ ! "$output" =~ "Signed-off-by:" ]]
}

@test "pairing" {
  git-together with jh nn
  touch foo
  git add foo
  git-together commit -m "add foo"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ "$output" =~ "Signed-off-by: Naomi Nagata <nnagata@rocinante.com>" ]]
}

@test "rotation" {
  git-together with jh nn

  touch foo
  git add foo
  git-together commit -m "add foo"

  touch bar
  git add bar
  git-together commit -m "add bar"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ "$output" =~ "Signed-off-by: James Holden <jholden@rocinante.com>" ]]
}

@test "mobbing" {
  git-together with jh nn ca

  touch foo
  git add foo
  git-together commit -m "add foo"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ "$output" =~ "Signed-off-by: Naomi Nagata <nnagata@rocinante.com>" ]]

  touch bar
  git add bar
  git-together commit -m "add bar"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "Chrisjen Avasarala <avasarala@un.gov>" ]
  run git show --no-patch --format=%B
  [[ "$output" =~ "Signed-off-by: Chrisjen Avasarala <avasarala@un.gov>" ]]

  touch baz
  git add baz
  git-together commit -m "add baz"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "Chrisjen Avasarala <avasarala@un.gov>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ "$output" =~ "Signed-off-by: James Holden <jholden@rocinante.com>" ]]
}

@test "auto-including .git-together" {
  git-together with jh
  run git config --local include.path
  [ "$status" -eq 1 ]

  touch .git-together

  git-together with jh
  run git config --local include.path
  [ "$output" = "../.git-together" ]

  git-together with jh
  run git config --local --get-all include.path
  [ "$output" = "../.git-together" ]
}

@test "git with no-one" {
  git-together with jh

  run git-together with
  expected=$(cat <<AUTHORS
ca: Chrisjen Avasarala <avasarala@un.gov>
jh: James Holden <jholden@rocinante.com>
nn: Naomi Nagata <nnagata@rocinante.com>
AUTHORS
)
  [[ "$output" =~ "$expected" ]]

  run git config git-together.active
  [ "$output" = "" ]
}

@test "no signoff" {
  git-together with jh nn
  touch foo
  git add foo
  GIT_TOGETHER_NO_SIGNOFF=1 git-together commit -m "add foo"

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ ! "$output" =~ "Signed-off-by: Naomi Nagata <nnagata@rocinante.com>" ]]
}

@test "merging" {
  git-together with jh nn
  touch foo
  git add foo
  git-together commit -m "add foo"

  git checkout -b bar
  touch bar
  git add bar
  git-together commit -m "add bar"

  git checkout -
  git-together merge --no-edit --no-ff bar

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ ! "$output" =~ "Signed-off-by:" ]]
}

@test "reverting" {
  git-together with jh nn
  touch foo
  git add foo
  git-together commit -m "add foo"
  git-together revert --no-edit HEAD

  run git show --no-patch --format="%aN <%aE>"
  [ "$output" = "Naomi Nagata <nnagata@rocinante.com>" ]
  run git show --no-patch --format="%cN <%cE>"
  [ "$output" = "James Holden <jholden@rocinante.com>" ]
  run git show --no-patch --format=%B
  [[ "$output" =~ "Signed-off-by: James Holden <jholden@rocinante.com>" ]]
}

@test "not in a git repo" {
  cd $BATS_TMPDIR

  run git-together with
  [ "$status" -eq 0 ]
}

setup() {
  # [ -f $BATS_TMPDIR/bin/git-together ] || cargo install --root $BATS_TMPDIR
  rm -rf $BATS_TMPDIR/bin
  cargo install --root $BATS_TMPDIR
  PATH=$BATS_TMPDIR/bin:$PATH

  rm -rf $BATS_TMPDIR/$BATS_TEST_NAME
  mkdir -p $BATS_TMPDIR/$BATS_TEST_NAME
  cd $BATS_TMPDIR/$BATS_TEST_NAME

  git init
  git config --add git-together.domain rocinante.com
  git config --add git-together.authors.jh "James Holden; jholden"
  git config --add git-together.authors.nn "Naomi Nagata; nnagata"
  git config --add git-together.authors.ca "Chrisjen Avasarala; avasarala@un.gov"
}
