{
    description = "A flake for chisel, verilator, and OpenSTA development";
    inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

    outputs = { self, nixpkgs }: {
        
        
        devShell = 
            let 
                circt_rev = "50a7139fbd1acd4a3d4cfa695e694c529dd26f3a";
                opensta_rev = "20925bb00965c1199c45aca0318c2baeb4042c5a";
                opensta_sha256 = "sha256-gWAN+d6ioxQtxtgeq3vR+Zgq3nYRyn/u104L/xqumuY=";

                env_exports = ''
                    export FIRTOOL_REV=${circt_rev}
                    export FIRTOOL_VER=1.44.0
                '';
            in 
        {
            aarch64-darwin = with nixpkgs.legacyPackages.aarch64-darwin; mkShellNoCC {
                packages = with pkgs; [
                    # Chisel
                    (let
                        circtpkgs = import (builtins.fetchTree { 
                        type = "github"; 
                        owner = "nixos"; 
                        repo = "nixpkgs"; 
                        rev = circt_rev; }) 
                        { inherit (pkgs) system; };
                    in circtpkgs.circt)
                
                    # Scala 
                    sbt
                    scala-cli
                    scalafmt

                    # Verilator
                    verilator
                    ninja
                    cmake
                    gtkwave

                    # Synthesis
                    yosys

                    # Formal Verification
                    # sby
                    yices
                    z3

                    # LaTeX
                    texliveFull

                    # Other
                    python3
                    nodePackages_latest.wavedrom-cli


                    # OpenSTA
                    (pkgs.stdenv.mkDerivation {
                        pname = "opensta";
                        version = "2024-09-09";
                        src =  pkgs.fetchFromGitHub {
                            owner = "The-OpenROAD-Project";
                            repo = "OpenSTA";
                            rev = opensta_rev;
                            sha256 = opensta_sha256;
                        };
                        buildInputs = [ 
                            pkgs.cmake 
                            pkgs.gcc 
                            pkgs.tcl 

                            pkgs.bison
                            pkgs.flex
                            pkgs.eigen
                            pkgs.swig4
                            pkgs.cudd
                            pkgs.tclreadline
                            pkgs.zlib
                            pkgs.tcllib
                        ];
                        nativeBuildInputs = [ 
                            pkgs.cmake 
                            pkgs.tcl 
                        ];
                        cmakeFlags = [
                            "-DTCL_LIBRARY=${pkgs.tcl}/lib/libtcl8.6.dylib"
                            "-DCMAKE_INSTALL_PREFIX=${placeholder "out"}"
                        ];
                        installPhase = ''
                            mkdir -p ${placeholder "out"}
                            make install
                        '';
                    })
                    yosys
                    gtkwave
                    verilog

                    # LaTeX
                    texliveFull

                    # Other
                    python3
                    nodePackages_latest.wavedrom-cli
                ];
                shellHook = env_exports + ''
                    export CXX=/usr/bin/c++
                    export CC=/usr/bin/cc
                '';
            };

            x86_64-linux = with nixpkgs.legacyPackages.x86_64-linux; mkShellNoCC {
                packages = with pkgs; [
                    # Chisel
                    (let
                        circtpkgs = import (builtins.fetchTree {
                        type = "github";
                        owner = "nixos";
                        repo = "nixpkgs";
                        rev = circt_rev; })
                        { inherit (pkgs) system; };
                    in circtpkgs.circt)

                    # Scala
                    sbt
                    scala-cli

                    # Verilator
                    # Verilator has issues with nix (VUart__pch.h.slow: No such file or director)
                    # Installing from derivation needs to be done with clang or it will fail at runtime
                    # a workaround is to install verilator via the system package manager
                    (let
                       systemcClang = pkgs.systemc.override { stdenv = pkgs.clangStdenv; };
                     in
                     pkgs.clangStdenv.mkDerivation rec {
                       pname = "verilator";
                       version = "5.006";

                       VERILATOR_SRC_VERSION = "v${version}";

                       src = pkgs.fetchFromGitHub {
                         owner = pname;
                         repo = pname;
                         rev = "v${version}";
                         hash = "sha256-YgK60fAYG5575uiWmbCODqNZMbRfFdOVcJXz5h5TLuE=";
                       };

                       enableParallelBuilding = true;
                       buildInputs = [
                         pkgs.perl
                         pkgs.python3
                         systemcClang
                       ];
                       nativeBuildInputs = [
                         pkgs.makeWrapper
                         pkgs.flex
                         pkgs.bison
                         pkgs.autoconf
                         pkgs.help2man
                         pkgs.git
                       ];
                       nativeCheckInputs = [
                         pkgs.which
                         pkgs.numactl
                         pkgs.coreutils
                       ];

                       doCheck = pkgs.stdenv.hostPlatform.isLinux;
                       checkTarget = "test";

                       preConfigure = "autoconf";

                       postPatch = ''
                         patchShebangs bin/* src/* nodist/* docs/bin/* examples/xml_py/* \
                         test_regress/{driver.pl,t/*.{pl,pf}} \
                         ci/* ci/docker/run/* ci/docker/run/hooks/* ci/docker/buildenv/build.sh
                         sed -i 's|/bin/echo|${pkgs.coreutils}/bin/echo|' bin/verilator
                       '';

                       env = {
                         SYSTEMC_INCLUDE = "${pkgs.lib.getDev systemcClang}/include";
                         SYSTEMC_LIBDIR = "${pkgs.lib.getLib systemcClang}/lib";
                       };

                       meta = with pkgs.lib; {
                         description = "Fast and robust (System)Verilog simulator/compiler and linter";
                         homepage = "https://www.veripool.org/verilator";
                         license = with licenses; [ lgpl3Only artistic2 ];
                         platforms = platforms.unix;
                         maintainers = with maintainers; [ thoughtpolice amiloradovsky ];
                       };
                     })
                    # The simplest way to use verilator is to install from package manager
                    ninja
                    cmake

                    # OpenSTA
                    (pkgs.stdenv.mkDerivation {
                        pname = "opensta";
                        version = "2024-09-09";
                        src =  pkgs.fetchFromGitHub {
                            owner = "The-OpenROAD-Project";
                            repo = "OpenSTA";
                            rev = opensta_rev;
                            sha256 = opensta_sha256;
                        };
                        buildInputs = [
                            pkgs.cmake
                            pkgs.gcc
                            pkgs.tcl

                            pkgs.bison
                            pkgs.flex
                            pkgs.eigen
                            pkgs.swig4
                            pkgs.cudd
                            pkgs.tclreadline
                            pkgs.zlib
                            pkgs.tcllib
                        ];
                        nativeBuildInputs = [
                            pkgs.cmake
                            pkgs.tcl
                        ];
                        cmakeFlags = [
                            "-DTCL_LIBRARY=${pkgs.tcl}/lib/libtcl8.6.so"
                            "-DCMAKE_INSTALL_PREFIX=${placeholder "out"}"
                        ];
                        installPhase = ''
                            mkdir -p ${placeholder "out"}
                            make install
                        '';
                    })
                    yosys
                    gtkwave
                    verilog

                     # LaTeX
                    texliveFull

                    # Other
                    gcc
                    clang
                    python3
                    nodePackages_latest.wavedrom-cli
                ];
                shellHook = env_exports + ''
                    export CHISEL_FIRTOOL_PATH="${pkgs.circt}/bin"
                    export CXX=clang++
                    export CC=clang
                '';
            };
        };
    };
}

