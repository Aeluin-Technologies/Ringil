{
  lib,
  buildRosPackage,
  ament-python,
  launch,
  launch-ros,
}:
buildRosPackage {
  pname = "ringil_bringup";
  version = "0.0.1";

  src = ../../../../ros2/src/ringil_bringup;

  buildType = "ament_python";

  propagatedBuildInputs = [
    ament-python
    launch
    launch-ros
  ];

  meta = with lib; {
    description = "Launch Ringil";
    license = licenses.agpl3Only;
  };
}
