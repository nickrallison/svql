{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils}:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [ 
            ## Project
            clang 
            gcc 
            cmake
            ninja
            pkg-config
            asio

            ## Rust
            cargo 
            rustc 
            rustfmt 
            pre-commit 
            rustPackages.clippy 

            ## Yosys
            gtkwave 
            llvmPackages.bintools 
            bison 
            flex 
            libffi 
            tcl 
            tk 
            readline 
            python3 
            zlib 
            git 
            gtest 
            abc-verifier 
            verilog 
            boost 
            python3Packages.boost
          ];
        };
      }
    );
}