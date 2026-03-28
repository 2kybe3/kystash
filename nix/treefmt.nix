{ pkgs, ... }:
{
  projectRootFile = "flake.nix";
  programs = {
    taplo.enable = true;
    typos.enable = true;
    nixfmt.enable = true;
    yamlfmt.enable = true;
    rustfmt.enable = true;
  };
  settings = {
    excludes = [
      "target/*"
      "result/*"
      ".git/*"
    ];
    formatter."copyright" = {
      command = "${pkgs.bash}/bin/bash";
      options = [
        "-euc"
        (builtins.readFile ./copyright.sh)
        "--"
      ];
      includes = [ "*.rs" ];
    };
  };
}
