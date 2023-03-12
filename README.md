# AMBA

Automatic and Manual methods for Binary Analysis

A bachelor thesis project

## System requirements

- An x86_64 linux system supporting kvm (so running within a linux VM would
    require the host to support nested virualization)
- Maybe 20GB of disk space
- Maybe 16GB RAM or 8GB+swap+patience, if not using the `nix.u3836.se` binary
    cache
- A functioning [Nix](https://github.com/NixOS/nix) installation,
- with the `/etc/nix/nix.conf` settings including
```
trusted-users = root <YOUR USERNAME>
experimental-features = nix-command flakes
```
replacing `<YOUR USERNAME>` with your username. On NixOS you configure this in
your system config.

Missing the `trusted-users`-line may manifest in the warning `ignoring untrusted
substituter 'https://nix.u3836.se'`, followed by the build not using the binary
cache.

## Instructions for running and building

Build AMBA by running `nix build`. This might take anywhere from 15 minutes to
several hours depending on your hardware, network speed and the population of
the `nix.u3836.se` cache.

You can run AMBA directly through `nix run . -- --help`. You can configure the
directory where guest VM images and session files are placed by setting the
environment variable `AMBA_DATA_DIR`, which defaults to `$XDG_DATA_HOME/amba` or
`$HOME/.local/share/amba`.
