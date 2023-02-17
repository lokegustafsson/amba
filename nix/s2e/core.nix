{ repositories, pkgs, lib }:
let
  makeIncludePath = lib.makeSearchPathOutput "dev" "include";

  s2e-src = pkgs.stdenvNoCC.mkDerivation {
    name = "s2e-src";
    inherit (repositories) scripts s2e;

    fake_clang_finder = pkgs.writeText "fake-clang-finder" ''
      #!${pkgs.python3Minimal}/bin/python3
      print("x86_64-linux-gnu-ubuntu-18.04")
    '';

    dontUnpack = true;
    buildPhase = ''
      cp -r $s2e ./s2e
      cp -r $scripts ./scripts
      chmod -R +w .
      ln -s ./scripts/Makefile ./Makefile
      ln -s ./scripts/Makefile.docker ./Makefile.docker

      cp $fake_clang_finder ./s2e/scripts/determine_clang_binary_suffix.py
    '';
    installPhase = ''
      mkdir -p $out
      cp -r . $out
    '';
  };
  fake-curl = let
    content =
      (builtins.mapAttrs (url: sha256: pkgs.fetchurl { inherit url sha256; }) {
        "https://www.lua.org/ftp/lua-5.3.4.tar.gz" =
          "sha256-9oGqUYIzvEB+I6zw9Yh8iE8XQ28ADUU7JJGp8RpSQAw=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//llvm-14.0.0.src.tar.xz" =
          "sha256-TfftULi3AXuQ3CIgL2tZ6QBqKalWgjjGryjfnASce5s=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//clang-14.0.0.src.tar.xz" =
          "sha256-9df/uG7Vf5fXxHHVQsTlaF20t1+4F8TD8Ce/pJ5WG5s=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//compiler-rt-14.0.0.src.tar.xz" =
          "sha256-J6t/z7IdEICTwL52ap7V/hjATk90+TYGlxGjEsiuA3c=";
        "https://www.prevanders.net/libdwarf-20190110.tar.gz" =
          "sha256-S6m363DlQLGg4WmisG7BR28hl7pb+iqMyScCAFxtMRw=";
        "https://github.com/Z3Prover/z3/releases/download/z3-4.7.1//z3-4.7.1-x64-ubuntu-16.04.zip" =
          "sha256-y1OgudLggk7yc7IghIPLy9rx2W3508YNNtC187ob37U=";
        "https://github.com/aquynh/capstone/archive/4.0.2.tar.gz" =
          "sha256-fIHXmAIvgedQfxpg1oF/Y6p25ImqTnBVJV8hoi9eUmo=";
        "https://github.com/protocolbuffers/protobuf/releases/download/v3.7.1/protobuf-cpp-3.7.1.tar.gz" =
          "sha256-l/bNqgck1ajNM3XV9c9L0lPVrVKRFU9TPtDZSp1QHvM=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//s2e32.sys" =
          "sha256-h+/M6MJEpOkTN5VwuEUqwxkKTD2GxDAxrphAnygk1vQ=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//s2e.inf" =
          "sha256-HZ4soKEoqYIrbd9tyvQtmMf15a/th5ky/YRejDfI0AQ=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//drvctl32.exe" =
          "sha256-QiD7z6cqI0slhLu17GNQmoQbRZAq2XfqVqkaRII9sOg=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//libs2e32.dll" =
          "sha256-siqSOLCV0SMx1gE7eEbTRIDVWR4/c2t2oJbyVsb5IWk=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//tickler32.exe" =
          "sha256-/dFzz0ZsFegxaptIVAQ3MOq3H1+DU9ZaQN4zham88Kw=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//s2e.sys" =
          "sha256-vdqJpTKvn0HSUjDlu7nktpF0kePNX58fQR1xgn2tYP8=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//drvctl.exe" =
          "sha256-6QmL4hurpPEPZ7H9KXyXakity91wpDIGfHaaTyNE36c=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//libs2e64.dll" =
          "sha256-m2F+ZpQEYCZ5/VvhTRjY+iTvikpH395+u44gbJZRooQ=";
        "https://github.com/S2E/s2e/releases/download/v2.0.0//tickler64.exe" =
          "sha256-aaMFczwxtp91/qCF86nl0X+Cjb6a7tEdmeu/LZGvJUA=";
      });
    dict = lib.strings.concatStringsSep ","
      (lib.attrsets.mapAttrsToList (url: drv: "'${url}': '${drv}'") content);
  in pkgs.writeScriptBin "curl" ''
    #!${pkgs.python3Minimal}/bin/python3
    import sys
    import shutil
    argv = tuple(sys.argv)
    if len(argv) == 6 and argv[1:3] == ("-f", "-L") and argv[4] == "-o":
      url = argv[3]
      target = argv[5]
      resolved = {${dict}}[url]
      shutil.copyfile(resolved, target)
    else:
      raise Exception("fake curl given", argv)
  '';
  clang_and_llvm = pkgs.symlinkJoin {
    name = "clang-and-llvm";
    paths = [ pkgs.clang_14 pkgs.llvmPackages_14.llvm ];
  };

  BUILD_ARCH = "haswell";
  INJECTED_CLANG_CC = "${clang_and_llvm}/bin/clang";
  INJECTED_CLANG_CXX = "${clang_and_llvm}/bin/clang++";
  INJECTED_RAPIDJSON_SRC = pkgs.fetchFromGitHub {
    owner = "Tencent";
    repo = "rapidjson";
    rev = "fd3dc29a5c2852df569e1ea81dbde2c412ac5051";
    sha256 = "sha256-r86AJJiJz2mv/2/NgtSObIVgGR3IljL3cbhYzAtrCzQ=";
  };
  LLVM_BUILD = "${s2e-llvm}";

  s2e-llvm = pkgs.stdenv.mkDerivation {
    name = "s2e-llvm";
    src = s2e-src;
    dontConfigure = true;
    dontInstall = true;
    patches = [ ../patches/s2e/makefile-llvm.patch ];
    buildPhase = ''
      mkdir -p $out
      mv * $out/
      cd $out
      S2E_PREFIX=$out make -f ./Makefile stamps/llvm-release-make
    '';
    buildInputs = [
      fake-curl
      pkgs.binutils-unwrapped
      pkgs.cmake
      pkgs.libxcrypt
      pkgs.python3Minimal
    ];
    inherit BUILD_ARCH INJECTED_CLANG_CC INJECTED_CLANG_CXX;
  };
  libgomp = let
    version = "11.3.0";
    src = pkgs.fetchzip {
      url = "mirror://gcc/releases/gcc-${version}/gcc-${version}.tar.xz";
      sha256 = "sha256-fI8Uu8GLuFfQnq09s6cgUYcAEzX1x6gWqymbb5njhQY=";
    };
  in pkgs.stdenv.mkDerivation {
    pname = "libgomp";
    inherit version src;
    unpackPhase = ''
      mkdir build2
      cd build2
      cp -r $src/* .
      cp -r $src/config-ml.in ..
      chmod -R +w .
    '';
    configurePhase = ''
      cd libgomp && ./configure --prefix=$out
    '';
  };
  s2e-lib = pkgs.stdenvNoCC.mkDerivation {
    name = "s2e-lib";
    src = s2e-src;
    dontConfigure = true;
    dontInstall = true;
    dontMoveLib64 = true;
    patches =
      [ ../patches/s2e/makefile-llvm.patch ../patches/s2e/makefile-git.patch ];
    buildPhase = ''
      mkdir -p $out

      S2E_PREFIX=$out make -f ./Makefile stamps/libs2e-release-install \
        stamps/libvmi-release-install stamps/llvm-release-install
    '';
    buildInputs = let p = pkgs;
    in [
      fake-curl
      p.clang_14
      p.cmake
      p.glib.dev
      p.libbsd
      p.libmemcached
      p.pkg-config
      p.unzip
    ];
    inherit BUILD_ARCH INJECTED_CLANG_CC INJECTED_CLANG_CXX;
    CPATH = (makeIncludePath [ pkgs.libelf pkgs.boost ]);
    LIBRARY_PATH = lib.makeLibraryPath [ pkgs.libelf pkgs.boost ];
    INJECTED_LIBS2E_CXXFLAGS =
      "-Wno-unused-command-line-argument -L${libgomp}/lib";

    INJECTED_SOCI_SRC = pkgs.fetchFromGitHub {
      owner = "SOCI";
      repo = "soci";
      rev = "438e3549594eb59d84b434c814647648e7c2f10a";
      sha256 = "sha256-HsQyHhW8EP7rK/Pdi1TSXee9yKJsujoDE9QkVdU9WIk=";
    };
    INJECTED_GTEST_SRC = pkgs.fetchurl {
      url =
        "https://github.com/google/googletest/archive/release-1.11.0.tar.gz";
      sha256 = "sha256-tIcL8SH/d5W6INILzdhie44Ijy0dqymaAxwQNO3ck9U=";
    };
    inherit INJECTED_RAPIDJSON_SRC LLVM_BUILD;
  };
  s2e-tools = pkgs.stdenvNoCC.mkDerivation {
    name = "s2e-tools";
    src = s2e-src;
    dontConfigure = true;
    dontInstall = true;
    dontMoveLib64 = true;
    patches =
      [ ../patches/s2e/makefile-llvm.patch ../patches/s2e/makefile-git.patch ];
    buildPhase = ''
      mkdir -p $out
      S2E_PREFIX=$out make -f ./Makefile stamps/tools-release-install
    '';
    buildInputs = let p = pkgs;
    in [ fake-curl p.clang_14 p.cmake p.glib.dev p.libbsd p.pkg-config ];
    inherit BUILD_ARCH INJECTED_CLANG_CC INJECTED_CLANG_CXX;
    CPATH = (makeIncludePath (let p = pkgs;
    in [ p.libelf p.boost p.glibc.dev p.pkgsCross.gnu32.glibc.dev ]));
    LIBRARY_PATH = lib.makeLibraryPath [ pkgs.libelf ];
    inherit INJECTED_RAPIDJSON_SRC LLVM_BUILD;
  };
  s2e-guest-tools = pkgs.stdenvNoCC.mkDerivation {
    name = "s2e-guest-tools";
    src = s2e-src;
    dontConfigure = true;
    dontInstall = true;
    dontMoveLib64 = true;
    patches =
      [ ../patches/s2e/makefile-llvm.patch ../patches/s2e/makefile-git.patch ];
    buildPhase = ''
      mkdir -p $out

      S2E_PREFIX=$out make -f ./Makefile stamps/guest-tools64-install

      #S2E_PREFIX=$out make -f ./Makefile stamps/guest-tools32-install
      #S2E_PREFIX=$out make -f ./Makefile stamps/guest-tools32-win-install
      #S2E_PREFIX=$out make -f ./Makefile stamps/guest-tools64-win-install
    '';
    buildInputs = let p = pkgs; in [ fake-curl p.clang_14 p.cmake p.nasm ];
    inherit BUILD_ARCH INJECTED_CLANG_CC INJECTED_CLANG_CXX;
    CPATH = (makeIncludePath [ pkgs.libelf ]);
    LIBRARY_PATH = lib.makeLibraryPath [ pkgs.libelf pkgs.glibc.static ];
  };
  s2e = pkgs.stdenvNoCC.mkDerivation {
    name = "s2e";
    phases = [ "installPhase" "fixupPhase" ];
    buildInputs = [ pkgs.rsync ];
    installPhase = ''
      rsync -a ${s2e-lib}/* ${s2e-tools}/* ${s2e-guest-tools}/* ${pkgs.qemu}/* $out/
      chmod -R +w $out
      rsync -a $out/lib64/* $out/lib/
      rm -r $out/lib64
    '';
  };
in { inherit s2e-src s2e-llvm s2e-lib s2e-tools s2e-guest-tools s2e libgomp; }
