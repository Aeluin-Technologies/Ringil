{
  ringil.env.mode = "dev";
  services.openssh.enable = true;
  users.users.root.hashedPassword = "";
  services.journald.extraConfig = "Storage=persistent";
}
