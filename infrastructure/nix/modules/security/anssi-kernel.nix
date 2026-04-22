{
  lib,
  config,
  ...
}: let
  cfg = config.ringil.env;
  isProd = cfg.mode == "prod";
in {
  boot.kernel.sysctl = lib.mkIf isProd {
    "kernel.kptr_restrict" = "2";
    "kernel.dmesg_restrict" = "1";
    "kernel.unprivileged_bpf_disabled" = "1";
    "kernel.sysrq" = "0";
    "kernel.perf_event_paranoid" = "2";

    "kernel.modules_disabled" = "1";

    "net.ipv4.ip_forward" = "0";
    "net.ipv4.conf.all.accept_redirects" = "0";
    "net.ipv6.conf.all.disable_ipv6" = "1";
  };
}
