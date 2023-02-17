{ pkgs, lib }:
let
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
    {
      repo = "pyelftools";
      rev = "a007128bb89b66e08a03fce7bfdafeb01be21f0b";
      sha256 = "sha256-LIA1Pghs7LKQs4GdB2xQqaow0ertUQWWHZrXBjUq7O4=";
    }
  ]);

  core = import ./core.nix { inherit pkgs lib repositories; };
  env = import ./env.nix { inherit pkgs lib repositories; };

  fake-wget = let
    content =
      (builtins.mapAttrs (url: sha256: pkgs.fetchurl { inherit url sha256; }) {
        "https://cdimage.debian.org/mirror/cdimage/archive/11.3.0/i386/iso-cd/debian-11.3.0-i386-netinst.iso" =
          "sha256-GWdwWC6fT000z0EszfgRiyR+kCdIQwVdYnuX7WWixco=";
      });
    dict = lib.strings.concatStringsSep ","
      (lib.attrsets.mapAttrsToList (url: drv: "'${url}': '${drv}'") content);
  in pkgs.writeScriptBin "wget" ''
    #!${pkgs.python3Minimal}/bin/python3
    import sys
    import shutil
    argv = tuple(sys.argv)
    if len(argv) == 5 and argv[1:3] == ("--no-use-server-timestamps", "-O"):
      target = argv[3]
      url = argv[4]
      resolved = {${dict}}[url]
      shutil.copyfile(resolved, target)
    else:
      raise Exception("fake wget given", argv)
  '';

  guest-images = pkgs.stdenvNoCC.mkDerivation {
    name = "guest-images";
    src = repositories.guest-images;
    patches = [ ../patches/guest-images/makefile.patch ];
    buildPhase = ''
      patchShebangs ./scripts/*.py
      make -j linux
    '';
    S2E_INSTALL_ROOT = core.s2e;
    S2E_LINUX_KERNELS_ROOT = repositories.s2e-linux-kernel;

    requiredSystemFeatures = [ "kvm" ];
    buildInputs = [
      (pkgs.python3.withPackages (py-pkgs: with py-pkgs; [ jinja2 ]))
      fake-wget
      pkgs.jq
      pkgs.procps
      pkgs.p7zip
      pkgs.cdrkit
    ];
  };
in {
  inherit (core) s2e-src s2e-llvm s2e-lib s2e-tools s2e-guest-tools s2e libgomp;
  inherit (env) s2e-env;
  inherit guest-images;
}
