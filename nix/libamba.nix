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

  devShell = pkgs.mkShell ({
    packages = let p = pkgs;
    in [ p.mold p.clang-tools_14 p.gnumake p.gdb p.ctags ];
  } // all-include-paths);

in { inherit s2e-include-paths libamba devShell; }
