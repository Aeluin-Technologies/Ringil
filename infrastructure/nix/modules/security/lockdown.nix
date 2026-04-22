{
  lib,
  config,
  ...
}: let
  isProd = config.ringil.env.mode == "prod";
in {
  boot.kernelParams = lib.mkIf isProd [
    "lockdown=confidentiality"
    "page_poison=1"
    "slub_debug=FZP"
  ];

  boot.kernel.sysctl."kernel.yama.ptrace_scope" = lib.mkDefault 1;
}
