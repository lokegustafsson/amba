{ pkgs, lib, repositories, makeIncludePath }:
let
  qemu-src = let
    modules = (builtins.map (set: {
      inherit (set) path;
      src = pkgs.fetchFromGitLab {
        owner = if set ? owner then set.owner else "qemu-project";
        inherit (set) repo rev sha256;
      };
    }) [
      {
        repo = "seabios";
        rev = "f9626ccb91e771f990fbb2da92e427a399d7d918";
        sha256 = "sha256-gGGnbfjbQBqWTeiDBIg53n8a7tCQp6GRo0zw5Aaixx4=";
        path = "roms/seabios";
      }
      {
        repo = "SLOF";
        rev = "7d37babcfa48a6eb08e726a8d13b745cb2eebe1c";
        sha256 = "sha256-tTIUlJKIZpZagND3LZydOxj8AEeaEfsPDTc12U6J82c=";
        path = "roms/SLOF";
      }
      {
        repo = "ipxe";
        rev = "0600d3ae94f93efd10fc6b3c7420a9557a3a1670";
        sha256 = "sha256-foWF+GJe7g8W+QpUH/QRQbi0lQhBzzbrZjzDt4Hg9iI=";
        path = "roms/ipxe";
      }
      {
        repo = "openbios";
        rev = "8fe6f5f96f6ca39f1f62200be7fa130e929f13f2";
        sha256 = "sha256-ztEg8X5FNm4AI0MSlPWA0TVX6GRwfafrDkTzGGsi0eY=";
        path = "roms/openbios";
      }
      {
        repo = "openhackware";
        rev = "c559da7c8eec5e45ef1f67978827af6f0b9546f5";
        sha256 = "sha256-wSSI5VaI+0Ww+r8AFp2SCS2sO33NfBx2yVlIxtCUZJk=";
        path = "roms/openhackware";
      }
      {
        repo = "qemu-palcode";
        rev = "f3c7e44c70254975df2a00af39701eafbac4d471";
        sha256 = "sha256-ldOKUPkwrHE4y1oB2GltChSTzgjnliPHqB6qIb+xM+w=";
        path = "roms/qemu-palcode";
      }
      {
        repo = "sgabios";
        rev = "cbaee52287e5f32373181cff50a00b6c4ac9015a";
        sha256 = "sha256-TbKnPytlNd1MovBmAepV4fkB3oYXDB7yOU4uVS/aXpU=";
        path = "roms/sgabios";
      }
      {
        repo = "dtc";
        rev = "e54388015af1fb4bf04d0bca99caba1074d9cc42";
        sha256 = "sha256-vKExFpZW4LVnbKRABl+ZyPG3+VlVFKTdCTZ7gZ6hRf0=";
        path = "dtc";
      }
      {
        repo = "u-boot";
        rev = "d85ca029f257b53a96da6c2fb421e78a003a9943";
        sha256 = "sha256-VLD3OHVeAt67ACnshYyPx8VkiJT9yVROFFRh4n0W5/E=";
        path = "roms/u-boot";
      }
      {
        repo = "skiboot";
        rev = "e0ee24c27a172bcf482f6f2bc905e6211c134bcc";
        sha256 = "sha256-kFH0NIxJczYuw+eV+fTiCjylh8o9lLgS44N+InWRKt4=";
        path = "roms/skiboot";
      }
      {
        repo = "QemuMacDrivers";
        rev = "d4e7d7ac663fcb55f1b93575445fcbca372f17a7";
        sha256 = "sha256-yqkLos9ObAC/GWR/i3z7LEKvEqSA4Pupbqoh/lqWOhU=";
        path = "roms/QemuMacDrivers";
      }
      {
        repo = "keycodemapdb";
        rev = "6b3d716e2b6472eb7189d3220552280ef3d832ce";
        sha256 = "sha256-0deq+Ji6j9JXzRrNYg1hgfPNaz3DxeN1MNA8iXToQ0k=";
        path = "ui/keycodemapdb";
      }
      {
        repo = "capstone";
        rev = "22ead3e0bfdb87516656453336160e0a37b066bf";
        sha256 = "sha256-XhS7z4KehtC0qXPY6tXZx9Hw98WfrRH1ld05fIgibrE=";
        path = "capstone";
      }
      {
        repo = "seabios-hppa";
        rev = "1ef99a01572c2581c30e16e6fe69e9ea2ef92ce0";
        sha256 = "sha256-tdZ5yyW60K68inTq06hEiiZWspUVApCyXwBHrCXw4pE=";
        path = "roms/seabios-hppa";
      }
      {
        repo = "u-boot-sam460ex";
        rev = "60b3916f33e617a815973c5a6df77055b2e3a588";
        sha256 = "sha256-ntIHx/iwKPxjCzJZI5rFGhQMlfGPODOllCesnXLNWO4=";
        path = "roms/u-boot-sam460ex";
      }
    ]);
    rsyncSubmodules = builtins.concatStringsSep "\n"
      (builtins.map (set: "rsync -a ${set.src}/* $out/${set.path}") modules);
  in pkgs.stdenvNoCC.mkDerivation {
    name = "qemu-src";
    phases = [ "installPhase" ];
    installPhase = ''
      mkdir -p $out
      rsync -a ${repositories.qemu}/* $out/
      chmod -R +w $out/*
      ${rsyncSubmodules}
    '';
    buildInputs = [ pkgs.rsync ];

    meta = {
      homepage = "https://github.com/S2E/qemu";
      description = "Source code";
      license = lib.licenses.gpl2Plus;
    };
  };

  s2e-qemu = pkgs.stdenv.mkDerivation {
    pname = "qemu";
    version = "3.0.0-se";
    src = qemu-src;
    configureFlags = [
      "--target-list=i386-softmmu,x86_64-softmmu"
      "--disable-smartcard"
      "--disable-virtfs"
      "--disable-xen"
      "--disable-bluez"
      "--disable-vde"
      "--disable-libiscsi"
      "--disable-docs"
      "--disable-spice"
      "--python=${pkgs.python3Minimal}/bin/python"
    ];
    enableParallelBuilding = true;
    buildInputs = let p = pkgs; in [ p.pkg-config p.glib.dev p.pixman ];

    CFLAGS = "-march=haswell -fno-omit-frame-pointer";
    CXXFLAGS = "-march=haswell -fno-omit-frame-pointer";

    CPATH = (makeIncludePath [ pkgs.zlib.dev pkgs.libpng.dev ]);
    LIBRARY_PATH = lib.makeLibraryPath [ pkgs.zlib pkgs.libpng ];

    meta = {
      homepage = "https://github.com/S2E/qemu";
      description = "Executable";
      license = lib.licenses.gpl2Plus;
    };
  };
in { inherit qemu-src s2e-qemu; }
