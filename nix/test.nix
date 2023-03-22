{ lib, pkgs, amba }:
let
  hello = pkgs.stdenv.mkDerivation {
    name = "hello";
    src = ../demos;
    nativeBuildInputs = [ pkgs.musl ];
    buildPhase = "make hello";
    installPhase = ''
      mkdir -p $out/bin
      cp hello $out/bin
    '';
  };
  test-amba = pkgs.writeShellApplication {
    name = "test-amba";
    text = ''
      # Amba skips unnecessary download internally
      ${amba.amba}/bin/amba init --download
      # Run musl hello world
      ${amba.amba}/bin/amba run ${hello}/bin/hello
    '';
  };
in { inherit test-amba; }
