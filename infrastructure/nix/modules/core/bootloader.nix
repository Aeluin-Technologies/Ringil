{
  lib,
  config,
  ...
}: let
  cfg = config.ringil.env;
in {
  boot.loader.systemd-boot.enable = lib.mkDefault true;
  boot.loader.efi.canTouchEfiVariables = true;

  boot.loader.timeout = lib.mkIf (cfg.mode == "prod") 0;
}
