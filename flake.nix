{
  description = "Stock Market Simulation";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        }
        ideSetup = import ~/setup.nix;
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = ideSetup.buildInputs ++ [
            rust-bin.nightly.latest.default
            clippy
            linuxPackages_latest.perf # Profiler
          ];
          shellHook = ''
          ${ideSetup.shellHook}
          '';
        };
      };
    );
}
