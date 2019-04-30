#!/usr/bin/env sh

set -ex

: "${TARGET?The TARGET environment variable must be set.}"

# Tests are all super fast anyway, and they fault often enough on travis that
# having only one thread increases debuggability to be worth it.
#export RUST_BACKTRACE=full
#export RUST_TEST_NOCAPTURE=1

RUSTFLAGS="$RUSTFLAGS --cfg stdsimd_strict"

case ${TARGET} in
    # On 32-bit use a static relocation model which avoids some extra
    # instructions when dealing with static data, notably allowing some
    # instruction assertion checks to pass below the 20 instruction limit. If
    # this is the default, dynamic, then too many instructions are generated
    # when we assert the instruction for a function and it causes tests to fail.
    #
    # It's not clear why `-Z plt=yes` is required here. Probably a bug in LLVM.
    # If you can remove it and CI passes, please feel free to do so!
    i686-* | i586-*)
        export RUSTFLAGS="${RUSTFLAGS} -C relocation-model=static -Z plt=yes"
        ;;
esac

echo "RUSTFLAGS=${RUSTFLAGS}"
echo "FEATURES=${FEATURES}"
echo "OBJDUMP=${OBJDUMP}"
echo "STDSIMD_DISABLE_ASSERT_INSTR=${STDSIMD_DISABLE_ASSERT_INSTR}"
echo "STDSIMD_TEST_EVERYTHING=${STDSIMD_TEST_EVERYTHING}"
echo "CROSS=${CROSS}"

cargo_setup() {
    if [ "$CROSS" = "1" ]
    then
        export RUST_TARGET_PATH="/checkout/ci/cross"
        echo "RUST_TARGET_PATH=${RUST_TARGET_PATH}"
    fi
}

cargo_test() {
    if [ "$CROSS" = "1" ]
    then
        cmd="/cargo-h/bin/xargo"
    else
        cmd="cargo"
    fi
    subcmd="test"
    if [ "$NORUN" = "1" ]
    then
        if [ "$CROSS" = "1" ]
        then
            export subcmd="rustc"
        else
            export subcmd="build"
        fi
    fi
    cmd="$cmd ${subcmd} --target=$TARGET $1"
    if [ "$NOSTD" = "1" ]
    then
        cmd="$cmd -p coresimd"
    else
        cmd="$cmd -p coresimd -p stdsimd"
    fi
    cmd="$cmd -- $2"
    if [ "$NORUN" != "1" ]
    then
      if [ "$TARGET" != "wasm32-unknown-unknown" ]
      then
        cmd="$cmd --quiet"
      fi
    fi
    if [ "$CROSS" = "1" ]
    then
        cmd="$cmd --emit=asm"
    fi
    $cmd
}

cargo_output() {
    if [ "$CROSS" = "1" ]
    then
        find /checkout/target -name "*.s"
    fi
}

cargo_setup
cargo_test
cargo_test "--release"
cargo_output

# Test targets compiled with extra features.
case ${TARGET} in
    x86*)
        export STDSIMD_DISABLE_ASSERT_INSTR=1
        export RUSTFLAGS="${RUSTFLAGS} -C target-feature=+avx"
        cargo_test "--release"
        ;;
    wasm32-unknown-unknown*)
        # There's no node or other runtime which supports the most recent SIMD
        # proposal, but hopefully that's coming soon! For now just test that we
        # can codegen with no LLVM faults, and we'll remove `--no-run` at a
        # later date.
        export RUSTFLAGS="${RUSTFLAGS} -C target-feature=+simd128"
        export RUSTFLAGS="${RUSTFLAGS} -Cllvm-args=-wasm-enable-unimplemented-simd"
        cargo_test "--release --no-run"
        ;;
    *)
        ;;
esac
