{pkgs, ...}:
pkgs.rustPlatform.buildRustPackage {
  pname = "ringil-workspace";
  version = "0.1.0";

  src = ../../../..;

  cargoLock = {
    lockFile = ../../../../Cargo.lock;
    allowBuiltinFetchGit = true;
    outputHashes = {};
  };

  nativeBuildInputs = with pkgs; [pkg-config protobuf rustPlatform.bindgenHook];
  buildInputs = with pkgs; [udev v4l-utils openssl];
}
