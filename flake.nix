{
  description = "oscons";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let supportedSystems = [ "aarch64-linux" "x86_64-linux" ];
    in flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = (import nixpkgs {
          inherit system;
          overlays = [ ];
        }).pkgsCross.i686-embedded;
      in {
        devShell = pkgs.mkShell {
          depsBuildBuild = with pkgs.pkgsBuildBuild; [ qemu ];
          nativeBuildInputs = with pkgs.pkgsBuildHost; [ yasm ];
        };
      });
}
