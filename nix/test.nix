{ lib, pkgs, amba }:
let
  mkDemo = name:
    let
      thePackage = pkgs.stdenv.mkDerivation {
        inherit name;
        src = ../demos;
        nativeBuildInputs = [ pkgs.musl ];
        buildPhase = "make ${name}";
        installPhase = ''
          mkdir -p $out/
          cp ${name}.c ${name} ${name}.recipe.json $out/
        '';
      };
    in [
      {
        name = name;
        value = thePackage;
      }
      {
        name = "test-amba-${name}";
        value = pkgs.writeShellApplication {
          name = "test-amba-${name}";
          text = ''
            export RUST_BACKTRACE=full
            # Amba skips unnecessary download internally
            ${amba.amba}/bin/amba init --download
            # Run musl ${name}
            time ${amba.amba}/bin/amba run ${thePackage}/${name}.recipe.json --no-gui
          '';
        };
      }
    ];
in builtins.listToAttrs (builtins.concatMap mkDemo [
  "hello"
  "control-flow"
  "state-splitter"
  "backdoor"
  "demo1"
  "demo2"
])
