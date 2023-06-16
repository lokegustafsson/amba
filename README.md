# AMBA

Short for Automatic and Manual methods for Binary Analysis, this is a continuation of a bachelor
thesis project. The thesis included producing the following video introducing symbolic execution and
demoing an earlier version of AMBA:

[<img src="https://img.youtube.com/vi/VE_4biDqmhQ/maxresdefault.jpg" width=50%>](https://youtu.be/VE_4biDqmhQ)

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

## Report

The project plan and report (both in Swedish for Chalmers-related regulatory
reasons) reside in the directories `doc/plan` and `doc/report`. They can be
built by entering the devshell using `nix develop`, navigating to their
respective directories and running `make`. For those who have not already
installed AMBA dependencies, the nix script `nix run '.#documents'` will build both
documents quicker than installing the devshell.

## License

AMBA is licensed under AGPLv3 or later. The main dependency S2E consists of
software components with various licenses, described in
<https://github.com/S2E/s2e/blob/master/LICENSE>. All Nix derivations should
have the proper license metadata.

## READMEs
* [amba/DEVELOPMENT.md](DEVELOPMENT.md)
* [amba/ARCHITECTURE.md](ARCHITECTURE.md)
