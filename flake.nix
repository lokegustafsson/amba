{
  inputs = {
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-22.11";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  # Cache settings
  nixConfig = {
    extra-substituters = [ "https://nix.u3836.se/" ];
    extra-trusted-public-keys =
      [ "nix.u3836.se:t7H/bFWi14aBFYPE5A00eEQawd7Ssl/fXbq/2C+Bsrs=" ];
  };

  outputs =
    { self, nixpkgs, nixpkgs-stable, flake-utils, rust-overlay, cargo2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            cargo2nix.overlays.default
            (final: prev: { stable = nixpkgs-stable.legacyPackages.${system}; })
          ];
        };
        lib = nixpkgs.lib;

        s2e = import ./nix/s2e { inherit lib pkgs libamba; };
        libamba = import ./nix/libamba.nix { inherit lib pkgs s2e; };

        rust = import ./nix/rust.nix {
          inherit lib pkgs;
          workspace-binaries = {
            amba = {
              rpath = p: [ ];
              run_time_ld_library_path = p: [ ];
            };
          };
          extra-overrides = { mkNativeDep, mkEnvDep, p }: [
            (mkEnvDep "s2e" ({
              # For autocxx to run
              LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
            } // libamba.s2e-include-paths))
            (mkNativeDep "s2e" [ p.clang_14 ])

            # NOTE: This crate name should really be "amba", but that does not work for some reason
            (mkEnvDep "dummy-dep" {
              AMBA_DEPENDENCIES_DIR = "${s2e.amba-deps}";
              AMBA_SRC_DIR = ./.;
            })
          ];
        };
      in {
        devShells = {
          default = rust.rustPkgs.workspaceShell {
            packages = let p = pkgs;
            in [
              cargo2nix.outputs.packages.${system}.cargo2nix
              p.rust-bin.nightly.latest.clippy
              p.rust-bin.nightly.latest.rustfmt
              p.rust-bin.stable.latest.default
              p.rust-bin.stable.latest.rust-analyzer
            ];
          };
          doc = pkgs.mkShell {
            packages = let p = pkgs;
            in [ p.stable.tectonic p.texlab p.gnumake ];
          };
          c_dev = libamba.devShell;
          s2e = pkgs.mkShell { packages = [ s2e.s2e-env ]; };
        };

        packages = rust.packages // s2e // {
          inherit (libamba) libamba;
          default = rust.packages.amba;
        };
        apps = {
          s2e-env = {
            type = "app";
            program = "${s2e.s2e-env}/bin/s2e";
          };
        };
      });
}
