{pkgs, ...}: {
  nixpkgs.config = {
    allowUnfree = true; # sadly.
    cudaSupport = true;
    cudaCapabilities = ["8.7"]; # NVIDIA Orin.
  };

  environment.systemPackages = with pkgs; [
    nvidia-jetpack.cudaPackages.cudatoolkit
    nvidia-jetpack.cudaPackages.cudatoolkit
    onnxruntime

    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
  ];

  hardware.graphics.enable = true;
}
