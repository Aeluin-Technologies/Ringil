{pkgs, ...}: {
  security.tpm2 = {
    enable = true;
    pkcs11.enable = true;
    tctiEnvironment.enable = true;
  };

  environment.systemPackages = with pkgs; [
    tpm2-tools
    tpm2-tss
    tpm2-abrmd
  ];

  users.users.ringil.extraGroups = ["tss"];
}
