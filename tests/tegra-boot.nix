let
  lib = {
    mkDefault = value: value;
    mkIf = condition: value:
      if condition
      then value
      else null;
  };
  packages = {
    nvidia-jetpack = {
      cudaPackages.cudatoolkit = "cuda";
      tensorrt = "tensorrt";
      tegra-eeprom-tool = "tegra-eeprom-tool";
    };
    pciutils = "pciutils";
    usbutils = "usbutils";
  };
  inputs.nix-ros-overlay.overlays.default = "ros-overlay";
  configFor = mode: signing: {
    ringil.env.mode = mode;
    ringil.tegra.signing.enable = signing;
    nixpkgs.pkgs = packages;
  };
  bootFor = mode:
    import ../infrastructure/nix/modules/core/bootloader.nix {
      inherit lib;
      config = configFor mode false;
      pkgs = packages;
    };
  jetsonFor = mode: signing:
    import ../infrastructure/nix/hardware/jetson.nix {
      inherit lib inputs;
      config = configFor mode signing;
    };
  productionBoot = bootFor "prod";
  developmentBoot = bootFor "dev";
  productionJetson = jetsonFor "prod" false;
  signedJetson = jetsonFor "prod" true;
  developmentJetson = jetsonFor "dev" false;
in
  assert productionBoot.boot.loader.systemd-boot.enable;
  assert developmentBoot.boot.loader.systemd-boot.enable;
  assert !(productionBoot.boot ? lanzaboote);
  assert productionJetson.hardware.nvidia-jetpack.firmware.secureBoot.pkcFile == null;
  assert productionJetson.hardware.nvidia-jetpack.firmware.uefi.secureBoot.signer == null;
  assert signedJetson.hardware.nvidia-jetpack.firmware.secureBoot.pkcFile == "/secure/vault/ringil-jetson/jetson_rcm_priv.pem";
  assert signedJetson.hardware.nvidia-jetpack.firmware.uefi.secureBoot.signer.cert == "/secure/vault/ringil-jetson/db.crt";
  assert signedJetson.hardware.nvidia-jetpack.firmware.uefi.secureBoot.signer.key == "/secure/vault/ringil-jetson/db.key";
  assert builtins.elem "ringil-jetson-signing" signedJetson.hardware.nvidia-jetpack.firmware.secureBoot.requiredSystemFeatures;
  assert developmentJetson.hardware.nvidia-jetpack.firmware.secureBoot.pkcFile == null;
  assert builtins.elem "tegra-eeprom-tool" productionJetson.environment.systemPackages; true
