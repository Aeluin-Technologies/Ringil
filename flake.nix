{
  description = "Autonomous swarm drone OS";

  nixConfig = {
    extra-substituters = [
      "https://ros.cachix.org"
      "https://cache.nixos-cuda.org"
      "https://nix-community.cachix.org"
      "https://anduril.cachix.org"
    ];
    extra-trusted-public-keys = [
      "ros.cachix.org-1:dSyZxI8geDCJrwgvCOHDoAfOm5sV1wCPjBkKL+38Rvo="
      "cache.nixos-cuda.org:74DUi4Ye579gUqzH4ziL9IyiJBlDpMRn9MBN8oNan9M="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "anduril.cachix.org-1:69Y9YpYAsH9zDsqLaoW6NfO9U66TirFvJ0S69v4IioI="
    ];
  };

  inputs = {
    jetpack-nixos.url = "github:anduril/jetpack-nixos/master";
    jetpack-nixos.inputs.nixpkgs.follows = "nixpkgs";
    nix-ros-overlay.url = "github:lopsided98/nix-ros-overlay";
    nixpkgs.follows = "nix-ros-overlay/nixpkgs";
    disko.url = "github:nix-community/disko";
    disko.inputs.nixpkgs.follows = "nixpkgs";
    lanzaboote.url = "github:nix-community/lanzaboote/v0.4.2";
    lanzaboote.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    jetpack-nixos,
    disko,
    lanzaboote,
    nix-ros-overlay,
    ...
  } @ inputs: let
    supportedSystems = ["aarch64-linux"];
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

    sharedNixConfig = {
      nix.settings = {
        experimental-features = ["nix-command" "flakes"];
        substituters = [
          "https://cache.nixos.org"
          "https://ros.cachix.org"
          "https://anduril.cachix.org"
        ];
        trusted-public-keys = [
          "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
          "ros.cachix.org-1:dSyZxI8geDCJrwgvCOHDoAfOm5sV1wCPjBkKL+38Rvo="
          "anduril.cachix.org-1:69Y9YpYAsH9zDsqLaoW6NfO9U66TirFvJ0S69v4IioI="
        ];
        trusted-users = ["root" "@wheel"];
      };
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
            nix-ros-overlay.nixosModules.default
            disko.nixosModules.disko
            lanzaboote.nixosModules.lanzaboote

            {
              nixpkgs.config.allowUnfree = true;
              nixpkgs.config.allowUnsupportedSystem = true;
              nixpkgs.overlays = [
                inputs.nix-ros-overlay.overlays.default
                inputs.jetpack-nixos.overlays.default
              ];
            }

            sharedNixConfig

            ./infrastructure/nix/modules/core/env.nix
            ./infrastructure/nix/modules/core/bootloader.nix
            ./infrastructure/nix/modules/core/filesystems.nix
            ./infrastructure/nix/modules/ringil/ros2.nix
            ./infrastructure/nix/modules/network/galadril-link.nix
            ./infrastructure/nix/modules/security/users.nix
            ./infrastructure/nix/modules/observability/metrics.nix
            ./infrastructure/nix/modules/observability/logs.nix
            ./infrastructure/nix/modules/ringil/service.nix

            ./infrastructure/nix/machines/${profile}/default.nix
          ]
          ++ (
            if isSim
            then [{networking.hostName = hostname;}]
            else [
              jetpack-nixos.nixosModules.default
              ./infrastructure/nix/hardware/jetson.nix
              ./infrastructure/nix/hardware/px4-interfaces.nix
              ./infrastructure/nix/hardware/cuda-tensorrt.nix
              ./infrastructure/nix/modules/core/rt.nix
              ./infrastructure/nix/modules/security/lockdown.nix
              ./infrastructure/nix/modules/security/tpm-wg.nix
              ./infrastructure/nix/modules/security/tpm2.nix
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
        profile = "sim";
        system = "aarch64-linux";
        isSim = true;
      };
      "prod-swarm" = mkDrone {
        hostname = "prod-swarm";
        profile = "prod";
        system = "aarch64-linux";
      };
    };

    formatter.aarch64-linux = nixpkgs.legacyPackages.aarch64-linux.alejandra;
    formatter.aarch64-darwin = nixpkgs.legacyPackages.aarch64-darwin.alejandra;

    devShells = forAllSystems (system: let
      jetpackPkgs = import jetpack-nixos.inputs.nixpkgs {
        inherit system;
        config.allowUnfree = true;
        config.allowUnsupportedSystem = true;
        overlays = [jetpack-nixos.overlays.default];
      };

      rosPkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        config.allowUnsupportedSystem = true;
        overlays = [nix-ros-overlay.overlays.default];
      };
    in {
      default = rosPkgs.mkShell {
        name = "drone-dev";

        packages = [
          rosPkgs.colcon
          (with rosPkgs.rosPackages.lyrical; [
            ros-core
            rmw-zenoh-cpp
            behaviortree-cpp
          ])
        ];

        buildInputs = with rosPkgs; [
          pkg-config
          openssl
          rustc
          cargo
          alejandra
        ];

        shellHook = ''
          echo "🚀 Drone Dev Environment (${system})"

          export RMW_IMPLEMENTATION=rmw_zenoh_cpp
          export ROS_DOMAIN_ID=42
          echo "ROS 2 is active with RMW: $RMW_IMPLEMENTATION"
        '';
      };
    });
  };
}
