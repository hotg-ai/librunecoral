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
$ make docker-image
$ docker image ls
REPOSITORY                      TAG     IMAGE ID       CREATED         SIZE
runecoral-cross-debian-stretch     latest  b431b6fa5895   7 hours ago     2.94GB

```

### Enter the docker container to do the build
```bash
$ docker run --rm -it -v $PWD:$PWD \
               -v $HOME:$HOME \
               -v /etc/group:/etc/group:ro \
               -v /etc/passwd:/etc/passwd:ro \
               -v /etc/localtime:/etc/localtime:ro \
               -u $(id -u ${USER}):$(id -g ${USER}) \
               -e HOME=$HOME \
               -e USER=$USER runecoral-cross-debian-stretch
user@39b50cb9fe24:/src/librunecoral$ CPU=aarch64 make
user@39b50cb9fe24:/src/librunecoral$ ls dist/include
runecoral.h
user@39b50cb9fe24:/src/librunecoral$ ls dist/lib/linux/aarch64
librunecoral.so
```

# Thanks to:
* Webcoral
* libedgetpu
