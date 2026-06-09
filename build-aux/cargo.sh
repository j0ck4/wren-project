#!/bin/sh
set -eu

# Wrapper invoked by Meson's custom_target.
# Args:
#   $1 — CARGO_TARGET_DIR
#   $2 — rust profile: "debug" | "release"
#   $3 — binary name
#   $4 — output path that Meson expects
#   $5+ — extra cargo flags (e.g. --manifest-path …)

CARGO_TARGET_DIR="$1"
RUST_PROFILE="$2"
BIN_NAME="$3"
OUTPUT="$4"
shift 4

export CARGO_TARGET_DIR
# Honour a CARGO_HOME provided by the environment (the Flatpak/Flathub
# build sets it to the vendored offline registry); otherwise default to
# a directory under the target dir.
export CARGO_HOME="${CARGO_HOME:-$CARGO_TARGET_DIR/cargo-home}"

if [ "$RUST_PROFILE" = "release" ]; then
    cargo build --release "$@"
else
    cargo build "$@"
fi

cp "$CARGO_TARGET_DIR/$RUST_PROFILE/$BIN_NAME" "$OUTPUT"
