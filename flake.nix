{
  description = "Build restedpi for the Raspberry Pi";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, flake-utils, rust-overlay, ... }: 
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = { self', pkgsm, lib, system, ... }:
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
          noCheck = true;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ pkgs.sqlite ];
          buildFeatures = [ "raspberrypi" ];
        };

      in {
        packages = rec {
          restedpi = rustBuild;
          default = restedpi;
        };

        devShells.default = pkgs.mkShell { 
          DATABASE_URL = "dev-restedpi.db";
          buildInputs = with pkgs; [ 
            nixfmt 
            sqlite 
            diesel-cli
            rustVersion 
        ]; };

        apps = {
          info = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "info" ''
              echo "HEY WORLD"
            '';
          };
        };
      };
  };
}
