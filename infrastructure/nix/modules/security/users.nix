{
  lib,
  config,
  pkgs,
  ...
}: let
  isProd = config.ringil.env.mode == "prod";
in {
  users.mutableUsers = !isProd;
  users.allowNoPasswordLogin = isProd;

  users.users.ringil = {
    isSystemUser = true;
    group = "ringil";
    extraGroups = ["dialout" "i2c" "spi" "video"];
    hashedPassword = lib.mkIf isProd "!";
    shell =
      if isProd
      then "${pkgs.shadow}/bin/nologin"
      else pkgs.bash;
  };
  users.groups.ringil = {};

  users.users.root = {
    hashedPassword = lib.mkIf isProd "!";
    shell = lib.mkIf isProd "${pkgs.shadow}/bin/nologin";
  };

  security.sudo.enable = lib.mkDefault (!isProd);
}
