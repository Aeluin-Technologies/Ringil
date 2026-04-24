{...}: {
  system.stateVersion = "26.05";

  imports = [
    ../../profiles/prod.nix
    ../../modules/security/anssi-kernel.nix
    ../../modules/core/filesystems.nix
  ];

  systemd.services.ringil.environment = {
    NODE_ID = "alpha-01";
    SWARM_ID = "shadow-fleet";
  };
}
