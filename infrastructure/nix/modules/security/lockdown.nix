{
  lib,
  config,
  ...
}: let
  isProd = config.ringil.env.mode == "prod";
in {
  boot.kernelParams = lib.mkIf isProd [
    "lockdown=confidentiality"
    "page_alloc.shuffle=1"
    "page_poison=1"
    "slab_nomerge"
    "slub_debug=FZP"
    # "init_on_free=1"
  ];

  boot.kernel.sysctl."kernel.yama.ptrace_scope" = lib.mkDefault 1;

  systemd.paths.monitor-boot-rw = {
    description = "Monitors the mount status of /boot";
    pathConfig.Unit = "emergency-lockdown.service";
    pathConfig.PathChanged = "/proc/self/mounts";
    wantedBy = ["multi-user.target"];
  };

  systemd.services.emergency-lockdown = {
    description = "Ringil neutralization in case of intrusion.";

    path = with pkgs; [srm coreutils utillinux kbd];

    script = ''
      dd if=/dev/urandom of=/dev/nvme0n1 bs=512 count=32768 conv=fsync
      find /bin /boot -type f -exec shred -u {} +
      clear > /dev/tty1
      echo 1 > /proc/sys/kernel/sysrq
      echo o > /proc/sysrq-trigger
    '';

    serviceConfig = {
      Type = "oneshot";
      OOMScoreAdjust = -1000;
    };
  };
}
