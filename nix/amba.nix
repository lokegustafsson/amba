{ lib, pkgs, s2e, libamba }:
let
  amba-deps = pkgs.stdenvNoCC.mkDerivation {
    name = "amba-deps";
    phases = [ "installPhase" "fixupPhase" ];
    buildInputs = [ pkgs.rsync ];
    installPhase = ''
      mkdir -p $out/share/libs2e/ $out/bin/
      rsync -a ${s2e.s2e}/share/libs2e/* $out/share/libs2e/
      rsync -a ${s2e.s2e}/bin/guest-tools* $out/bin/
      rsync -a ${s2e.s2e}/bin/qemu-system-* $out/bin/
    '';
    fixupPhase = ''
      chmod -R u+w $out/share/libs2e/*

      patchelf \
        --add-needed libamba.so \
        --add-rpath ${libamba.libamba}/lib \
        $out/share/libs2e/libs2e-*.so
    '';

    meta = {
      homepage = "https://github.com/lokegustafsson/amba";
      description = "Run time dependencies of AMBA";
      license = lib.licenses.agpl3Plus;
    };
  };

  AMBA_DEPENDENCIES_DIR = "${amba-deps}";
  AMBA_BUILD_GUEST_IMAGES_SCRIPT =
    "${s2e.build-guest-images}/bin/build-guest-images";

  rust = import ./rust.nix {
    inherit lib pkgs;
    workspace-binaries = {
      amba = {
        rpath = p: [ ];
        run_time_ld_library_path = p: [ ];
      };
    };
    extra-overrides = { mkNativeDep, mkEnvDep, mkOverride, p }: [
      (mkEnvDep "s2e" ({
        # For autocxx to run
        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
      } // libamba.s2e-include-paths))
      (mkNativeDep "s2e" [ p.clang_14 ])

      (mkEnvDep "amba" {
        inherit AMBA_DEPENDENCIES_DIR AMBA_BUILD_GUEST_IMAGES_SCRIPT;
      })
    ];
  };

  impure-amba = pkgs.writeShellApplication {
    name = "impure-amba";
    runtimeInputs = let p = pkgs; in [ p.rsync p.patchelf p.gnumake ];
    text = ''
      SHOULD_BE_AMBA=$(basename "$PWD")
      if [ "$SHOULD_BE_AMBA" != "amba" ]; then
        echo "error: run this from the amba checkout directory"
        exit 1
      fi

      echo 'Rsyncing amba-deps to target/impure-amba-deps'
      mkdir -p target/impure-amba-deps
      rsync -a ${amba-deps}/* target/impure-amba-deps
      chmod -R u+w target/impure-amba-deps

      echo 'Making libamba.so'
      make -sC crates/libamba libamba.so

      echo 'Patchelfing libs2e-x86_64-*.so (1/2)'
      patchelf --shrink-rpath --remove-needed libamba.so \
        target/impure-amba-deps/share/libs2e/libs2e-x86_64-*.so

      echo 'Patchelfing libs2e-x86_64-*.so (2/2)'
      patchelf --add-needed libamba.so \
        --add-rpath "$PWD""/crates/libamba" \
        target/impure-amba-deps/share/libs2e/libs2e-x86_64-*.so

      echo 'Running amba'
      AMBA_DEPENDENCIES_DIR="$PWD""/target/impure-amba-deps" ${rust.packages.amba}/bin/amba "$@"
    '';
  };
in {
  inherit AMBA_DEPENDENCIES_DIR AMBA_BUILD_GUEST_IMAGES_SCRIPT amba-deps rust
    impure-amba;
}
