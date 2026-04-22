{
  pkgs,
  lib,
  ...
}: {
  # Sadly, most Nvidia packages are not FOSS...
  nixpkgs.config.allowUnfree = true;

  environment.systemPackages = with pkgs; [
    cudaPackages.tensorrt
    cudaPackages.cudatoolkit
  ];

  environment.variables.LD_LIBRARY_PATH = "/run/opengl-driver/lib:${pkgs.cudaPackages.tensorrt}/lib:${pkgs.cudaPackages.cudatoolkit}/lib";
}
