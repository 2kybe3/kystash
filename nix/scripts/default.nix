{ pkgs }:
{
  upgrade = pkgs.writeShellApplication {
    name = "upgrade";

    runtimeInputs = with pkgs; [
      cargo-edit
      gnused
    ];

    text = builtins.readFile ./upgrade.sh;
  };
}
