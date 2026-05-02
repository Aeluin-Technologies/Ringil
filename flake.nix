{
  description = "Autonomous swarm drone OS (Jetson / PX4)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    jetpack-nixos.url = "github:anduril/jetpack-nixos";
    jetpack-nixos.inputs.nixpkgs.follows = "nixpkgs";
    disko.url = "github:nix-community/disko";
    disko.inputs.nixpkgs.follows = "nixpkgs";
    lanzaboote.url = "github:nix-community/lanzaboote/v0.4.2";
    lanzaboote.inputs.nixpkgs.follows = "nixpkgs";
    nix-ros-overlay.url = "github:lopsided98/nix-ros-overlay";
    nix-ros-overlay.inputs.nixpkgs.follows = "nixpkgs";
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
    supportedSystems = ["aarch64-darwin" "aarch64-linux" "x86_64-linux"];
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
            nix-ros-overlay.nixosModules.default
            disko.nixosModules.disko
            lanzaboote.nixosModules.lanzaboote

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
            then [
              {networking.hostName = hostname;}
            ]
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

    formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.alejandra);

    devShells = forAllSystems (system: let
      isLinux = nixpkgs.lib.hasSuffix "-linux" system;
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (final: prev:
            if isLinux
            then nix-ros-overlay.overlays.default final prev
            else {})
          (final: prev: {
            vcstool = prev.vcs2l;
            pythonPackagesExtensions =
              prev.pythonPackagesExtensions
              ++ [
                (pyFinal: pyPrev: {
                  vcstool = pyFinal.vcs2l;
                })
              ];
          })
        ];
      };
    in {
      default = pkgs.mkShell {
        name = "drone-dev";
        buildInputs = with pkgs;
          [
            rustc
            cargo
            rust-analyzer
            pkg-config
            openssl
            alejandra
          ]
          ++ lib.optionals isLinux [
            rosPackages.jazzy.ros-core
            rosPackages.jazzy.rmw-zenoh-cpp
            rosPackages.jazzy.behaviortree-cpp
          ];

        shellHook = ''
          echo "🚀 Drone Dev Environment (${system})"
          echo "Target: ${
            if system == "aarch64-linux"
            then "Jetson/VM"
            else "x86_64 PC"
          }"

          export RMW_IMPLEMENTATION=rmw_zenoh_cpp
          export ROS_DOMAIN_ID=42
          echo "ROS 2 is active with RMW: $RMW_IMPLEMENTATION"
        '';
      };
    });
  };
}
