SHELL := /bin/bash
MAKEFILE_DIR := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
PREFIX ?= $(MAKEFILE_DIR)

DOCKER_IMAGE_LINUX := tinyverseml/runecoral-cross-linux
DOCKER_IMAGE_ANDROID := tinyverseml/runecoral-cross-android
DOCKER_RUN := docker run -i --rm -v "`pwd`":"`pwd`" \
           -v $$HOME:$$HOME \
           -v /etc/group:/etc/group:ro \
           -v /etc/passwd:/etc/passwd:ro \
           -v /etc/localtime:/etc/localtime:ro \
           -u `id -u $$USER`:`id -g $$USER` \
           -e HOME=$$HOME \
           -e USER=$$USER \
           -w "`pwd`"

# Allowed COMPILATION_MODE values: opt, dbg, fastbuild
COMPILATION_MODE ?= opt
ifeq ($(filter $(COMPILATION_MODE),opt dbg fastbuild),)
$(error COMPILATION_MODE must be opt, dbg or fastbuild)
endif

BAZEL ?= bazel

ifeq ($(COMPILATION_MODE), opt)
BAZEL_BUILD_FLAGS += --linkopt=-Wl,--strip-all
else
# From tensorflow's bazelrc
# Workaround for: https://github.com/tensorflow/tensorflow/issues/33360
BAZEL_BUILD_FLAGS += -c dbg --cxxopt -DTF_LITE_DISABLE_X86_NEON --copt -DDEBUG_BUILD
endif

EDGETPU_ACCELERATION ?= false
GPU_ACCELERATION ?= false

ifeq ($(EDGETPU_ACCELERATION), true)
BAZEL_BUILD_FLAGS += --define edgetpu_acceleration=true
endif

ifeq ($(GPU_ACCELERATION), true)
BAZEL_BUILD_FLAGS += --define gpu_acceleration=true
endif

SOURCES = $(MAKEFILE_DIR)/runecoral/runecoral.h \
	  $(MAKEFILE_DIR)/runecoral/private/accelerationbackends.h \
	  $(MAKEFILE_DIR)/runecoral/private/utils.h \
	  $(MAKEFILE_DIR)/runecoral/runecoral.cpp

.PHONY: all \
        clean \
        help
all: dist

dist: runecoral_header librunecoral-linux librunecoral-android

runecoral_header: $(MAKEFILE_DIR)/runecoral/runecoral.h
	mkdir -p $(PREFIX)/dist/include
	install $(MAKEFILE_DIR)/runecoral/runecoral.h $(PREFIX)/dist/include

librunecoral-linux-%: $(SOURCES)
	if [ "$$RUNECORAL_BUILD_ENVIRONMENT" = true ]; then\
		cat /etc/passwd ;\
		$(BAZEL) build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=linux_$* //runecoral:runecoral ;\
	else \
		$(DOCKER_RUN) $(DOCKER_IMAGE_LINUX)-$* $(BAZEL) build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=linux_$* //runecoral:runecoral ;\
	fi
	mkdir -p $(PREFIX)/dist/lib/linux/$*/
	install $(MAKEFILE_DIR)/bazel-bin/runecoral/librunecoral.a $(PREFIX)/dist/lib/linux/$*

librunecoral-android-%: $(SOURCES)
	if [ "$$RUNECORAL_BUILD_ENVIRONMENT" = true ]; then\
		$(BAZEL) build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=android_$* //runecoral:runecoral ;\
	else \
		$(DOCKER_RUN) $(DOCKER_IMAGE_ANDROID)-$* $(BAZEL) build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=android_$* //runecoral:runecoral ;\
	fi
	mkdir -p $(PREFIX)/dist/lib/android/$*/
	install $(MAKEFILE_DIR)/bazel-bin/runecoral/librunecoral.a $(PREFIX)/dist/lib/android/$*

librunecoral-macos-%: $(SOURCES)
	mkdir -p $(PREFIX)/dist/lib/macos/$*/
	bazel build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=darwin_$* //runecoral:runecoral
	install $(MAKEFILE_DIR)/bazel-bin/runecoral/librunecoral.a $(PREFIX)/dist/lib/macos/$*

librunecoral-ios-%: $(SOURCES)
	mkdir -p $(PREFIX)/dist/lib/ios/$*/
	bazel build -c $(COMPILATION_MODE) $(BAZEL_BUILD_FLAGS) --config=ios_$* //runecoral:runecoral
	install $(MAKEFILE_DIR)/bazel-bin/runecoral/librunecoral.a $(PREFIX)/dist/lib/ios/$*

librunecoral-linux: librunecoral-linux-arm librunecoral-linux-aarch64 librunecoral-linux-x86_64
librunecoral-android: librunecoral-android-arm librunecoral-android-aarch64 librunecoral-android-x86
librunecoral-apple: librunecoral-macos-x86_64 librunecoral-ios-aarch64

docker-image-linux-%:
	docker build $(DOCKER_IMAGE_OPTIONS) -t $(DOCKER_IMAGE_LINUX)-$* -f $(MAKEFILE_DIR)/docker/cross.$*-unknown-linux-gnu.Dockerfile $(MAKEFILE_DIR)/docker
docker-image-android-%:
	docker build $(DOCKER_IMAGE_OPTIONS) -t $(DOCKER_IMAGE_ANDROID)-$* -f $(MAKEFILE_DIR)/docker/cross.$*-linux-android.Dockerfile $(MAKEFILE_DIR)/docker

docker-image-linux: docker-image-linux-x86_64 docker-image-linux-aarch64
docker-image-android: docker-image-android-x86_64 docker-image-android-aarch64

clean:
	rm -rf $(MAKEFILE_DIR)/bazel-* \
	       $(MAKEFILE_DIR)/build \
	       $(MAKEFILE_DIR)/dist

help:
	@echo "make all                   - Build all native code"
	@echo "make librunecoral-linux    - Build native code"
	@echo "make clean                 - Remove generated files"
	@echo "make help                  - Print help message"

