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
  };
  users.groups.ringil = {};

  users.users.root.hashedPassword = lib.mkIf isProd "!";
}
