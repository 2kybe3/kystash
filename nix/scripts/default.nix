{ pkgs, ... }:
{
  release = pkgs.writeShellApplication {
    name = "release";

    inheritPath = false;
    runtimeInputs = with pkgs; [
      gnused
      diffutils
    ];

    text = builtins.readFile ./release.sh;
  };
  build-all = pkgs.writeShellApplication {
    name = "build-all";

    inheritPath = false;
    runtimeInputs = with pkgs; [
      cargo-cross
      coreutils
      gnugrep
      docker # For cross
      rustup
      curl
      gawk
      jq
    ];

    text = builtins.readFile ./build-all.sh;
  };
}
