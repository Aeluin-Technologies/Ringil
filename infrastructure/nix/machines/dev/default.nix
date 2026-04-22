{...}: {
  imports = [
    ../../profiles/dev.nix
    ../../modules/core/filesystems.nix
  ];

  services.openssh.settings.PermitRootLogin = "yes";
}
