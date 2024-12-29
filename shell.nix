let
  pkgs = import <nixpkgs> {};
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc       # Rust compiler
    cargo       # Rust package manager
    rustfmt     # Code formatter
    clippy      # Linter
  ];
}