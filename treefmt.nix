{ ... }:
{
  projectRootFile = "flake.nix";
  programs = {
    typos.enable = true;
    nixfmt.enable = true;
    yamlfmt.enable = true;
    rustfmt.enable = true;
  };
}
