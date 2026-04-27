{pkgs, ...}:
pkgs.rustPlatform.buildRustPackage {
  pname = "ringil-daemon";
  version = "0.1.0";

  src = ../../../..;

  cargoLock = {
    lockFile = ../../../../Cargo.lock;
    outputHashes = {
      "ort-2.0.0-rc.12" = "sha256-BptpN7BO5FVO1znc01YXuWIkLn1H5bSuJCDSJOdJNFg=";
      "ultralytics-inference-0.0.10" = "sha256-7P6v/ZjJ3SKq/2YqefgClmzIx2NAyOf5tilPFwHAnpo=";
      "orb-slam3-0.1.0" = "sha256-6CINKdSMuAdDqklfu0CXsALbOQKzvN5vi7xpg2690N4=";
    };
  };

  nativeBuildInputs = with pkgs; [pkg-config protobuf rustPlatform.bindgenHook];
  buildInputs = with pkgs; [udev v4l-utils openssl];
}
