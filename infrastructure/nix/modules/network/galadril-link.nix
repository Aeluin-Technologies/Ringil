{
  config,
  lib,
  ...
}: {
  networking.wireguard.interfaces = {
    wg-galadril = {
      ips = ["10.100.0.10/24"];
      privateKeyFile = "/etc/wireguard/private.key";

      peers = [
        {
          publicKey = "CLÉ_PUBLIQUE_GALADRIL_ICI";
          allowedIPs = ["10.100.0.0/16"];
          endpoint = "galadril.votre-domaine.com:51820";
          persistentKeepalive = 25;
        }
      ];
    };
  };
}
