{
  lib,
  config,
  inputs,
  ...
}: let
  isProd = config.ringil.env.mode == "prod";
  signing = config.ringil.tegra.signing.enable;
  vaultEnv = builtins.getEnv "RINGIL_JETSON_KEY_VAULT";
  vault =
    if vaultEnv == ""
    then "/secure/vault/ringil-jetson"
    else vaultEnv;
  keysetId = builtins.getEnv "RINGIL_JETSON_KEYSET_ID";
in {
  hardware.nvidia-jetpack = {
    enable = true;
    som = "orin-nano"; # Other options include orin-agx, xavier-agx, xavier-nx, and xavier-nx-emmc.
    carrierBoard = "devkit";
    super = false;

    firmware.autoUpdate = false;
    firmware.secureBoot.pkcFile =
      if signing
      then "${vault}/jetson_rcm_priv.pem"
      else null;
    firmware.secureBoot.requiredSystemFeatures = lib.mkIf signing ["ringil-jetson-signing"];
    firmware.secureBoot.preSignCommands = lib.mkIf signing ''
      if [ -z ${lib.escapeShellArg keysetId} ]; then
        printf '%s\n' 'Run the Ringil Jetson build command to validate the key vault before signing.' >&2
        exit 1
      fi
      printf 'Ringil Jetson: signing with key set %s\n' ${lib.escapeShellArg keysetId} >&2
    '';
    firmware.uefi.secureBoot.signer = lib.mkIf signing {
      cert = "${vault}/db.crt";
      key = "${vault}/db.key";
    };
  };

  assertions = [
    {
      assertion = !signing || isProd;
      message = "Tegra firmware signing is only enabled for production configurations.";
    }
  ];

  hardware.graphics.enable = true;

  nixpkgs.config.allowUnfree = true;
  nixpkgs.config.allowUnsupportedSystem = true;

  nixpkgs.overlays = [
    inputs.nix-ros-overlay.overlays.default
  ];

  environment.systemPackages = with config.nixpkgs.pkgs; [
    nvidia-jetpack.cudaPackages.cudatoolkit
    nvidia-jetpack.tensorrt
    nvidia-jetpack.tegra-eeprom-tool

    pciutils
    usbutils
  ];

  users.users.ringil = {
    extraGroups = ["video" "render"];
  };
}
