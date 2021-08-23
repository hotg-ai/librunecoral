SHELL := /bin/bash
MAKEFILE_DIR := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
OS := $(shell uname -s)
DOCKER_IMAGE_LINUX := tinyverseml/runecoral-cross-debian-stretch
DOCKER_IMAGE_ANDROID := tinyverseml/runecoral-cross-android

# Allowed CPU values: k8, armv7a, aarch64, darwin
ifeq ($(OS),Linux)
CPU ?= k8
else ifeq ($(OS),Darwin)
CPU ?= darwin
else
$(error $(OS) is not supported)
endif
ifeq ($(filter $(CPU),k8 armv7a aarch64 darwin),)
$(error CPU must be k8, armv7a, aarch64, or darwin)
endif

# Allowed COMPILATION_MODE values: opt, dbg, fastbuild
COMPILATION_MODE ?= opt
ifeq ($(filter $(COMPILATION_MODE),opt dbg fastbuild),)
$(error COMPILATION_MODE must be opt, dbg or fastbuild)
endif

COMMON_BAZEL_BUILD_FLAGS_Linux := --crosstool_top=@crosstool//:toolchains \
                                  --compiler=gcc
COMMON_BAZEL_BUILD_FLAGS_Darwin :=
COMMON_BAZEL_BUILD_FLAGS := --compilation_mode=$(COMPILATION_MODE) \
                            --verbose_failures \
                            --sandbox_debug \
                            --subcommands \
                            --define darwinn_portable=1 \
                            --cpu=$(CPU) \
                            --experimental_repo_remote_exec \
                            --force_pic \
                            $(COMMON_BAZEL_BUILD_FLAGS_$(OS))

ifeq ($(COMPILATION_MODE), opt)
BAZEL_BUILD_FLAGS_Linux += --linkopt=-Wl,--strip-all
endif


ifeq ($(CPU),k8)
RUNE_CORAL_DIST_DIR := $(MAKEFILE_DIR)/dist/lib/linux/x86_64
else ifeq ($(CPU),aarch64)
BAZEL_BUILD_FLAGS_Linux += --copt=-ffp-contract=off
RUNE_CORAL_DIST_DIR := $(MAKEFILE_DIR)/dist/lib/linux/aarch64
else ifeq ($(CPU),armv7a)
BAZEL_BUILD_FLAGS_Linux += --copt=-ffp-contract=off
RUNE_CORAL_DIST_DIR := $(MAKEFILE_DIR)/dist/lib/linux/armv7l
else ifeq ($(CPU), darwin)
RUNE_CORAL_DIST_DIR := $(MAKEFILE_DIR)/dist/lib/darwin
endif

BAZEL_BUILD_FLAGS := $(COMMON_BAZEL_BUILD_FLAGS) \
                     $(BAZEL_BUILD_FLAGS_$(OS))
.PHONY: all \
        runecoral \
        clean \
        help

all: dist

dist: runecoral_header librunecoral-linux

runecoral_header: runecoral/runecoral.h
	mkdir -p $(MAKEFILE_DIR)/dist/include
	install runecoral/runecoral.h $(MAKEFILE_DIR)/dist/include

librunecoral-linux: runecoral/runecoral.cpp
	bazel build $(BAZEL_BUILD_FLAGS) //runecoral:runecoral
	mkdir -p $(RUNE_CORAL_DIST_DIR)/

librunecoral-android: runecoral/runecoral.cpp
	bazel build --config android_arm64 //runecoral:runecoral

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
