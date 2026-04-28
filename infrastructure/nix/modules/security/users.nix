{
  lib,
  config,
  ...
}: let
  isProd = config.ringil.env.mode == "prod";
in {
  users.mutableUsers = false;

  users.users.ringil = {
    isSystemUser = true;
    group = "ringil";
    extraGroups = ["dialout" "i2c" "spi" "video"];
    shell = if isProd then "${pkgs.shadow}/bin/nologin" else pkgs.bash;
    hashedPassword = lib.mkIf isProd "!";
  };
  users.groups.ringil = {};

  users.users.root = {
    hashedPassword = lib.mkIf isProd "!";
    shell = lib.mkIf isProd "${pkgs.shadow}/bin/nologin";
  };

  security.sudo.enable = lib.mkDefault (!isProd);
}
