{
  lib,
  config,
  ...
}: let
  cfg = config.ringil.env;
  isProd = cfg.mode == "prod";
in {
  boot.loader.systemd-boot.enable = lib.mkDefault true;

  boot.loader.efi.canTouchEfiVariables = true;
  boot.loader.timeout = lib.mkIf isProd 0;

  boot.loader.systemd-boot.editor = lib.mkIf isProd false;
  boot.initrd.systemd.emergencyAccess = lib.mkIf isProd false;
}
