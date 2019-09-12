# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        windows)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # Build binary (already done by script.sh?)
    #cargo rustc --bin railplot --target $TARGET --release -- -C lto
    #cargo build --target $TARGET --release

    # Copy binary
    cp target/$TARGET/release/junction $stage/

    # Copy examples
    #for f in examples/bundled/*railml; do 
        #cp $f $stage/example-$(basename -- "$f").xml
    #done

    # Copy txt files
    #cp LICENSE $stage/LICENSE.txt
    cp README.md $stage/README.txt

    cd $stage
    case $TRAVIS_OS_NAME in
        linux)
            tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$BINNAME.tar.gz *
            ;;
        osx)
            tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$BINNAME.tar.gz *
            ;;
        windows)
            7z a $src/$CRATE_NAME-$TRAVIS_TAG-$BINNAME.zip *
            ;;
    esac
    cd $src

    rm -rf $stage
}

main
