{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs = {
    self,
    nixpkgs,
    crate2nix,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      kystashRoot = crate2nix.tools.${system}.appliedCargoNix {
        name = "kystash";
        src = ./.;
      };
    in {
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          cargo
          rustc
          clippy
          rustfmt
        ];
      };
      checks = {
        rustnix = kystashRoot.rootCrate.build.override {
          runTests = true;
        };
      };
      packages = rec {
        kystash = kystashRoot.rootCrate.build;
        default = kystash;
      };
      formatter = nixpkgs.legacyPackages.${system}.alejandra;
    });
}
