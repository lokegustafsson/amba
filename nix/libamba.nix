{ lib, pkgs, rust, s2e }:
let

  s2e-include-paths = {
    BOOST_PATH = "${pkgs.boost.dev}/include";
    CLANGLIBS_PATH = "${pkgs.clang_14}/resource-root/include";
    LLVM_PATH = "${pkgs.llvmPackages_14.llvm.dev}/include";
    GCCLIBS_PATH = "${pkgs.gcc-unwrapped}/include/c++/11.3.0";
    GCCLIBS_PATH_L =
      "${pkgs.gcc-unwrapped}/include/c++/11.3.0/x86_64-unknown-linux-gnu";
    GLIBC_PATH = "${pkgs.glibc.dev}/include";
    S2E_PATH = "${s2e.s2e-src}/s2e";
  };

  all-include-paths = s2e-include-paths // {
    ZYDIS_PATH = "${pkgs.zydis}";
    ZYCORE_PATH = "${pkgs.callPackage ./zycore.nix { }}";
  };

  libamba = pkgs.stdenv.mkDerivation ({
    name = "libamba";
    src = ../crates/libamba;

    buildInputs = let p = pkgs; in [ p.boost p.zydis ];

    installPhase = ''
      mkdir -p $out/lib
      cp libamba.so $out/lib
    '';
  } // all-include-paths);

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
      rsync -a ${s2e.amba-deps}/* target/impure-amba-deps
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

  devShell = pkgs.mkShell ({
    packages = let p = pkgs;
    in [ p.mold p.clang-tools_14 p.gnumake p.gdb p.ctags impure-amba ];
  } // all-include-paths);

in { inherit s2e-include-paths libamba devShell; }
