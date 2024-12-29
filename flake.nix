{
  description = "Rust + Nix";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = { self, nixpkgs }: {
    devShell = pkgs: pkgs.mkShell {
      buildInputs = with pkgs; [
        rustc       # Rust compiler
        cargo       # Rust package manager
        rustfmt     # Code formatter
        clippy      # Linter
      ];
    };
  };
}
