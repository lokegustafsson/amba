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
    APPEND_CFLAGS = "-static";
  };
  control-flow = pkgs.stdenv.mkDerivation {
    name = "control-flow";
    src = ../demos;
    nativeBuildInputs = [ pkgs.musl ];
    buildPhase = "make control-flow";
    installPhase = ''
      mkdir -p $out/
      cp control-flow control-flow.recipe.json $out/
    '';
    APPEND_CFLAGS = "-static";
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
in { inherit control-flow test-amba; }
