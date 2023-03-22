{ lib, pkgs, s2e, libamba }:
let
  bootstrap = import ./rust.nix {
    inherit lib;
    # Using mold breaks the build, and disabling mold is easier than fixing the
    # underlying problem
    use-mold = false;
    pkgs = pkgs.pkgsCross.musl64;
    extra-overrides = { mkNativeDep, mkEnvDep, mkOverride, p }: [ ];
  };
  amba-deps = pkgs.stdenvNoCC.mkDerivation {
    name = "amba-deps";
    phases = [ "installPhase" "fixupPhase" ];
    buildInputs = [ pkgs.rsync ];
    installPhase = ''
      mkdir -p $out/share/libs2e/ $out/bin/
      rsync -a ${s2e.s2e}/share/libs2e/* $out/share/libs2e/
      rsync -a ${s2e.s2e}/bin/guest-tools* $out/bin/
      rsync -a ${s2e.s2e}/bin/qemu-system-* $out/bin/
      cp ${bootstrap.bootstrap}/bin/bootstrap $out/bin/
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

  COMPILE_TIME_AMBA_DEPENDENCIES_DIR = "${amba-deps}";
  AMBA_BUILD_GUEST_IMAGES_SCRIPT =
    "${s2e.build-guest-images}/bin/build-guest-images";

  rust = import ./rust.nix {
    inherit lib pkgs;
    extra-overrides = { mkNativeDep, mkEnvDep, mkRpath, mkOverride, p }: [
      (mkEnvDep "s2e" ({
        # For autocxx to run
        LIBCLANG_PATH = "${pkgs.llvmPackages_14.libclang.lib}/lib";
      } // libamba.s2e-include-paths))
      (mkNativeDep "s2e" [ p.clang_14 ])

      # GUI
      (mkNativeDep "expat-sys" [ p.cmake ])
      (mkNativeDep "freetype-sys" [ p.cmake p.freetype p.pkg-config ])
      (mkNativeDep "servo-fontconfig-sys" [ p.pkg-config p.fontconfig ])

      (mkEnvDep "amba" {
        inherit COMPILE_TIME_AMBA_DEPENDENCIES_DIR
          AMBA_BUILD_GUEST_IMAGES_SCRIPT;
      })
      (mkRpath "amba" [
        p.fontconfig
        p.freetype
        p.libGL
        p.wayland
        p.xorg.libX11
        p.xorg.libXcursor
        p.xorg.libXi
        p.xorg.libXrandr
      ])
    ];
  };

  amba-wrapped = pkgs.writeShellScriptBin "amba-wrapped" ''
    ${pkgs.nixgl.nixGLIntel}/bin/nixGLIntel ${rust.amba}/bin/amba "$@"
  '';

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

      echo 'Patchelfing libs2e-x86_64-*.so (1/3)'
      patchelf --remove-needed libamba.so \
        target/impure-amba-deps/share/libs2e/libs2e-x86_64-*.so

      echo 'Patchelfing libs2e-x86_64-*.so (2/3)'
      patchelf --shrink-rpath \
        target/impure-amba-deps/share/libs2e/libs2e-x86_64-*.so

      echo 'Patchelfing libs2e-x86_64-*.so (3/3)'
      patchelf --add-needed libamba.so \
        --add-rpath "$PWD""/crates/libamba" \
        target/impure-amba-deps/share/libs2e/libs2e-x86_64-*.so

      echo 'Running amba'
      RUN_TIME_AMBA_DEPENDENCIES_DIR="$PWD""/target/impure-amba-deps" ${rust.amba}/bin/amba "$@"
    '';
  };
in {
  inherit COMPILE_TIME_AMBA_DEPENDENCIES_DIR AMBA_BUILD_GUEST_IMAGES_SCRIPT
    amba-deps impure-amba amba-wrapped;
  inherit (rust) amba workspaceShell;
}
