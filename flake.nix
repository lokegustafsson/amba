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
    nixgl = {
      url = "github:guibou/nixGL";
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

  outputs = { self, nixpkgs, nixpkgs-stable, flake-utils, rust-overlay
    , cargo2nix, nixgl }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            cargo2nix.overlays.default
            (final: prev: { stable = nixpkgs-stable.legacyPackages.${system}; })
            nixgl.overlay
          ];
        };
        lib = nixpkgs.lib;

        s2e = import ./nix/s2e { inherit lib pkgs; };
        libamba = import ./nix/libamba.nix { inherit lib pkgs s2e; };
        amba = import ./nix/amba.nix { inherit lib pkgs s2e libamba; };
        test = import ./nix/test.nix { inherit lib pkgs amba; };
      in {
        devShells = {
          default = amba.workspaceShell ({
            packages = let p = pkgs;
            in [
              amba.impure-amba
              cargo2nix.outputs.packages.${system}.cargo2nix
              p.cargo-flamegraph
              p.clang-tools_14
              p.ctags
              p.gdb
              p.gnumake
              p.mold
              p.nixfmt
              p.nixgl.nixGLIntel
              p.rust-bin.nightly.latest.rustfmt
              p.rust-bin.stable.latest.clippy
              p.rust-bin.stable.latest.default
              p.rust-bin.stable.latest.rust-analyzer
              p.stable.tectonic
              p.texlab
            ];
            IMPURE_RUST = 1;
            inherit (amba)
              COMPILE_TIME_AMBA_DEPENDENCIES_DIR AMBA_BUILD_GUEST_IMAGES_SCRIPT;

            LD_LIBRARY_PATH =
              lib.strings.makeLibraryPath (amba.gui-native-dependencies pkgs);

            meta.description =
              "Rust, C++ and LaTeX tooling for developing AMBA";
          } // libamba.all-include-paths);
        };

        packages = {
          inherit (libamba) libamba;
          inherit (amba) amba-deps impure-amba;
          inherit (s2e) s2e s2e-src build-guest-images guest-images-src;
          inherit (test) control-flow;
          inherit (amba) amba amba-wrapped;
          default = amba.amba-wrapped;
        };
        apps = {
          s2e-env = {
            type = "app";
            program = "${s2e.s2e-env}/bin/s2e";
          };
          documents = {
            type = "app";
            program = builtins.toString
              (pkgs.writeShellScript "build-documents" ''
                export PATH=${
                  lib.strings.makeBinPath [
                    pkgs.coreutils
                    pkgs.gnumake
                    pkgs.stable.tectonic
                  ]
                }
                make -C doc/plan
                make -C doc/report
              '');
          };
          test-amba-hello = {
            type = "app";
            program = "${test.test-amba-hello}/bin/test-amba-hello";
          };
          test-amba-control-flow = {
            type = "app";
            program =
              "${test.test-amba-control-flow}/bin/test-amba-control-flow";
          };
          test-amba-state-splitter = {
            type = "app";
            program =
              "${test.test-amba-state-splitter}/bin/test-amba-state-splitter";
          };
          run-amba-hello = {
            type = "app";
            program = "${test.run-amba-hello}/bin/run-amba-hello";
          };
          run-amba-control-flow = {
            type = "app";
            program =
              "${test.run-amba-control-flow}/bin/run-amba-control-flow";
          };
          run-amba-state-splitter = {
            type = "app";
            program =
              "${test.run-amba-state-splitter}/bin/run-amba-state-splitter";
          };
        };
      });
}
