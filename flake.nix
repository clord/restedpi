{
  description = "Build restedpi for the Raspberry Pi";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
        rustBuild = rustPlatform.buildRustPackage {
          pname = "restedpi";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ pkgs.sqlite ];
        };
      in {
        defaultPackage = rustBuild;
        devShell = pkgs.mkShell {
          buildInputs =
            [
              pkgs.sqlite
              (rustVersion.override { extensions = [ "rust-src" ]; }) 
            ];
        };
      });
}
