from setuptools import find_packages, setup
import os
from glob import glob

package_name = "ringil_bringup" 

setup(
    name=package_name,
    version="0.0.1",
    packages=find_packages(exclude=["test"]),
    data_files=[
        ("share/ament_index/resource_index/packages", ["resource/" + package_name]),
        ("share/" + package_name, ["package.xml"]),
        (os.path.join("share", package_name, "launch"), glob("launch/*.launch.py")),
        (os.path.join("share", package_name, "config"), glob("config/*.yaml")),
    ],
    install_requires=["setuptools"],
    zip_safe=True,
    maintainer="Aeluin Technologies",
    maintainer_email="aeluin@gravitalia.com",
    description="Launch Ringil (VSLAM, RTAB-Map, nvblox and EgoPlanner)",
    license="AGPL3",
    tests_require=["pytest"],
    entry_points={
        "console_scripts": [],
    },
)
