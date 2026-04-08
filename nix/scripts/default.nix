{ pkgs, ... }:
{
  release = pkgs.writeShellScriptBin "release" (builtins.readFile ./release.sh);
}
