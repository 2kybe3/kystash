{ ... }:
{
  projectRootFile = "flake.nix";
  programs = {
    typos.enable = true;
    nixfmt.enable = true;
    rustfmt.enable = true;
  };
}
