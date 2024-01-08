{
	description = "Build restedpi for the Raspberry Pi";
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay.url = "github:oxalica/rust-overlay";
		sops-nix.url = "github:Mic92/sops-nix";
  	};

	outputs = { self, nixpkgs, sops-nix, flake-utils, rust-overlay, ... }: let
		system = "aarch64-linux";
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
				"${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
				({...}: {
					config = {
						# Sops config
						imports =  [ <sops-nix/modules/sops> ];
						sops.defaultSopsFile = ./secrets/rpi.yaml;
						sops.age.keyFile = "/var/lib/sops-nix/key.txt";
						sops.age.generateKey = true;
						sops.secrets.application_key = {};
						
						time.timeZone = "America/Edmonton";
						i18n.defaultLocale = "en_CA.UTF-8";
						sdImage.compressImage = false;
						system = { stateVersion = "23.11";};
						
						networking = {
							wireless.enable = false;
							useDHCP = true;
						};
						hardware.bluetooth.powerOnBoot = false;
						users.users.clord = {
							isNormalUser = true;
							description = "Christopher Lord";
							extraGroups = ["wheel"];
							openssh.authorizedKeys.keys = [
								"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIP3DeyWHOIc+SdTqNP9iFD4jpf0fg1FVTsaWn2qcKDTa clord@edmon"
								"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINXLYw43gNlnfEoHpmK/UWae4DcQyLBQTGQH9ZYlRG5q clord@wildwood"
								"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINLtiIXQ0r+l0gtnjCj1hT5Z1YzRqgJ/g66pP/eEuXM3 clord@ipad"
								"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIO7InFCQXQltDI/NMOlm5ayhQmLnSN4K4TwX+KRlG3OG rpi-deploy"
								"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHTOl4xwPOT82EmW5bEBpWyi5Iy9ZEYWPToJEQjIagyO clord@1p"
							];	
						};
						environment.systemPackages = [
							pkgs.git
							pkgs.htop
							pkgs.vim
							rustBuild
						];
						nix = {
							package = pkgs.nix;
							settings.experimental-features = [ "nix-command" "flakes" ];
						};	
						services.openssh = {
							enable = true;
							settings.PasswordAuthentication = false;
							settings.KbdInteractiveAuthentication = false;
						};
						systemd.services.restedpi = {
							enable = true;
							environment = {
								RUST_BACKTRACE = "1";
							};
							description = "Runs the restedpi system";
							unitConfig = {
								
							};
							serviceConfig = {
								ExecStart = "${rustBuild}/bin/restedpi --config-file ${configFile} --log-level 'warn' server";
							};
							wantedBy = [ "multi-user.target" ];
						};
					};
				})
			];
		};
	in {
		defaultPackage = rustBuild;
		rpi4 = rpi4System.config.system.build.sdImage;
		devShell = pkgs.mkShell {
			buildInputs = [
				pkgs.sqlite
				(rustVersion.override { extensions = [ "rust-src" ]; })
			];
		};
	};
}
