{
  nixConfig.extra-substituters = [ "https://attic.kybe.xyz/main" ];
  nixConfig.extra-trusted-public-keys = [
    "main:cb7V485kGP0lG7LtQ/suOgKOgtVxNXrnD6i5yCtnaMQ="
  ];

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    crane.url = "github:ipetkov/crane";
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
        src = pkgs.lib.cleanSourceWith {
          filter =
            path: type:
            ((craneLib.filterCargoSources path type) || (builtins.match ".*/assets/.*" path != null));
          src = ./.;
          name = "source";
        };

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
          inherit kystash;
          default = kystash;
        };
        apps.default = {
          type = "app";
          program = "${kystash}/bin/kystash";
          meta.description = "A simple image/file sharing server/client";
        };
        checks = {
          inherit kystash;

          kystash-tests = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          kystash-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "-- --deny warnings";
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
