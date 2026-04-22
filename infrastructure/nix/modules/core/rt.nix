{ pkgs, ... }:

{
  boot.kernelPackages = pkgs.linuxPackages_rt;

  boot.kernelParams = [ "threadirqs" ];
  powerManagement.cpuFreqGovernor = "performance";

  security.pam.loginLimits = [

  ];
}
