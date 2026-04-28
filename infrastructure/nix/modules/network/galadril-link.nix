{
  config,
  lib,
  galadrilConfig,
  ...
}: {
  networking.wireguard.interfaces = {
    wg-galadril = {
      ips = ["10.100.0.10/24"];
      privateKeyFile = "/run/wireguard/private.key";

      preSetup = ''
        while [ ! -f /run/wireguard/private.key ]; do
          sleep 0.1
        done
      '';

      peers = [
        {
          publicKey = galadrilConfig.publicKey;
          allowedIPs = ["10.100.0.0/16"];
          endpoint = galadrilConfig.endpoint;
          persistentKeepalive = 25;
        }
      ];
    };
  };

  systemd.services."wireguard-wg-galadril" = {
    after = ["unlock-wg-key.service"];
    requires = ["unlock-wg-key.service"];
  };
}
