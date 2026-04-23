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
    onnxruntime-cuda

    pkg-config
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
  ];

  environment.variables = {
    ORT_STRATEGY = "system";
    ONNXRUNTIME_LIB_PATH = "${pkgs.onnxruntime-cuda}/lib";
    PKG_CONFIG_PATH = lib.makeSearchPath "lib/pkgconfig" [
      pkgs.gst_all_1.gstreamer.dev
      pkgs.gst_all_1.gst-plugins-base
    ];
    LD_LIBRARY_PATH = [
      "/run/opengl-driver/lib"
      "${pkgs.cudaPackages.tensorrt}/lib"
      "${pkgs.cudaPackages.cudatoolkit}/lib"
      "${pkgs.onnxruntime-cuda}/lib"
    ];
  };
}
