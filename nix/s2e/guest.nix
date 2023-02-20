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

  guest-images = pkgs.stdenvNoCC.mkDerivation {
    name = "guest-images";
    src = repositories.guest-images;
    patches = [
      ../patches/guest-images/makefile.patch
      ../patches/guest-images/makefile.linux.patch
      ../patches/guest-images/qemu.wrapper.patch
    ];
    configurePhase = ''
      patchShebangs ./scripts/*.py
    '';
    makeFlags = [
      #"debian-11.3-i386"
      #"debian-11.3-x86_64"
      "ubuntu-22.04-x86_64"
      "OUTDIR=$out"
    ];
    installPhase = ''
      mkdir -p $out
      cp * $out/
    '';

    #enableParallelBuilding = true;
    S2E_INSTALL_ROOT = core.s2e;
    S2E_LINUX_KERNELS_ROOT = repositories.s2e-linux-kernel;
    GRAPHICS = " ";

    requiredSystemFeatures = [ "kvm" ];
    buildInputs = let p = pkgs;
    in [
      (pkgs.python3.withPackages (py-pkgs: with py-pkgs; [ jinja2 ]))
      copy_nix_built_linux_kernel
      fake-wget
      p.cdrkit
      p.cloud-utils
      p.jq
      p.ncurses
      p.p7zip
      p.procps
      p.libguestfs-with-appliance
    ];
  };
  guest-images-shell = pkgs.mkShell {
    SRC = pkgs.stdenvNoCC.mkDerivation {
      name = "guest-images-src";
      src = repositories.guest-images;
      patches = [
        ../patches/guest-images/makefile.patch
        ../patches/guest-images/makefile.linux.patch
        ../patches/guest-images/qemu.wrapper.patch
      ];
      installPhase = ''
        mkdir -p $out
        cp -r * $out/
      '';
      phases = [ "unpackPhase" "patchPhase" "fixupPhase" "installPhase" ];
      buildInputs = [ pkgs.bash ];
    };
    S2E_INSTALL_ROOT = core.s2e;
    S2E_LINUX_KERNELS_ROOT = repositories.s2e-linux-kernel;
    GRAPHICS = " ";

    requiredSystemFeatures = [ "kvm" ];
    buildInputs = let p = pkgs;
    in [
      (pkgs.python3.withPackages (py-pkgs: with py-pkgs; [ jinja2 ]))
      copy_nix_built_linux_kernel
      fake-wget
      p.cdrkit
      p.cloud-utils
      p.jq
      p.libguestfs-with-appliance
      p.ncurses
      p.p7zip
      p.procps
    ];
  };
in { inherit guest-kernel32 guest-kernel64 guest-images guest-images-shell; }
