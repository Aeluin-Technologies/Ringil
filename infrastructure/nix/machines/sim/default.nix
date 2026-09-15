{
  pkgs,
  lib,
  ...
}: {
  system.stateVersion = "26.05";
  ringil.env.mode = "dev";

  users.mutableUsers = lib.mkForce true;
  users.allowNoPasswordLogin = lib.mkForce true;
  users.users.root.initialPassword = "nixos";

  hardware.graphics = {
    enable = true;
  };

  environment.systemPackages = with pkgs; [
    gazebo
    mavlink
    qgroundcontrol
    (python3.withPackages (ps:
      with ps; [
        mavros
      ]))
  ];

  networking.useDHCP = lib.mkDefault true;
}
