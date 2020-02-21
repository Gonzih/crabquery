let
   pkgs = import <nixpkgs> {};
in pkgs.stdenv.mkDerivation rec {
  name = "rquery";
  buildInputs = with pkgs; [
    stdenv
    glib
    pkgconfig
    rustup
    cargo
    curl
  ];
}
