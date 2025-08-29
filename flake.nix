{
  description = "SVQL and OpenPiton/Ariane development environment with legacy tool support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowInsecure = true;
            permittedInsecurePackages = [
              "python-2.7.18.8"
              "python-2.7.18.8-env"
            ];
          };
        };
        
        # Python 2.7 environment (without packages that don't support it)
        python27Env = pkgs.python27;

        # Python 3 environment with required packages
        python3Env = pkgs.python3.withPackages (ps: with ps; [
          pyyaml
        ]);

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust development tools
            cargo 
            rustc 
            rustfmt 
            pre-commit 
            rustPackages.clippy

            # Python environments
            python27Env
            python3Env
            python314
            
            # FuseSoC and Python packages
            python3Packages.pip
            python3Packages.setuptools
            
            # Hardware design tools
            yosys
            verilator
            gtkwave
            
            # Version control and utilities
            git
            gnumake
            
            # Text processing
            gnused
            gawk
            
            # Optional: Icarus Verilog for simulation
            verilog
            
            # Optional: RISC-V toolchain if needed
            # pkgsCross.riscv64.buildPackages.gcc
          ];

          shellHook = ''
            # Clear conflicting environment variables
            unset VERILATOR_ROOT
            
            echo "SVQL and OpenPiton/Ariane development environment loaded!"
            echo ""
            echo "Available tools:"
            echo "  - Rust: $(rustc --version)"
            echo "  - Cargo: $(cargo --version)"
            echo "  - Python 2.7: $(python2.7 --version)"
            echo "  - Python 3: $(python3 --version)"
            echo "  - Python 3.14: $(python3.14 --version)"
            echo "  - Yosys: $(yosys --version | head -1)"
            echo "  - Verilator: $(verilator --version | head -1)"
            echo ""
            
            # Install FuseSoC in a virtual environment if not already installed
            if ! command -v fusesoc &> /dev/null; then
              echo "Installing FuseSoC in virtual environment..."
              if [ ! -d .venv ]; then
                python3 -m venv .venv
              fi
              source .venv/bin/activate
              pip install fusesoc
              export PATH="$(pwd)/.venv/bin:$PATH"
            fi
            
            echo "  - FuseSoC: $(fusesoc --version 2>/dev/null || echo 'installing...')"
            echo ""
            echo "To use the Python 2.7 preprocessor:"
            echo "  python2.7 piton/tools/bin/pyhp.py <input.pyv>"
            echo ""
            echo "To use FuseSoC:"
            echo "  fusesoc run --target=pickle openpiton::system:0.1"
            echo ""
            
            # Set up FuseSoC library if not already done
            if [ ! -f ~/.config/fusesoc/fusesoc.conf ]; then
              echo "Setting up FuseSoC library..."
              mkdir -p ~/.config/fusesoc
              fusesoc library add hackatdac21 . > /dev/null 2>&1 || true
            fi
          '';

          # Environment variables
          PYTHON2 = "${python27Env}/bin/python2.7";
          PYTHON3 = "${python3Env}/bin/python3";
          
          # Make sure the preprocessor uses Python 2.7
          PYTHONPATH = "${python27Env}/${python27Env.sitePackages}";
        };

        # Rust-focused development shell
        devShells.rust = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust development tools
            cargo 
            rustc 
            rustfmt 
            pre-commit 
            rustPackages.clippy

            # Hardware design tools
            yosys

            # Python for scripts
            python314
          ];
          
          shellHook = ''
            echo "SVQL Rust development environment loaded!"
            echo "  - Rust: $(rustc --version)"
            echo "  - Cargo: $(cargo --version)"
            echo "  - Yosys: $(yosys --version | head -1)"
          '';
        };

        # Alternative shell with just the OpenPiton essentials
        devShells.minimal = pkgs.mkShell {
          buildInputs = with pkgs; [
            python27Env
            yosys
            gnumake
          ];
          
          shellHook = ''
            echo "Minimal OpenPiton environment (Python 2.7 + Yosys)"
          '';
        };
      });
}