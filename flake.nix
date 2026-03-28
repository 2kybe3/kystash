{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    crane.url = "github:kybe236/crane";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      crane,
      nixpkgs,
      treefmt-nix,
      flake-utils,
      advisory-db,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        treefmtEval = treefmt-nix.lib.evalModule pkgs ./nix/treefmt.nix;
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        kystash = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        packages = rec {
          kystash = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
          };
          default = kystash;
        };
        apps.default = {
          type = "app";
          program = "${kystash}/bin/kystash";
          meta.description = "A simple image/file sharing server/client";
        };
        checks = {
          inherit kystash;

          kystash-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          kystash-audit = craneLib.cargoAudit (
            commonArgs
            // {
              inherit src advisory-db;
            }
          );

          formatting = treefmtEval.config.build.check self;
        };
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          KYSTASH_CLIENT_PATH = "./test-client";
          KYSTASH_SERVER_PATH = "./test-server";
        };
        formatter = treefmtEval.config.build.wrapper;
      }
    );
}
