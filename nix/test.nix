{ lib, pkgs, amba }:
let
  hello = pkgs.stdenv.mkDerivation {
    name = "hello";
    src = ../demos;
    nativeBuildInputs = [ pkgs.musl ];
    buildPhase = "make hello";
    installPhase = ''
      mkdir -p $out/
      cp hello hello.recipe.json $out/
    '';
  };
  test-amba = pkgs.writeShellApplication {
    name = "test-amba";
    text = ''
      export RUST_BACKTRACE=full
      # Amba skips unnecessary download internally
      ${amba.amba}/bin/amba init --download
      # Run musl hello world
      ${amba.amba}/bin/amba run ${hello}/hello.recipe.json
    '';
  };
in { inherit test-amba; }
