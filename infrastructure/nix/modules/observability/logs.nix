{
  lib,
  config,
  ...
}: let
  isProd = config.ringil.env.mode == "prod";
in {
  services.journald.extraConfig =
    if isProd
    then ''
      Storage=volatile
      RuntimeMaxUse=50M
      MaxLevelStore=info
    ''
    else ''
      Storage=persistent
      SystemMaxUse=1G
    '';

  boot.consoleLogLevel =
    if isProd
    then 0
    else 4;
}
