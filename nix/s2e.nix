{ pkgs, lib }:
let
  makeIncludePath = lib.makeSearchPathOutput "dev" "include";

  # From (https://github.com/S2E/manifest/blob/master/default.xml)
  repositories = builtins.listToAttrs (builtins.map (set: {
    name = set.repo;
    value = pkgs.fetchFromGitHub ({ owner = "S2E"; } // set);
  }) [
    {
      repo = "scripts";
      rev = "cfc158d7b82b55e21982e04cf9109f09cb3ed614";
      sha256 = "sha256-LI7KChvD1TmQUZqCYQ2rXHfcKUUemklq80nZAilzQ44=";
    }
    {
      repo = "decree";
      rev = "a523ec2ec1ca1e1369b33db755bed135af57e09c";
      sha256 = "sha256-BziFix8sUWvvpquv+9xvLoVL+gI/VKD0Gmn6LGaZACo=";
    }
    {
      repo = "guest-images";
      rev = "70c8591cf109d12eb35899569190a7fb1b9ae31b";
      sha256 = "sha256-oa513Tlgu8S8G9CCb0Q/tvmxsjLL0tVtTDCU2nkSJnQ=";
    }
    {
      repo = "qemu";
      rev = "638782a47ed9bb3f280b57a3627bb4e11b2a9cf1";
      sha256 = "sha256-hGcUKp+hXjZNYxJ2fdRSAbGM+4u5fKiwUDlyyRQS8Lw=";
    }
    {
      repo = "s2e";
      rev = "60a21a84fa1ab4754c1067f4efa3188feba59dcb";
      sha256 = "sha256-zeySmRIneMUfhcYljyO8NRXU95a7twFen93xNOA9gdI=";
    }
    {
      repo = "s2e-env";
      rev = "98d68b694b18ed24760e67caa07885b57bba9ca8";
      sha256 = "sha256-zV0Uk5iu3H7EWXpmkGrJz2gs2nlSgLPibg8n2i0Ho4I=";
    }
    {
      repo = "s2e-linux-kernel";
      rev = "81dcf04137d1ff68989d7823dc0689751affe3cd";
      sha256 = "sha256-803cDp4gw9Lw8gQmfUwm4NMpG5NZGhiPrxRm7RJZinw=";
    }
  ]);
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
  s2e-llvm = pkgs.stdenv.mkDerivation {
    name = "s2e-llvm";
    src = s2e-src;
    dontConfigure = true;
    dontInstall = true;
    patches = [ ./makefile-llvm.patch ];
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
    BUILD_ARCH = "haswell";
    LD_PRELOAD_PATH = lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib ];
    INJECTED_CLANG_CC = "${pkgs.clang_14}/bin/clang";
    INJECTED_CLANG_CXX = "${pkgs.clang_14}/bin/clang++";
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
    patches = [ ./makefile-llvm.patch ./makefile-git.patch ];
    buildPhase = ''
      mkdir -p $out
      #S2E_PREFIX=$out make -f ./Makefile install
      S2E_PREFIX=$out make -f ./Makefile stamps/libs2e-release-install
    '';
    buildInputs = let p = pkgs;
    in [
      fake-curl
      p.clang_14
      p.cmake
      p.glib.dev
      p.libbsd
      p.libmemcached
      p.libxcrypt
      p.pkg-config
      p.python3Minimal
      p.unzip
    ];
    BUILD_ARCH = "haswell";
    CPATH = (makeIncludePath (let p = pkgs; in [ p.libelf p.zlib p.boost ]));
    LIBRARY_PATH = lib.makeLibraryPath
      (let p = pkgs; in [ libgomp p.libelf p.zlib p.glib.out p.boost ]);
    VERBOSE = "1";
    INJECTED_LIBS2E_CXXFLAGS = "-v -Wno-unused-command-line-argument -L${libgomp}/lib";
    INJECTED_CLANG_CC = "${clang_and_llvm}/bin/clang";
    INJECTED_CLANG_CXX = "${clang_and_llvm}/bin/clang++";
    INJECTED_SOCI_SRC = pkgs.fetchFromGitHub {
      owner = "SOCI";
      repo = "soci";
      rev = "438e3549594eb59d84b434c814647648e7c2f10a";
      sha256 = "sha256-HsQyHhW8EP7rK/Pdi1TSXee9yKJsujoDE9QkVdU9WIk=";
    };
    INJECTED_RAPIDJSON_SRC = pkgs.fetchFromGitHub {
      owner = "Tencent";
      repo = "rapidjson";
      rev = "fd3dc29a5c2852df569e1ea81dbde2c412ac5051";
      sha256 = "sha256-r86AJJiJz2mv/2/NgtSObIVgGR3IljL3cbhYzAtrCzQ=";
    };
    INJECTED_GTEST_SRC = pkgs.fetchurl {
      url =
        "https://github.com/google/googletest/archive/release-1.11.0.tar.gz";
      sha256 = "sha256-tIcL8SH/d5W6INILzdhie44Ijy0dqymaAxwQNO3ck9U=";
    };
    LLVM_BUILD = "${s2e-llvm}";
  };
in { inherit s2e-src s2e-llvm s2e-lib libgomp; }
