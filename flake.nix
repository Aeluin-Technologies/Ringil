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

    galadrilConfig = {
      endpoint = let
        env = builtins.getEnv "GALADRIL_ENDPOINT";
      in
        if env != ""
        then env
        else "localhost:51820";
      publicKey = let
        env = builtins.getEnv "GALADRIL_PUBKEY";
      in
        if env != ""
        then env
        else "";
    };

    mkDrone = {
      hostname,
      profile,
    }:
      nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {inherit inputs galadrilConfig;};
        modules = [
          disko.nixosModules.disko
          jetpack-nixos.nixosModules.default

          ./infrastructure/nix/hardware/jetson.nix
          ./infrastructure/nix/hardware/px4-interfaces.nix
          ./infrastructure/nix/hardware/cuda-tensorrt.nix
          ./infrastructure/nix/modules/core/env.nix
          ./infrastructure/nix/modules/core/bootloader.nix
          ./infrastructure/nix/modules/core/filesystems.nix
          ./infrastructure/nix/modules/network/galadril-link.nix
          ./infrastructure/nix/modules/security/anssi-kernel.nix
          ./infrastructure/nix/modules/security/lockdown.nix
          ./infrastructure/nix/modules/security/tpm-wg.nix
          ./infrastructure/nix/modules/security/tpm2.nix
          ./infrastructure/nix/modules/security/users.nix
          ./infrastructure/nix/modules/network/galadril-link.nix
          ./infrastructure/nix/modules/observability/metrics.nix
          ./infrastructure/nix/modules/observability/logs.nix
          ./infrastructure/nix/modules/ringil/service.nix

          ./infrastructure/nix/machines/${profile}/default.nix
          {networking.hostName = hostname;}
        ];
      };
  in {
    nixosConfigurations = {
      "dev-drone" = mkDrone {
        hostname = "dev-drone";
        profile = "dev";
      };
      "prod-swarm" = mkDrone {
        hostname = "prod-swarm";
        profile = "prod";
      };
    };

    formatter.${system} = nixpkgs.legacyPackages.${system}.alejandra;
    formatter.aarch64-darwin = nixpkgs.legacyPackages.aarch64-darwin.alejandra;
    formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.alejandra;
  };
}
