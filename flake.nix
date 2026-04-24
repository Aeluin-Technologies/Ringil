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
    supportedSystems = ["aarch64-linux" "x86_64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

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
      system,
      isSim ? false,
    }:
      nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {inherit inputs galadrilConfig;};
        modules =
          [
            disko.nixosModules.disko

            ./infrastructure/nix/modules/core/env.nix
            ./infrastructure/nix/modules/core/bootloader.nix
            ./infrastructure/nix/modules/core/filesystems.nix
            ./infrastructure/nix/modules/network/galadril-link.nix
            ./infrastructure/nix/modules/security/users.nix
            ./infrastructure/nix/modules/observability/metrics.nix
            ./infrastructure/nix/modules/observability/logs.nix
            ./infrastructure/nix/modules/ringil/service.nix
          ]
          ++ (
            if isSim
            then [
              {
                networking.hostName = hostname;
                hardware.opengl.enable = true;
                security.anssi-kernel.enable = false;
                environment.systemPackages = with nixpkgs.legacyPackages.${system}; [
                  gazebo
                  mavlink
                  python3Packages.mavros
                ];
              }
            ]
            else [
              jetpack-nixos.nixosModules.default
              ./infrastructure/nix/hardware/jetson.nix
              ./infrastructure/nix/hardware/px4-interfaces.nix
              ./infrastructure/nix/hardware/cuda-tensorrt.nix
              ./infrastructure/nix/modules/security/anssi-kernel.nix
              ./infrastructure/nix/modules/security/lockdown.nix
              ./infrastructure/nix/modules/security/tpm-wg.nix
              ./infrastructure/nix/modules/security/tpm2.nix
              ./infrastructure/nix/machines/${profile}/default.nix
              {networking.hostName = hostname;}
            ]
          );
      };
  in {
    nixosConfigurations = {
      "dev-drone" = mkDrone {
        hostname = "dev-drone";
        profile = "dev";
        system = "aarch64-linux";
      };

      "sim-drone" = mkDrone {
        hostname = "sim-drone";
        profile = "dev";
        system = "aarch64-linux";
        isSim = true;
      };

      "prod-swarm" = mkDrone {
        hostname = "prod-swarm";
        profile = "prod";
        system = "aarch64-linux";
      };
    };

    formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.alejandra);
    devShells = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        name = "drone-dev";
        buildInputs = with pkgs; [
          rustc
          cargo
          rust-analyzer
          gazebo
          mavlink
          pkg-config
          openssl
          alejandra
        ];

        shellHook = ''
          echo "🚀 Drone Dev Environment (${system})"
          echo "Target: ${
            if system == "aarch64-linux"
            then "Jetson/VM"
            else "x86_64 PC"
          }"
        '';
      };
    });
  };
}
