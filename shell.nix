{ pkgs ? import <nixpkgs> {}, ... }:
let
  ideSetup = import ~/setup.nix { inherit pkgs; };
in
pkgs.mkShell {
  name = "Stock Market Simulation";

  buildInputs = ideSetup.buildInputs ++ (with pkgs; [
    rustc       # Rust compiler
    cargo       # Rust package manager
    rustfmt     # Code formatter
    clippy      # Linter
    linuxPackages_latest.perf # Profiler
  ]);
  shellHook = ''
   ${ideSetup.shellHook}
  '';
}
