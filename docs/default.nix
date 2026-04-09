{ pkgs }:
pkgs.stdenv.mkDerivation {
  pname = "kystash-docs";
  version = "0.0.1";

  src = ./.;

  nativeBuildInputs = [ pkgs.mdbook ];

  buildPhase = ''
    mdbook build
  '';

  installPhase = ''
    mkdir -p $out
    cp -r book/* $out/
  '';
}
