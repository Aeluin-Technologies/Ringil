{pkgs, ...}:
pkgs.rustPlatform.buildRustPackage {
  pname = "ringil-daemon";
  version = "0.1.0";

  src = ../../../..;

  cargoLock = {
    lockFile = ../../../../Cargo.lock;
    outputHashes = {
      "ort-2.0.0-rc.12" = "sha256-BptpN7BO5FVO1znc01YXuWIkLn1H5bSuJCDSJOdJNFg=";
      "ultralytics-inference-0.0.11" = "sha256-Lf4drYPdpw74nxq8h4GXVl+CenTwNf1l5I+pUcZIHWg=";
    };
  };

  nativeBuildInputs = with pkgs; [pkg-config protobuf rustPlatform.bindgenHook];
  buildInputs = with pkgs; [udev v4l-utils openssl];
}
