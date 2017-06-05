# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    # check format, if there is a diff, exit code 4 is returned
    # cargo fmt -- --write-mode=diff

    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ -n $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release

    # cross run --target $TARGET
    # cross run --target $TARGET --release

    ~/bin/bats --tap bats
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
