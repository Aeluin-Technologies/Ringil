{lib, ...}: {
  ringil.env.mode = "prod";
  users.allowNoPasswordLogin = false;

  services.openssh.enable = false;
  users.users.root.hashedPassword = "!";

  services.journald.extraConfig = ''
    Storage=volatile
    RuntimeMaxUse=50M
  '';

  boot.kernelParams = ["quiet" "loglevel=0" "console=tty0"];
  boot.blacklistedKernelModules = [
    "bluetooth" "btusb"
    "firewire-core"
    "thunderbolt" "v4l2loopback"
  "dvb_core"
  "saa7134"
  "uas"
  "ums_realtek" "usb_storage"
  ];
  boot.initrd.availableKernelModules = [ 
  "sdhci_tegra" 
  "nvme"
  "cdc_acm"
  "uvcvideo"
];
  systemd.services."serial-getty@ttyS0".enable = false;

  services.udisks2.enable = false;
  services.xserver.enable = false;
  documentation.enable = false;
  documentation.nixos.enable = false;

  services.usbguard = {
    enable = true;
    defaultPolicy = "block";
    presentDevicePolicy = "apply-policy";
  };
}
