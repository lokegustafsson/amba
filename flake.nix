{
  inputs = {
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

  outputs = { self, nixpkgs, flake-utils, rust-overlay, cargo2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default ];
        };
        lib = nixpkgs.lib;
        rust = import ./nix/rust.nix {
          inherit lib pkgs;
          workspace-binaries = {
            amba = {
              rpath = p: [ ];
              run_time_ld_library_path = p: [ ];
            };
          };
          extra-overrides = { mkNativeDep, mkEnvDep, p }: [
            (mkNativeDep "amba" [ ])
            (mkEnvDep "s2e" {
              # Required to parse s2e headers
              BOOST_PATH = "${pkgs.boost.dev}/include";
              CLANGLIBS_PATH = "${pkgs.clang_14}/resource-root/include";
              LLVM_PATH = "${pkgs.llvmPackages_14.llvm.dev}/include";
              GCCLIBS_PATH = "${pkgs.gcc-unwrapped}/include/c++/11.3.0";
              GCCLIBS_PATH_L =
                "${pkgs.gcc-unwrapped}/include/c++/11.3.0/x86_64-unknown-linux-gnu";
              GLIBC_PATH = "${pkgs.glibc.dev}/include";
              S2E_PATH = "${s2e.s2e-src}/s2e";

              # For autocxx to run
              LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
            })
            (mkNativeDep "s2e" [ p.clang_14 ])
          ];
        };
        s2e = import ./nix/s2e { inherit lib pkgs; };
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
            packages = let p = pkgs; in [ p.tectonic p.gnumake ];
          };
          s2e = pkgs.mkShell { packages = [ s2e.s2e-env ]; };
          guest = s2e.guest-images-shell;
        };

        packages = rust.packages // {
          default = rust.packages.amba;
          inherit (s2e)
            s2e s2e-qemu s2e-env guest-images guest-kernel32 guest-kernel64;
        };
      });
}
