{...}: {
  system.stateVersion = "26.05";

  imports = [
    ../../profiles/dev.nix
    ../../modules/core/filesystems.nix
  ];

  services.openssh.settings.PermitRootLogin = "yes";
}
