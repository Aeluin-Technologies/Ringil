{
  lib,
  config,
  ...
}: {
  hardware.nvidia-jetpack = {
    enable = true;
    som = "orin-nano"; # Other options include orin-agx, xavier-agx, xavier-nx, and xavier-nx-emmc.
    carrierBoard = "devkit";
    super = false;

    firmware.update.enable = false;
  };

  hardware.graphics = {
    enable = true;
    driSupport = true;
    extraPackages = with config.boot.kernelPackages; [
      nvidia-tegra
    ];
  };
}
