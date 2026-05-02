{
  config,
  lib,
  pkgs,
  ...
}: {
  boot.kernelPackages = lib.mkDefault pkgs.nvidia-jetpack.rtkernelPackages;

  boot.kernelParams = [
    "preempt=full"
    "threadirqs"
    "cpufreq.default_governor=performance"
  ];

  powerManagement.cpuFreqGovernor = "performance";

  security.pam.loginLimits = [
    {
      domain = "@ringil";
      item = "rtprio";
      type = "-";
      value = "99";
    }
    {
      domain = "@ringil";
      item = "memlock";
      type = "-";
      value = "unlimited";
    }
    {
      domain = "@ringil";
      item = "nofile";
      type = "-";
      value = "65535";
    }
  ];
}
