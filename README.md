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

If you get the following warning when running nix `ignoring untrusted
substituter 'https://nix.u3836.se/`, the issue could be that you are not in the
list of trusted users in your nix config. Add config flags `trusted-users =
root $USER` where `$USER` is your username. One way is to edit your
`/etc/nix/nix.conf` file and add the above flag.

You can run AMBA directly through `nix run . -- --help`. You can configure the
directory where guest vm images and session files are placed by setting the
environment variable `AMBA_DATA_DIR`, which defaults to `$XDG_DATA_HOME/amba` or
`$HOME/.local/share/amba`.
