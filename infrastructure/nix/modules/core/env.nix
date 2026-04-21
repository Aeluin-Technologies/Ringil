{ lib, ... }:
{
  options.ringil.env.mode = lib.mkOption {
    type = lib.types.enum [ "dev" "prod" ];
    default = "prod";
    description = ''
      Defines how drone operates.
      * 'prod', locks down the system (no SSH, no logs, full encryption).
      * 'dev', enables debugging access (SSH, persistent journald).
    '';
  };
}
