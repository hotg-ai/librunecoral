# librunecoral

A thinly veiled wrapper around tflite and libedgetpu from Google


## Building

#### Prerequisites

Linux/Android:
* docker installation [set up properly](https://docs.docker.com/get-started/)
* git

Windows 10:
* Visual Studio Build tools 2019
* Msys2
* choco install python llvm bazel
* pip install numpy
* rust

OS X:
* git
* bazel
* XCode
* pip install numpy
* rust

#### Getting the sources
```bash
$ git clone https://github.com/hotg-ai/librunecoral
$ cd librunecoral
$ git submodule update --init --recursive
```

#### Build the docker container
```bash
$ make docker-image-linux
$ docker image ls
REPOSITORY                      TAG     IMAGE ID       CREATED         SIZE
tinyverseml/runecoral-cross-linux-aarch64                   latest         349cd3de00b3   2 days ago      2.9GB
tinyverseml/runecoral-cross-linux-x86_64                    latest         4f5fe19abfb7   2 days ago      2.73GB
```

### Build the package for Linux
```bash
$ make librunecoral-linux-aarch64
$ ls dist/include
runecoral.h
$ ls dist/lib/linux/aarch64
librunecoral.so

# To build for all supported CPU architectures under linux
$ make librunecoral-linux
$ ls dist/lib/linux
aarch64  x86_64
```

### Build the package for / on Windows
```
$ bazel build --config windows //runecoral:runecoral
$ ls bazel-bin/runecoral/
_objs  runecoral.lib  runecoral.params
```
NOTE: On Windows you may need to clone librunecoral to C:\ or some such path in order to not run into Windows path length limitations

# Thanks to:
* Webcoral
* libedgetpu
* rust-embedded/cross (Especially for their docker container build scripts)
