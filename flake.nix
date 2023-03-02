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
        rust = import ./nix/rust.nix {
          inherit lib pkgs;
          workspace-binaries = {
            amba = {
              rpath = p: [ ];
              run_time_ld_library_path = p: [ ];
            };
          };
          extra-overrides = { mkNativeDep, mkEnvDep, p }: [
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
            # NOTE: This crate name should really be "amba", but that does not work for some reason
            (mkEnvDep "handlebars" {
              AMBA_DEPENDENCIES_DIR = "${s2e.amba-deps}";
              AMBA_SRC_DIR = ./.;
            })
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
            packages = let p = pkgs; in [ p.stable.tectonic p.texlab p.gnumake ];
          };
          c_dev = pkgs.mkShell {
            # Libraries (because setting up pkgconfig is even worse)
            BOOST_PATH = "${pkgs.boost.dev}/include";
            CLANGLIBS_PATH = "${pkgs.clang_14}/resource-root/include";
            LLVM_PATH = "${pkgs.llvmPackages_14.llvm.dev}/include";
            GCCLIBS_PATH = "${pkgs.gcc-unwrapped}/include/c++/11.3.0";
            GCCLIBS_PATH_L =
              "${pkgs.gcc-unwrapped}/include/c++/11.3.0/x86_64-unknown-linux-gnu";
            GLIBC_PATH = "${pkgs.glibc.dev}/include";
            S2E_PATH = "${s2e.s2e-src}/s2e";
            ZYDIS_PATH = "${pkgs.zydis}";
            ZYCORE_PATH = "${pkgs.callPackage nix/zycore.nix {}}";

            # Packages
            packages = (with pkgs; [
              mold
              clang-tools_14
              gnumake
              gdb
            ]) ++ (with pkgs.llvmPackages_14; [
              # libcxxStdenv
              # libcxxClang
              # libcxx
              # libcxxabi
              # libclang
              # libunwind
            ]);
          };
          s2e = pkgs.mkShell { packages = [ s2e.s2e-env ]; };
        };

        packages = rust.packages // s2e // { default = rust.packages.amba; };
        # `nix run '.#build-guest-images' -- $BUILDDIR $OUTDIR`
        apps = {
          s2e-env = {
            type = "app";
            program = "${s2e.s2e-env}/bin/s2e";
          };
        };
      });
}
