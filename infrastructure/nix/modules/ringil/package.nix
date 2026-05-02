{pkgs, ...}:
pkgs.rustPlatform.buildRustPackage {
  pname = "ringil-daemon";
  version = "0.1.0";

  src = ../../../..;

  cargoLock = {
    lockFile = ../../../../Cargo.lock;
    outputHashes = {
      "ort-2.0.0-rc.12" = "sha256-BptpN7BO5FVO1znc01YXuWIkLn1H5bSuJCDSJOdJNFg=";
      "ultralytics-inference-0.0.10" = "sha256-36wrMjKc6b2YazHY7PDXKfDuAtSTlc+O9cw0LCP8fuQ=";
      "orb-slam3-0.1.0" = "sha256-6CINKdSMuAdDqklfu0CXsALbOQKzvN5vi7xpg2690N4=";
    };
  };

  nativeBuildInputs = with pkgs; [pkg-config protobuf rustPlatform.bindgenHook];
  buildInputs = with pkgs; [udev v4l-utils openssl];
}
