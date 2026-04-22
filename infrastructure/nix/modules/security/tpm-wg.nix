{pkgs, ...}: {
  systemd.tmpfiles.rules = [
    "d /run/wireguard 0700 root root -"
  ];

  systemd.services.unlock-wg-key = {
    description = "Unlock WireGuard key via TPM2";

    after = ["tpm2-abrmd.service"];
    requires = ["tpm2-abrmd.service"];

    before = ["wireguard-wg-galadril.service"];
    wantedBy = ["multi-user.target"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };

    script = ''
      if [ -f /etc/secure/wg_key.blob ]; then
        ${pkgs.tpm2-tools}/bin/tpm2_unseal -c /etc/secure/wg_key.blob > /run/wireguard/private.key
        chmod 600 /run/wireguard/private.key
      else
        echo "ERROR: No blob key found on /etc/secure/"
        exit 1
      fi
    '';
  };
}
