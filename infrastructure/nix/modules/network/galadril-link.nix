{
  config,
  lib,
  ...
}: {
  networking.wireguard.interfaces = {
    wg-galadril = {
      ips = ["10.100.0.10/24"];
      privateKeyFile = "/run/wireguard/private.key";

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
}
