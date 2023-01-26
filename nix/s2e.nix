{ pkgs, lib }:
let
  # REQUIRES A LOT OF IMPURE NETWORK ACCESS
  s2e-env = pkgs.python3Packages.buildPythonPackage {
    name = "s2e-env";
    src = pkgs.fetchFromGitHub {
      owner = "s2e";
      repo = "s2e-env";
      rev = "98d68b694b18ed24760e67caa07885b57bba9ca8";
      sha256 = "sha256-zV0Uk5iu3H7EWXpmkGrJz2gs2nlSgLPibg8n2i0Ho4I=";
    };
    #doCheck = false;
    #propagatedBuildInputs = let p = pkgs;
    #in [ p.gmpxx p.boost pyyaml plastex six ];
    #nativeBuildInputs = let p = pkgs; in [ p.git p.automake p.autoconf ];
    nativeBuildInputs = let p = pkgs; in [ p.git ];
  };

  # From [https://github.com/S2E/manifest/blob/master/default.xml]
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
  s2e-src = pkgs.stdenv.mkDerivation {
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
      builtins.mapAttrs (url: sha256: pkgs.fetchurl { inherit url sha256; }) {
        "https://www.lua.org/ftp/lua-5.3.4.tar.gz" =
          "sha256-9oGqUYIzvEB+I6zw9Yh8iE8XQ28ADUU7JJGp8RpSQAw=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz" =
          "sha256-YVgiFdr6+3tXbqMMwTa+ksh3uh8cMd2703LW1lYi/vU=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//llvm-14.0.0.src.tar.xz" =
          "sha256-TfftULi3AXuQ3CIgL2tZ6QBqKalWgjjGryjfnASce5s=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//clang-14.0.0.src.tar.xz" =
          "sha256-9df/uG7Vf5fXxHHVQsTlaF20t1+4F8TD8Ce/pJ5WG5s=";
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0//compiler-rt-14.0.0.src.tar.xz" =
          "sha256-J6t/z7IdEICTwL52ap7V/hjATk90+TYGlxGjEsiuA3c=";
      };
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
  s2e-lib = pkgs.stdenv.mkDerivation {
    name = "s2e-lib";
    src = s2e-src;
    dontConfigure = true;
    buildPhase = ''
      mkdir -p $out
      S2E_PREFIX=$out make -f $src/Makefile install
    '';
    buildInputs = [ fake-curl pkgs.cmake ];
  };
in { inherit s2e-src s2e-lib; }
