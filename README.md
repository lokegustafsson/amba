# AMBA

Automatic and Manual methods for Binary Analysis

A bachelor thesis project

## System requirements

- An x86_64 linux system
- Maybe 20GB of disk space
- Maybe 16GB RAM or 8GB+swap+patience, if not using the `nix.u3836.se` binary cache
- A functioning [Nix](https://github.com/NixOS/nix) installation,
- with config flags `experimental-features = nix-command flakes`

## Instructions for running and building

Build AMBA by running `nix build`. This might take anywhere from 15 minutes to
several hours depending on your hardware, network speed and the population of
the `nix.u3836.se` cache.

You can run AMBA directly through `nix run . -- --help`. You can configure the
directory where guest vm images and session files are placed by setting the
environment variable `AMBA_DATA_DIR`, which defaults to `$HOME/amba`.
