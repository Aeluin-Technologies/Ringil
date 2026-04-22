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

    firmware.autoUpdate = false;
  };

  hardware.graphics.enable = true;
}
