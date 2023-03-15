{ pkgs, lib, repositories }:
let
  s2e-env = pkgs.python3Packages.buildPythonPackage {
    name = "s2e-env";
    src = repositories.s2e-env;
    patches = [ ../patches/s2e-env/setup.py.patch ];
    doCheck = false;

    meta = {
      homepage = "https://github.com/S2E/s2e-env";
      description = "A tool to run S2E (not used by amba)";
      license = lib.licenses.gpl2Plus;
    };

    propagatedBuildInputs = let
      p = pkgs.python3Packages;
      unicorn = pkgs.stdenv.mkDerivation rec {
        pname = "unicorn";
        version = "1.0.2-rc3";

        src = pkgs.fetchFromGitHub {
          owner = "unicorn-engine";
          repo = pname;
          rev = version;
          sha256 = "sha256-wgs+STqYWzTXeAER6qnBFIq8r6QX6i1I8xuD5lOKWT0=";
        };

        nativeBuildInputs = [ pkgs.pkgconfig pkgs.cmake pkgs.python3 ];
      };
      pyelftools = p.buildPythonPackage {
        name = "pyelftools";
        src = repositories.pyelftools;
        PYTHONPATH = "${repositories.pyelftools}/test";
      };

      protobuf = p.buildPythonPackage rec {
        pname = "protobuf";
        version = "3.20.1";
        src = p.fetchPypi {
          inherit pname version;
          sha256 = "sha256-rcMVZtAn9F7+P0TutbHzKdpDiRY01hx1pZROm+bdQsk=";
        };
      };
    in [
      pyelftools
      (p.buildPythonPackage rec {
        pname = "pdbparse";
        version = "1.5";
        src = p.fetchPypi {
          inherit pname version;
          sha256 = "sha256-braJoTaM7JA4nOweTWVHnnm70PH/p94TsIEtqRlDcLw=";
        };
        propagatedBuildInputs = [
          (p.buildPythonPackage rec {
            pname = "pefile";
            version = "2019.4.18";
            doCheck = false;
            src = p.fetchPypi {
              inherit pname version;
              sha256 = "sha256-pdboMFxrIQhJtHphdN35xFKyiINAuBd4dLhiumwgdkU=";
            };
            propagatedBuildInputs = [ p.future ];
          })
          (p.buildPythonPackage rec {
            pname = "construct";
            version = "2.9.52";
            doCheck = false;
            src = p.fetchPypi {
              inherit pname version;
              sha256 = "sha256-Xpysve3StvcGWSNS+kR+qQ9sp+n7tZH3R1SId9bYtGc=";
            };
          })
        ];
      })
      p.sh
      p.pygments
      p.patool
      (p.buildPythonPackage rec {
        pname = "alive-progress";
        version = "3.0.1";
        src = p.fetchPypi {
          inherit pname version;
          sha256 = "sha256-MkURQlO2rbSzjyoqGCjt/NnowBL34wpc7xkyynNE60Q=";
        };
        propagatedBuildInputs = [
          (p.buildPythonPackage rec {
            pname = "about-time";
            version = "4.2.1";
            src = p.fetchPypi {
              inherit pname version;
              sha256 = "sha256-alOIYtM85n2ZdCnRSZgxDh2/2my32bv795nEcJhH/s4=";
            };
          })
          p.grapheme
        ];
      })
      (p.buildPythonPackage rec {
        pname = "PyTrie";
        version = "0.4.0";
        doCheck = false;
        src = p.fetchPypi {
          inherit pname version;
          sha256 = "sha256-j0SI9ALTRlmT+2tu+gmGaEntjNp5A7UGR7fQNCuAU3k=";
        };
        propagatedBuildInputs = [ p.sortedcontainers ];
      })
      p.termcolor
      p.distro
      p.jinja2
      p.pyftpdlib
      p.pyyaml
      (p.buildPythonPackage rec {
        pname = "pyunpack";
        version = "0.2.2";
        src = p.fetchPypi {
          inherit pname version;
          sha256 = "sha256-jbjTUOMymtBvoKoAqigpX1ur+MEknOJiksAm6HW6JDY=";
        };
        propagatedBuildInputs = [ p.EasyProcess p.entrypoint2 ];
      })
      protobuf
      p.psutil
      (p.buildPythonPackage rec {
        pname = "pwntools";
        version = "4.3.1";
        doCheck = false;
        src = p.fetchPypi {
          inherit pname version;
          sha256 = "sha256-xGGI5xPEdhey2/PjLRhn+UjTXYL935qdIpSjP0dISoo=";
        };
        propagatedBuildInputs = [
          p.paramiko
          p.capstone
          p.intervaltree
          p.ropgadget
          p.psutil
          p.pysocks
          p.python-dateutil
          p.pyserial
          p.packaging
          p.Mako
          p.pygments
          p.requests
          pyelftools
          (p.buildPythonPackage rec {
            pname = "unicorn";
            version = "1.0.2rc3";
            src = unicorn.src;
            sourceRoot = "source/bindings/python";
            prePatch = ''
              ln -s ${unicorn}/lib/libunicorn.* prebuilt/
            '';
            checkPhase = ''
              runHook preCheck
              mv unicorn unicorn.hidden
              patchShebangs sample_*.py shellcode.py
              sh -e sample_all.sh
              runHook postCheck
            '';
            pythonImportsCheck = [ "unicorn" ];
          })
        ];
      })
      p.immutables
      p.python-magic
      (p.protobuf3-to-dict.overrideAttrs
        (final: prev: { propagatedBuildInputs = [ p.six protobuf ]; }))
      p.mock
    ];
  };
in { inherit s2e-env; }
