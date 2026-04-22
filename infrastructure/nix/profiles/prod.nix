{lib, ...}: {
  ringil.env.mode = "prod";
  users.allowNoPasswordLogin = true;

  services.openssh.enable = false;
  users.users.root.hashedPassword = "!";

  services.journald.extraConfig = ''
    Storage=volatile
    RuntimeMaxUse=50M
  '';

  boot.kernelParams = ["quiet" "loglevel=0" "console=tty0"];
  systemd.services."serial-getty@ttyS0".enable = false;
}
