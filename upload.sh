#!/usr/bin/env sh

set -e

nix build . --accept-flake-config
doas nix store sign --key-file ./eurydice-private-key --recursive ./result-bin
nix store verify --trusted-public-keys $(nix key convert-secret-to-public < ./eurydice-private-key) .
nix copy . --to ssh://eurydice
