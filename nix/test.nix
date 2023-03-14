{ lib, pkgs, amba }:
let
  hello = pkgs.stdenv.mkDerivation {
    name = "hello";
    src = ../demos;
    nativeBuildInputs = with pkgs; [ musl ];
    buildPhase = "make hello";
    installPhase = ''
      mkdir -p $out/bin
      cp hello $out/bin
    '';
  };
  test-amba = pkgs.writeShellApplication {
    name = "test-amba";
    text = ''
      if [[ -v AMBA_DATA_DIR && -d $AMBA_DATA_DIR ]]; then
        echo "Amba already setup"
      else
        ${amba.rust.packages.amba}/bin/amba init --download
      fi
      ${amba.rust.packages.amba}/bin/amba run ${hello}/bin/hello
    '';
  };
in { inherit test-amba; }
