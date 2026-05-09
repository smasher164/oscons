{
  description = "oscons";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    let supportedSystems = [ "aarch64-linux" "x86_64-linux" "aarch64-darwin" "x86_64-darwin" ];
    in flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = fenix.packages.${system}.latest.withComponents [
          "cargo"
          "clippy"
          "rustfmt"
          "rustc"
          "rust-src"
          "llvm-tools-preview"
        ];
      in {
        devShell = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.lld
            pkgs.binutils
            pkgs.qemu
          ];
        };
      });
}
