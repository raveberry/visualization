## Test
```
cargo run
```

## Crosscompile for the Pi

### Setup
```
# install crosscompiling for raspberry pi
sudo apt-get install arm-linux-gnueabihf-gcc
# add target
rustup target add armv7-unknown-linux-gnueabihf
# copy python sysconfig (required for pyo3 cross compilation)
mkdir lib
scp raveberry.local:/usr/lib/python3.9/_sysconfigdata__arm-linux-gnueabihf.py lib/
export PYO3_CROSS_LIB_DIR=$(pwd)/lib
# for building wheels
pip install maturin
# copy shaders and images to the pi
scp -r raveberry_visualization raveberry.local:
```

### Build
Binary:
```
# --release to reduce binary size, speeding up scp
cargo build --target armv7-unknown-linux-gnueabihf --release
scp target/armv7-unknown-linux-gnueabihf/release/raveberry-visualization raveberry.local:
```
Wheel:
```
# optional: add --release --strip
maturin build --target armv7-unknown-linux-gnueabihf
scp target/wheels/*.whl raveberry.local:
```

## Build

```
sudo apt-get install python3.8 python3.8-venv python3.9 python3.9-venv python3.10 python3.10-venv
for python in python3.8 python3.9 python3.10; do
	rm -rf .venv
	$python -m venv .venv
	. .venv/bin/activate
	pip install setuptools-rust wheel
	# plat-name obtained via "auditwheel show <wheel_file>"
	python setup.py bdist_wheel --plat-name manylinux_2_27_x86_64
done
# on the pi
pip install setuptools-rust wheel
python setup.py sdist bdist_wheel --plat-name manylinux_2_28_armv7l
# upload
twine upload dist/*
```
