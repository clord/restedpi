{
  description = "Build restedpi for the Raspberry Pi";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    sops-nix.url = "github:Mic92/sops-nix";
  };

  outputs = { self, nixpkgs, sops-nix, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "aarch64-linux" "aarch64-darwin" ] (system:
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

        rpi4System = nixpkgs.lib.nixosSystem {
          system = "aarch64-linux";
          modules = [
            sops-nix.nixosModules.sops
            {
              sops.defaultSopsFile = ./secrets/rpi.yaml;
              sops.age.keyFile = "/var/lib/sops/age/keys.txt";
              sops.age.generateKey = true;

              # Expose as /run/secrets/application_secret
              sops.secrets.application_secret = { };
              sops.secrets.configuration = { };
              # Expose as /run/secrets/rpi_cert
              sops.secrets.rip_cert = { };
              # Expose as /run/secrets/rpi_key
              sops.secrets.rip_key = { };

              # mkpasswd -m sha-512
              sops.secrets.clord_password = { neededForUsers = true; };
            }
            "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
            ({ config, ... }: {
              config = {

                time.timeZone = "America/Edmonton";
                services.timesyncd.enable = true;
                i18n.defaultLocale = "en_CA.UTF-8";
                sdImage.compressImage = false;
                system = { stateVersion = "23.11"; };

                networking = {
                  hostName = "chickenpi";
                  wireless.enable = false;
                  useDHCP = true;
                };

                hardware.bluetooth.powerOnBoot = false;

                users = {
                  mutableUsers = false;
                  users.clord = {
                    isNormalUser = true;
                    description = "Christopher Lord";
                    hashedPasswordFile =
                      config.sops.secrets.clord_password.path;
                    extraGroups = [ "wheel" ];
                    openssh.authorizedKeys.keys = [
                      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIP3DeyWHOIc+SdTqNP9iFD4jpf0fg1FVTsaWn2qcKDTa clord@edmon"
                      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINXLYw43gNlnfEoHpmK/UWae4DcQyLBQTGQH9ZYlRG5q clord@wildwood"
                      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINLtiIXQ0r+l0gtnjCj1hT5Z1YzRqgJ/g66pP/eEuXM3 clord@ipad"
                      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHTOl4xwPOT82EmW5bEBpWyi5Iy9ZEYWPToJEQjIagyO clord@1p"
                    ];
                  };
                };
                environment.systemPackages =
                  [ pkgs.git pkgs.htop pkgs.vim rustBuild ];
                nix = {
                  package = pkgs.nix;
                  settings = {
                    auto-optimise-store = true;
                    max-jobs = 4;
                    cores = 4;
                    trusted-users = [ "root" "clord" "@wheel" ];
                    experimental-features = [ "nix-command" "flakes" ];
                  };
                  gc = {
                    automatic = true;
                    dates = "weekly";
                    options = "--delete-older-than 60d";
                  };
                  # Free up to 1GiB whenever there is less than 100MiB left.
                  extraOptions = "  min-free = ${
                        toString (100 * 1024 * 1024)
                      }\n  max-free = ${
                        toString (1024 * 1024 * 1024)
                      }\n  keep-outputs = true\n  keep-derivations = true\n";
                };
                services.openssh = {
                  enable = true;
                  settings = {
                    PermitRootLogin = "yes";
                    PasswordAuthentication = false;
                    KbdInteractiveAuthentication = false;
                  };
                };
                services.prometheus = {
                  exporters = {
                    node = {
                      enable = true;
                      enabledCollectors = [ "systemd" ];
                      port = 9002;
                    };
                  };
                };
                systemd.services.restedpi = {
                  enable = true;
                  environment = { RUST_BACKTRACE = "1"; };
                  description = "Runs the restedpi system";
                  unitConfig = {

                  };
                  serviceConfig = {
                    ExecStart =
                      "${rustBuild}/bin/restedpi --config-file /run/secrets/configuration --log-level warn server";
                  };
                  wantedBy = [ "multi-user.target" ];
                };
              };
            })
          ];
        };
      in {
        defaultPackage.aarch64_linux = rustBuild;
        devShell = pkgs.mkShell { buildInputs = with pkgs; [ nixfmt sqlite ]; };

        apps = {
          info = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "info" ''
              echo "HEY WORLD"
            '';
          };
        };
        rpi4 = rpi4System.config.system.build.sdImage;
      });
}
