# librunecoral

A thinly veiled wrapper around tflite and libedgetpu from Google


## Building

#### Prerequisites
* docker installation [set up properly](https://docs.docker.com/get-started/)
* git

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
docker.pkg.github.com/hotg-ai/librunecoral/runecoral-cross-linux     latest  b431b6fa5895   7 hours ago     2.94GB

```

### Build the package
```bash
$ make librunecoral-linux-aarch64
$ ls dist/include
runecoral.h
$ ls dist/lib/linux/aarch64
librunecoral.so

# To build for all supported CPU architectures under linux
$ make librunecoral-linux
$ ls dist/lib/linux
arm  arm64  x86_64
```

# Thanks to:
* Webcoral
* libedgetpu
