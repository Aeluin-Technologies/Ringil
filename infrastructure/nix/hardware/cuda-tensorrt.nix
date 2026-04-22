{pkgs, ...}: {
  environment.systemPackages = with pkgs; [
    tensorrt
    cudatoolkit
  ];

  environment.variables.LD_LIBRARY_PATH = "/run/opengl-driver/lib:${pkgs.tensorrt}/lib";
}
