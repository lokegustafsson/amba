#!/bin/bash
# Generates the compile_flags.txt for libamba by evoking the make file target.

set -e

script_parent=$(dirname  -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
cd "$script_parent"
make -j -sC crates/libamba/ compile_flags.txt
