{
  lib,
  config,
  pkgs,
  ...
}: let
  cfg = config.ringil.env;
  isProd = cfg.mode == "prod";
in {
  boot.loader.systemd-boot.enable = lib.mkIf (!isProd) (lib.mkDefault true);
  
  boot.loader.efi.canTouchEfiVariables = true;
  boot.loader.timeout = lib.mkIf isProd 0;
  
  boot.lanzaboote = lib.mkIf isProd {
    enable = true;
    # Key must be generated using `sbctl create-keys`
    pkiBundle = "/etc/secure/keys";
  };

  boot.loader.systemd-boot.editor = lib.mkIf isProd false;
  boot.initrd.systemd.emergencyAccess = lib.mkIf isProd false;
}
