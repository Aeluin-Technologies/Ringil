{
  lib,
  config,
  ...
}: let
  isDev = config.ringil.env.mode == "dev";
in {
  services.prometheus.exporters.node = {
    enable = isDev;
    enabledCollectors = ["systemd" "cpu" "memory" "thermal"];
  };
}
