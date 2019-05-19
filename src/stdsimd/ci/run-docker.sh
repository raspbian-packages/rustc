#!/usr/bin/env sh

# Small script to run tests for a target (or all targets) inside all the
# respective docker images.

set -ex

run() {
    target=$(echo "${1}" | sed 's/-emulated//')
    echo "Building docker container for TARGET=${1}"
    docker build -t stdsimd -f "ci/docker/${1}/Dockerfile" ci/
    mkdir -p target
    echo "Running docker"
    # shellcheck disable=SC2016
    docker run \
      --user "$(id -u)":"$(id -g)" \
      --rm \
      --init \
      --volume "${HOME}"/.cargo:/cargo-h \
      --env CARGO_HOME=/cargo-h \
      --volume "$(rustc --print sysroot)":/rust:ro \
      --env TARGET="${target}" \
      --env STDSIMD_TEST_EVERYTHING \
      --env STDSIMD_ASSERT_INSTR_IGNORE \
      --env STDSIMD_DISABLE_ASSERT_INSTR \
      --env NOSTD \
      --env NORUN \
      --env RUSTFLAGS \
      --env STDSIMD_TEST_NORUN \
      --volume "$(pwd)":/checkout:ro \
      --volume "$(pwd)"/target:/checkout/target \
      --workdir /checkout \
      --privileged \
      stdsimd \
      bash \
      -c 'PATH=/rust/bin:$PATH exec ci/run.sh'
}

if [ -z "$1" ]; then
  for d in ci/docker/*; do
    run "${d}"
  done
else
  run "${1}"
fi
