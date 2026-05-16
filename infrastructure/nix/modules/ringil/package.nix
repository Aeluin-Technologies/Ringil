{pkgs, ...}:
pkgs.rustPlatform.buildRustPackage {
  pname = "ringil-daemon";
  version = "0.1.0";

  src = ../../../..;

  cargoLock = {
    lockFile = ../../../../Cargo.lock;
    outputHashes = {};
  };

  nativeBuildInputs = with pkgs; [pkg-config protobuf rustPlatform.bindgenHook];
  buildInputs = with pkgs; [udev v4l-utils openssl];
}
