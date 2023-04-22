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
  control-flow = pkgs.stdenv.mkDerivation {
    name = "control-flow";
    src = ../demos;
    nativeBuildInputs = [ pkgs.musl ];
    buildPhase = "make control-flow";
    installPhase = ''
      mkdir -p $out/
      cp control-flow control-flow.recipe.json $out/
    '';
  };
  state-splitter = pkgs.stdenv.mkDerivation {
    name = "state-splitter";
    src = ../demos;
    nativeBuildInputs = [ pkgs.musl ];
    buildPhase = "make state-splitter";
    installPhase = ''
      mkdir -p $out/
      cp state-splitter state-splitter.recipe.json $out/
    '';
  };
  test-amba-hello = pkgs.writeShellApplication {
    name = "test-amba-hello";
    text = ''
      export RUST_BACKTRACE=full
      # Amba skips unnecessary download internally
      ${amba.amba}/bin/amba init --download
      # Run musl hello world
      time ${amba.amba}/bin/amba run ${hello}/hello.recipe.json --no-gui
    '';
  };
  test-amba-control-flow = pkgs.writeShellApplication {
    name = "test-amba-control-flow";
    text = ''
      export RUST_BACKTRACE=full
      # Amba skips unnecessary download internally
      ${amba.amba}/bin/amba init --download
      # Run musl control flow
      time ${amba.amba}/bin/amba run ${control-flow}/control-flow.recipe.json --no-gui
    '';
  };
  test-amba-state-splitter = pkgs.writeShellApplication {
    name = "test-amba-state-splitter";
    text = ''
      export RUST_BACKTRACE=full
      # Amba skips unnecessary download internally
      ${amba.amba}/bin/amba init --download
      # Run musl state-splitter
      time ${amba.amba}/bin/amba run ${state-splitter}/state-splitter.recipe.json --no-gui
    '';
  };
in { inherit hello control-flow state-splitter test-amba-hello test-amba-control-flow test-amba-state-splitter; }
