from setuptools import setup
from setuptools_rust import Binding, RustExtension, Strip

version = None
with open("Cargo.toml") as f:
    for line in f.readlines():
        if line.startswith("version = "):
            version = line.split()[-1].strip('"')

with open("README.md") as f:
    long_description = f.read()

setup(
    name="raveberry-visualization",
    version=version,
    author="Jonathan Hacker",
    author_email="raveberry@jhacker.de",
    description="TODO",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/raveberry/visualization",
    classifiers=[
        "Development Status :: 4 - Beta",
        "License :: OSI Approved :: GNU Lesser General Public License v3 (LGPLv3)",
        "Programming Language :: Python :: 3",
    ],
    packages=["raveberry_visualization"],
    include_package_data=True,
    python_requires=">=3.8",
    rust_extensions=[
        RustExtension(
            "raveberry_visualization.raveberry_visualization",
            binding=Binding.PyO3,
            strip=Strip.All,
        )
    ],
    setup_requires=["setuptools-rust>=0.12.1"],
    zip_safe=False,
)
