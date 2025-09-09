# https://fasterthanli.me/series/building-a-rust-service-with-nix/part-10
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      with pkgs; {
        devShells.default = mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkg-config
            hwloc

            # Preprocess emails
            unrar
          ];
          buildInputs = [
            (python3.withPackages (python-pkgs: with python-pkgs; [
              dash
              dash-bootstrap-components
              pandas
              plotly
              polars
              pyarrow
              python-lsp-ruff
              python-lsp-server

              # Preprocess emails
              click
              rarfile
              tqdm
            ]))
          ];
        };
      }
    );
}
