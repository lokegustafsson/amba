{ lib, pkgs, amba }:
let
  mkDemo = { name, makeWithMusl ? true
    , filesToCopy ? [ "${name}.c" "${name}" "${name}.recipe.json" ] }:
    let
      thePackage = pkgs.stdenv.mkDerivation {
        inherit name;
        src = ../demos;
        nativeBuildInputs = if makeWithMusl then [ pkgs.musl ] else [ ];
        buildPhase = "make ${name}";
        dontBuild = !makeWithMusl;
        installPhase = ''
          mkdir -p $out/
          cp ${lib.strings.escapeShellArgs filesToCopy} $out/
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
  { name = "hello"; }
  { name = "control-flow"; }
  { name = "state-splitter"; }
  { name = "backdoor"; }
  { name = "demo1"; }
  { name = "demo2"; }
  {
    name = "grep";
    makeWithMusl = false;
    filesToCopy = [ "grep.recipe.json" ];
  }
  {
    name = "ugrep";
    makeWithMusl = false;
    filesToCopy = [ "${pkgs.pkgsStatic.ugrep}/bin/ugrep" "ugrep.recipe.json" ];
  }
])
