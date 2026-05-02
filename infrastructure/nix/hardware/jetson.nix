{
  lib,
  config,
  inputs,
  ...
}: {
  hardware.nvidia-jetpack = {
    enable = true;
    som = "orin-nano"; # Other options include orin-agx, xavier-agx, xavier-nx, and xavier-nx-emmc.
    carrierBoard = "devkit";
    super = false;

    firmware.autoUpdate = false;
  };

  hardware.graphics.enable = true;

  nixpkgs.config.allowUnfree = true;
  nixpkgs.config.allowUnsupportedSystem = true;

  nixpkgs.overlays = [
    inputs.nix-ros-overlay.overlays.default
  ];

  environment.systemPackages = with config.nixpkgs.pkgs; [
    nvidia-jetpack.cudaPackages.cudatoolkit
    nvidia-jetpack.tensorrt

    pciutils
    usbutils
  ];

  users.users.ringil = {
    extraGroups = ["video" "render"];
  };
}
