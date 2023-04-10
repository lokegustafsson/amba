{ lib, pkgs, s2e }:
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
    LUA_PATH = "${pkgs.lua53Packages.lua}/include";
  };

  all-include-paths = s2e-include-paths // {
    ZYDIS_PATH = "${pkgs.zydis}";
    ZYCORE_PATH = "${pkgs.callPackage ./zycore.nix { }}";
    LIBAMBA_PATH = "${rust.libamba}/lib/";
  };

  rust = import ./rust.nix {
    inherit lib pkgs;
    extra-overrides = { mkNativeDep, mkEnvDep, mkRpath, mkOverride, p }: [
      (mkOverride "libamba" (old: { dontFixup = true; }))
    ];
  };

  libamba = pkgs.stdenv.mkDerivation ({
    name = "libamba";
    src = ../crates/AmbaPlugin;

    OPT-LEVEL = "-O3 -g0";
    buildInputs = let p = pkgs; in [ p.boost p.zydis ];

    installPhase = ''
      mkdir -p $out/lib
      cp libamba.so $out/lib
    '';

    meta = {
      homepage = "https://github.com/lokegustafsson/amba";
      description = "The S2E plugin part of AMBA";
      license = lib.licenses.agpl3Plus;
    };
  } // all-include-paths);

in { inherit all-include-paths s2e-include-paths libamba; }
