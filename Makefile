SHELL := /bin/bash
MAKEFILE_DIR := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
OS := $(shell uname -s)

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

BAZEL_OUT_DIR :=  $(MAKEFILE_DIR)/bazel-out/$(CPU)-$(COMPILATION_MODE)/bin
COMMON_BAZEL_BUILD_FLAGS_Linux := --crosstool_top=@crosstool//:toolchains \
                                  --compiler=gcc
COMMON_BAZEL_BUILD_FLAGS_Darwin :=
COMMON_BAZEL_BUILD_FLAGS := --compilation_mode=$(COMPILATION_MODE) \
                            --copt=-DNPY_NO_DEPRECATED_API=NPY_1_7_API_VERSION \
                            --verbose_failures \
                            --sandbox_debug \
                            --subcommands \
                            --cpu=$(CPU) \
                            --experimental_repo_remote_exec \
                            $(COMMON_BAZEL_BUILD_FLAGS_$(OS))

ifeq ($(COMPILATION_MODE), opt)
BAZEL_BUILD_FLAGS_Linux += --linkopt=-Wl,--strip-all
endif


ifeq ($(CPU),k8)
RUNE_CORAL_SUFFIX := x86_64-linux-gnu.so
RUNE_CORAL_DIST_PLATFORM := linux_x86_64
else ifeq ($(CPU),aarch64)
BAZEL_BUILD_FLAGS_Linux += --copt=-ffp-contract=off
RUNE_CORAL_WRAPPER_SUFFIX := aarch64-linux-gnu.so
RUNE_CORAL_DIST_PLATFORM := linux_aarch64
else ifeq ($(CPU),armv7a)
BAZEL_BUILD_FLAGS_Linux += --copt=-ffp-contract=off
RUNE_CORAL_WRAPPER_SUFFIX := arm-linux-gnueabihf.so
RUNE_CORAL_DIST_PLATFORM := linux-armv7l
else ifeq ($(CPU), darwin)
RUNE_CORAL_WRAPPER_SUFFIX := darwin.so
endif

BAZEL_BUILD_FLAGS := $(COMMON_BAZEL_BUILD_FLAGS) \
                     $(BAZEL_BUILD_FLAGS_$(OS))
.PHONY: all \
        runecoral \
        clean \
        help

all: runecoral

runecoral:
	bazel build $(BAZEL_BUILD_FLAGS) \
	    --stamp \
	    //runecoral:runecoral

clean:
	rm -rf $(MAKEFILE_DIR)/bazel-* \
	       $(MAKEFILE_DIR)/build \
	       $(MAKEFILE_DIR)/dist

help:
	@echo "make all          - Build all native code"
	@echo "make runecoral       - Build pycoral native code"
	@echo "make clean        - Remove generated files"
	@echo "make help         - Print help message"

TEST_ENV := $(shell test -L $(MAKEFILE_DIR)/test_data && echo 1)
DOCKER_WORKSPACE := $(MAKEFILE_DIR)/$(if $(TEST_ENV),..,)
DOCKER_WORKSPACE_CD := $(if $(TEST_ENV),pycoral,)
DOCKER_CPUS := k8 armv7a aarch64
DOCKER_TAG_BASE := coral-edgetpu
include $(MAKEFILE_DIR)/third_party/libcoral/docker/docker.mk
