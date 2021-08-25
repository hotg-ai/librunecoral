SHELL := /bin/bash
MAKEFILE_DIR := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
OS := $(shell uname -s)
DOCKER_IMAGE_LINUX := tinyverseml/runecoral-cross-linux
DOCKER_IMAGE_ANDROID := tinyverseml/runecoral-cross-android

# Allowed COMPILATION_MODE values: opt, dbg, fastbuild
COMPILATION_MODE ?= opt
ifeq ($(filter $(COMPILATION_MODE),opt dbg fastbuild),)
$(error COMPILATION_MODE must be opt, dbg or fastbuild)
endif

ifeq ($(COMPILATION_MODE), opt)
BAZEL_BUILD_FLAGS += --linkopt=-Wl,--strip-all
endif

.PHONY: all \
        clean \
        help

all: dist

dist: runecoral_header librunecoral-linux librunecoral-android

runecoral_header: runecoral/runecoral.h
	mkdir -p $(MAKEFILE_DIR)/dist/include
	install runecoral/runecoral.h $(MAKEFILE_DIR)/dist/include

librunecoral-linux-%: runecoral/runecoral.h runecoral/runecoral.cpp runecoral/private/accelerationbackends.h runecoral/private/utils.h
	docker run --rm -it -v "`pwd`":"`pwd`" \
           -v $$HOME:$$HOME \
		   -v /etc/group:/etc/group:ro \
           -v /etc/passwd:/etc/passwd:ro \
           -v /etc/localtime:/etc/localtime:ro \
           -u `id -u $$USER`:`id -g $$USER` \
           -e HOME=$$HOME \
           -e USER=$$USER \
           -w "`pwd`" \
			$(DOCKER_IMAGE_LINUX) \
			bazel build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=linux_$* //runecoral:runecoral
	mkdir -p dist/lib/linux/$*/
	install bazel-bin/runecoral/librunecoral.a dist/lib/linux/$*

librunecoral-android-%: runecoral/runecoral.h runecoral/runecoral.cpp runecoral/private/accelerationbackends.h runecoral/private/utils.h
	docker run --rm -it -v "`pwd`":"`pwd`" \
           -v $$HOME:$$HOME \
		   -v /etc/group:/etc/group:ro \
           -v /etc/passwd:/etc/passwd:ro \
           -v /etc/localtime:/etc/localtime:ro \
           -u `id -u $$USER`:`id -g $$USER` \
           -e HOME=$$HOME \
           -e USER=$$USER \
           -w "`pwd`" \
			$(DOCKER_IMAGE_ANDROID) \
			bazel build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=android_$* //runecoral:runecoral
	mkdir -p dist/lib/android/$*/ ;
	install bazel-bin/runecoral/librunecoral.a dist/lib/android/$*

librunecoral-linux: librunecoral-linux-arm librunecoral-linux-arm64 librunecoral-linux-x86_64
librunecoral-android: librunecoral-android-arm librunecoral-android-arm64 librunecoral-android-x86

docker-image-linux:
	docker build $(DOCKER_IMAGE_OPTIONS) -t $(DOCKER_IMAGE_LINUX) -f $(MAKEFILE_DIR)/docker/Dockerfile.Linux $(MAKEFILE_DIR)/docker

docker-image-android:
	docker build $(DOCKER_IMAGE_OPTIONS) -t $(DOCKER_IMAGE_ANDROID) -f $(MAKEFILE_DIR)/docker/Dockerfile.Android $(MAKEFILE_DIR)/docker

clean:
	rm -rf $(MAKEFILE_DIR)/bazel-* \
	       $(MAKEFILE_DIR)/build \
	       $(MAKEFILE_DIR)/dist

help:
	@echo "make all                   - Build all native code"
	@echo "make librunecoral-linux    - Build native code"
	@echo "make clean                 - Remove generated files"
	@echo "make help                  - Print help message"
