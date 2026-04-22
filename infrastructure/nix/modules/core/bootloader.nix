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

  # TODO: Replace with Lanzaboote (SecureBoot) once the keys have been generated.
  # boot.lanzaboote.enable = lib.mkIf (cfg.mode == "prod") true;
}
