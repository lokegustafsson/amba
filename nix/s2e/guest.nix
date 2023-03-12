{ pkgs, lib, repositories, core }:
let
  mkGuestKernel = { name, arch, VERSION, pkgsCross }:
    pkgs.stdenvNoCC.mkDerivation {
      name = "${name}-s2e-${arch}";
      src = "${repositories.s2e-linux-kernel}/${name}";
      configurePhase = ''
        mv config-${arch} .config
      '';
      buildPhase = ''
        fakeroot make -j $NIX_BUILD_CORES bindeb-pkg
      '';
      installPhase = ''
        mkdir -p $out/${name}
        mv ../*.deb $out
        mv vmlinux $out/${name}
      '';
      C_INCLUDE_PATH = [ "${repositories.s2e-linux-kernel}/include" ];
      inherit VERSION;
      LOCALVERSION = "-s2e";

      depsBuildBuild = [ pkgs.stdenv.cc ];
      nativeBuildInputs = let p = pkgs;
      in [
        (p.writeScriptBin "hostname" ''
          echo 'linux-build-${arch}'
        '')
        p.bc
        p.dpkg
        p.fakeroot
        p.perl
        pkgsCross.stdenv.cc
      ];
    };
  guest-kernel64 = mkGuestKernel {
    name = "linux-4.9.3";
    arch = "x86_64";
    VERSION = "4.9.3";
    pkgsCross = pkgs.pkgsCross.gnu64;
  };
  guest-kernel32 = mkGuestKernel {
    name = "linux-4.9.3";
    arch = "i386";
    VERSION = "4.9.3";
    pkgsCross = pkgs.pkgsCross.gnu32;
  };

  fake-wget = let
    content =
      (builtins.mapAttrs (url: sha256: pkgs.fetchurl { inherit url sha256; }) {
        "https://cdimage.debian.org/mirror/cdimage/archive/11.3.0/i386/iso-cd/debian-11.3.0-i386-netinst.iso" =
          "sha256-GWdwWC6fT000z0EszfgRiyR+kCdIQwVdYnuX7WWixco=";
        "https://cdimage.debian.org/mirror/cdimage/archive/11.3.0/amd64/iso-cd/debian-11.3.0-amd64-netinst.iso" =
          "sha256-eJKYHh2iFuefs6FTbOXrqxV6/dIASP5FjyrjT7wmwZs=";
        "https://old-releases.ubuntu.com/releases/jammy/ubuntu-22.04-live-server-amd64.iso" =
          "sha256-hK6veCPIxhuqCuhi0KBrA0CTlIAAALMjWFSms460hW8=";
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
  copy_nix_built_linux_kernel =
    pkgs.writeScriptBin "copy_nix_built_linux_kernel" ''
      #!${pkgs.bash}/bin/bash
      echo Copy kernel called with
      echo $1
      echo $2
      mkdir -p "$2"
      if   [ "$1" = "linux-4.9.3-x86_64" ]; then
        ${pkgs.rsync}/bin/rsync -a ${guest-kernel64}/* "$2"
      elif [ "$1" = "linux-4.9.3-i386" ]; then
        ${pkgs.rsync}/bin/rsync -a ${guest-kernel32}/* "$2"
      fi
    '';
  guest-images-src = pkgs.stdenvNoCC.mkDerivation {
    name = "guest-images-src";
    src = repositories.guest-images;
    phases = [ "unpackPhase" "patchPhase" "installPhase" "fixupPhase" ];
    patches = [
      ../patches/guest-images/makefile.patch
      ../patches/guest-images/makefile.linux.patch
    ];
    installPhase = ''
      mkdir -p $out
      cp -r * $out/
    '';
    fixupPhase = ''
      patchShebangs $out/scripts/*.py
      patchShebangs $out/qemu.wrapper
    '';
    buildInputs = [ pkgs.bash ];
  };

  # TODO: Makefile targets "debian-11.3-i386" and "debian-11.3-x86_64"
  build-guest-images = pkgs.writeShellApplication {
    name = "build-guest-images";
    runtimeInputs = let p = pkgs;
    in [
      (pkgs.python3.withPackages (py-pkgs: with py-pkgs; [ jinja2 ]))
      copy_nix_built_linux_kernel
      fake-wget
      p.bash
      p.cdrkit
      p.cloud-utils
      p.coreutils
      p.findutils
      p.fuse
      p.gnumake
      p.gnutar
      p.jq
      p.libguestfs-with-appliance
      p.ncurses
      p.p7zip
      p.procps
      p.rsync
    ];
    text = let
      SRC = guest-images-src;
      S2E_INSTALL_ROOT = core.s2e;
      S2E_LINUX_KERNELS_ROOT = repositories.s2e-linux-kernel;
    in ''
      BUILDDIR=$1
      OUTDIR=$2

      if [[ -z "$BUILDDIR" ]]; then
          echo "error: MISSING BUILDDIR (first argument)"
          exit 1
      fi
      if [[ -z "$OUTDIR" ]]; then
          echo "error: missing OUTDIR (second argument)"
          exit 1
      fi
      if test -d "$BUILDDIR"; then
          echo "error: BUILDDIR=$BUILDDIR already exists"
          exit 1
      fi
      if test -d "$OUTDIR"; then
          echo "error: OUTDIR=$OUTDIR already exists"
          exit 1
      fi

      export S2E_INSTALL_ROOT=${S2E_INSTALL_ROOT}
      export S2E_LINUX_KERNELS_ROOT=${S2E_LINUX_KERNELS_ROOT}

      mkdir -p "$BUILDDIR"
      cp -r ${SRC} "$BUILDDIR"/
      chmod -R +w "$BUILDDIR"/*
      time make -C "$BUILDDIR"/"$(basename ${SRC})" ubuntu-22.04-x86_64 OUTDIR="$OUTDIR"
    '';
  };
in {
  inherit guest-kernel32 guest-kernel64 guest-images-src build-guest-images;
}
