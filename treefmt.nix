{ pkgs, ... }:
{
  projectRootFile = "flake.nix";
  programs = {
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
        (builtins.readFile ./fmt.sh)
        "--"
      ];
      includes = [ "*.rs" ];
      excludes = [ "src/main.rs" ];
    };
  };
}
