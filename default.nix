let
  pkgs = import <nixpkgs> {};
in
pkgs.stdenv.mkDerivation {
  name = "stocks";
  src = ./.;

  buildInputs = with pkgs; [ rustc cargo ];

  buildPhase = ''
    cargo build --release
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/my-rust-project $out/bin/
  '';
}