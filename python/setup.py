from setuptools import setup, find_packages
import shutil
from pathlib import Path

# Copy compiled .so into package directory for distribution
so_source = Path(__file__).parent.parent / "target" / "release" / "libseed_core.so"
so_target = Path(__file__).parent / "seed_core" / "libseed_core.so"
if so_source.exists():
    shutil.copyfile(so_source, so_target)

setup(
    name="seed-core",
    version="0.1.5",
    description="Sub-microsecond deterministic 2-stage execution gatekeeper and AGI decider in native Rust.",
    author="Jarosław Kuchta",
    author_email="jarohull@gmail.com",
    url="https://github.com/jarohull-ai/seed-core",
    packages=find_packages(),
    package_data={"seed_core": ["libseed_core.so"]},
    include_package_data=True,
    python_requires=">=3.8",
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: Rust",
        "License :: Other/Proprietary License",
        "Operating System :: POSIX :: Linux",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Security",
    ],
)
