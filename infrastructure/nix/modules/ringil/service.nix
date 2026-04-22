{
  pkgs,
  lib,
  config,
  ...
}: let
  ringilPkg = pkgs.callPackage ./package.nix {};
in {
  systemd.services.ringil = {
    description = "Ringil Autonomous Node Daemon";
    wantedBy = ["multi-user.target"];
    after = ["network.target" "wg-quick-wg-galadril.service"];

    serviceConfig = {
      ExecStart = "${ringilPkg}/bin/ringil-daemon";
      Restart = "always";
      RestartSec = "1s";

      User = "ringil";
      Group = "ringil";

      LimitRTPRIO = 99;
      CPUSchedulingPolicy = "fifo";
      CPUSchedulingPriority = 80;

      NoNewPrivileges = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      PrivateTmp = true;
      DeviceAllow = [
        "/dev/ttyTHS* rw" # MAVLink UART.
        "/dev/i2c-* rw" # I2C.
        "/dev/video* rw" # Camera V4L2.
      ];
    };
  };
}
