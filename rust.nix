{ lib, pkgs, workspace-binaries, extra-overrides }:
let
  rustPkgs = pkgs.rustBuilder.makePackageSet {
    rustVersion = "latest";
    packageFun = import ./Cargo.nix;
    packageOverrides = p:
      let
        mkNativeDep = cratename: extra-deps:
          p.rustBuilder.rustLib.makeOverride {
            name = cratename;
            overrideAttrs = drv: {
              propagatedBuildInputs = (drv.propagatedBuildInputs or [ ])
                ++ extra-deps;
            };
          };
        mkEnvDep = cratename: env-deps:
          mkNativeDep cratename [
            (p.rustBuilder.overrides.patches.propagateEnv cratename
              (lib.attrsets.mapAttrsToList
                (name: value: { inherit name value; }) env-deps))
          ];
        mkRpath = cratename: libs:
          p.rustBuilder.rustLib.makeOverride {
            name = cratename;
            overrideAttrs = drv: {
              preFixup = let libPath = lib.makeLibraryPath libs;
              in ''
                patchelf --set-rpath "${libPath}" $out/bin/${cratename}
              '';
            };
          };
        mkLdLibraryPath = cratename: libs:
          mkNativeDep cratename [
            (p.rustBuilder.overrides.patches.propagateEnv cratename [{
              name = "LD_LIBRARY_PATH";
              value = lib.makeLibraryPath libs;
            }])
          ];
      in [
        (p.rustBuilder.rustLib.makeOverride {
          overrideArgs = old: {
            rustcLinkFlags = old.rustcLinkFlags or [ ] ++ [
              "-C"
              "linker=${pkgs.clang}/bin/clang"
              "-C"
              "link-arg=-fuse-ld=${pkgs.mold}/bin/mold"
            ];
          };
        })
        (p.rustBuilder.rustLib.makeOverride {
          registry = "registry+https://github.com/rust-lang/crates.io-index";
          overrideArgs = old: {
            rustcLinkFlags = old.rustcLinkFlags or [ ]
              ++ [ "--cap-lints" "warn" ];
          };
        })
      ] ++ (extra-overrides { inherit mkNativeDep mkEnvDep p; })
      ++ (builtins.concatLists (builtins.attrValues (builtins.mapAttrs
        (cratename:
          { rpath, run_time_ld_library_path }: [
            (p.rustBuilder.rustLib.makeOverride {
              name = cratename;
              overrideAttrs = drv: {
                preFixup = let libPath = lib.makeLibraryPath (rpath p);
                in ''
                  patchelf --set-rpath "${libPath}" $out/bin/${cratename}
                '';
              };
            })
            (mkLdLibraryPath cratename (run_time_ld_library_path p))
          ]) workspace-binaries)));
  };
  wrapIfHasLdLibraryPath = cratename:
    { rpath, run_time_ld_library_path }:
    let
      crate = rustPkgs.workspace.${cratename} { };
      libPath = lib.makeLibraryPath (run_time_ld_library_path pkgs);
    in (if libPath == "" then
      crate
    else
      pkgs.writeShellScriptBin cratename ''
        LD_LIBRARY_PATH="${libPath}" ${crate}/bin/${cratename}
      '');
in {
  inherit rustPkgs;
  packages = builtins.mapAttrs wrapIfHasLdLibraryPath workspace-binaries;
}
