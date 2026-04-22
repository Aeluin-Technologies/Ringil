{
  description = "Autonomous swarm drone OS (Jetson / PX4)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    jetpack-nixos.url = "github:anduril/jetpack-nixos";
    jetpack-nixos.inputs.nixpkgs.follows = "nixpkgs";
    disko.url = "github:nix-community/disko";
    disko.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    jetpack-nixos,
    disko,
    ...
  } @ inputs: let
    system = "aarch64-linux";

    mkDrone = {
      hostname,
      profile,
    }:
      nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {inherit inputs;};
        modules = [
          disko.nixosModules.disko
          jetpack-nixos.nixosModules.default

          ./modules/core/env.nix
          ./modules/core/bootloader.nix
          ./modules/core/filesystems.nix
          ./hardware/jetson.nix
          ./hardware/px4-interfaces.nix
          ./hardware/cuda-tensorrt.nix
          ./modules/security/anssi-kernel.nix
          ./modules/security/users.nix
          ./modules/network/galadril-link.nix
          ./modules/observability/metrics.nix
          ./modules/ringil/service.nix

          ./profiles/${profile}.nix
          {networking.hostName = hostname;}
        ];
      };
  in {
    nixosConfigurations = {
      "dev-drone-01" = mkDrone {
        hostname = "dev-drone-01";
        profile = "dev";
      };
      "prod-swarm-alpha" = mkDrone {
        hostname = "prod-swarm-alpha";
        profile = "prod";
      };
    };

    formatter.${system} = nixpkgs.legacyPackages.${system}.alejandra;
    formatter.aarch64-darwin = nixpkgs.legacyPackages.aarch64-darwin.alejandra;
  };
}
