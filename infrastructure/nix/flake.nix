{
  description = "Autonomous swarm drone OS (Jetson / PX4)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    jetpack-nixos = {
      url = "github:anduril/jetpack-nixos";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    disko = {
      url = "github:nix-community/disko";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, jetpack-nixos, disko, ... }@inputs: let
    mkDrone = { hostname, system ? "aarch64-linux", profile }: 
      nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit inputs; };
        modules = [
          disko.nixosModules.disko
          jetpack-nixos.nixosModules.default

          ./modules/core/env.nix
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
  };
}
