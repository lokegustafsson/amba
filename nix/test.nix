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
        ${amba.rust.packages.amba}/bin/amba init --download
      else
        echo "Amba already setup"
      fi
      ${amba.rust.packages.amba}/bin/amba run ${hello}/bin/hello
    '';
  };
in {
  inherit test-amba;
}
