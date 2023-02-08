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
            decompiler = {
              rpath = p: [ ];
              run_time_ld_library_path = p: [ ];
            };
          };
          extra-overrides = { mkNativeDep, mkEnvDep, p }:
            [ (mkNativeDep "decompiler" [ ]) ];
        };
        s2e = import ./nix/s2e.nix { inherit lib pkgs; };
      in {
        devShells = {
          default = rust.rustPkgs.workspaceShell {
            packages = let p = pkgs;
            in [
              cargo2nix.outputs.packages.${system}.cargo2nix
              p.rust-bin.nightly.latest.clippy
              p.rust-bin.nightly.latest.rustfmt
              p.rust-bin.stable.latest.default
            ] ++ builtins.attrValues rust.packages;
          };
          doc = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ tectonic gnumake ];
          };
        };

        packages = rust.packages // {
          default = rust.packages.decompiler;
          inherit (s2e) s2e-src s2e-lib s2e-llvm;
        };
      });
}
