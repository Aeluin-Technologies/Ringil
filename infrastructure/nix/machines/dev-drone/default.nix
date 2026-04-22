{...}: {
  imports = [
    ../../profiles/dev.nix
    ../../modules/core/filesystems.nix
  ];

  services.openssh.settings.PermitRootLogin = "yes";

  environment.systemPackages = [(import ../../modules/ringil/package.nix {inherit pkgs;})];
}
