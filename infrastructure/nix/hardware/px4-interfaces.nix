{
  config,
  lib,
  pkgs,
  ...
}: let
  xrce-agent = pkgs.stdenv.mkDerivation rec {
    pname = "micro-xrce-dds-agent";
    version = "2.4.2";

    src = pkgs.fetchFromGitHub {
      owner = "eProsima";
      repo = "Micro-XRCE-DDS-Agent";
      rev = "v${version}";
      hash = "";
    };

    nativeBuildInputs = [pkgs.cmake];

    buildInputs = with pkgs.rosPackages.jazzy; [
      fastrtps
      fastcdr
      foonathan-memory-vendor
    ];

    cmakeFlags = [
      "-DUAGENT_SUPERBUILD=OFF"
      "-DUAGENT_BUILD_EXECUTABLE=ON"
    ];
  };

  arch =
    if pkgs.stdenv.hostPlatform.isAarch64
    then "aarch64"
    else "x86_64";

  zenoh-bridge = pkgs.stdenv.mkDerivation rec {
    pname = "zenoh-bridge-dds";
    version = "1.9.0";

    src = pkgs.fetchzip {
      url = "https://github.com/eclipse-zenoh/zenoh-plugin-dds/releases/download/${version}/zenoh-bridge-dds-${version}-${arch}-unknown-linux-gnu.zip";
      hash = "";
    };

    installPhase = ''
      mkdir -p $out/bin
      cp zenoh-bridge-dds $out/bin/
      chmod +x $out/bin/zenoh-bridge-dds
    '';
  };
in {
  services.udev.extraRules = ''
    KERNEL=="ttyTHS[0-9]*", GROUP="dialout", MODE="0660"
    KERNEL=="i2c-[0-9]*", GROUP="i2c", MODE="0660"
    KERNEL=="spidev*", GROUP="spi", MODE="0660"
  '';

  users.groups.i2c = {};
  users.groups.spi = {};

  systemd.services."px4-micro-xrce-dds" = {
    description = "eProsima Micro XRCE-DDS Agent for PX4";
    wantedBy = ["multi-user.target"];
    after = ["network.target"];
    serviceConfig = {
      ExecStart = "${xrce-agent}/bin/MicroXRCEAgent serial --dev /dev/ttyTHS1 -b 921600";
      Restart = "always";
      RestartSec = "2";
      User = "ringil";
      Group = "dialout";
    };
  };

  systemd.services."zenoh-bridge-dds" = {
    description = "Zenoh to DDS Bridge for Swarm and ROS 2";
    wantedBy = ["multi-user.target"];
    after = ["px4-micro-xrce-dds.service"];
    serviceConfig = {
      ExecStart = "${zenoh-bridge}/bin/zenoh-bridge-dds -d 42";
      Restart = "always";
      RestartSec = "2";
      User = "ringil";
    };
  };
}
