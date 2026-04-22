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

  # nix-shell -p sbctl then sudo sbctl create-keys
  boot.lanzaboote = {
    enable = lib.mkIf (cfg.mode == "prod") true;
    pkiBundle = "/etc/secureboot";
  };
}
