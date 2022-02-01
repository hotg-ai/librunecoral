workspace(name = "runecoral")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Configure libedgetpu and downstream libraries (TF and Crosstool).
new_local_repository(
    name = "libedgetpu",
    path = "third_party/libedgetpu",
    build_file = "third_party/libedgetpu/BUILD"
)

TENSORFLOW_COMMIT = "c256c071bb26e1e13b4666d1b3e229e110bc914a"
TENSORFLOW_SHA256 = "ff0df77ec72676d3260502dd19f34518ecd65bb9ead4f7dfdf8bd11cff8640e3"

load("@libedgetpu//:workspace.bzl", "libedgetpu_dependencies")
libedgetpu_dependencies(TENSORFLOW_COMMIT, TENSORFLOW_SHA256)

load("@org_tensorflow//tensorflow:workspace3.bzl", "tf_workspace3")
tf_workspace3()

load("@coral_crosstool//:configure.bzl", "cc_crosstool")
cc_crosstool(name = "crosstool")
load("@org_tensorflow//tensorflow:workspace2.bzl", "tf_workspace2")
tf_workspace2()

load("@org_tensorflow//tensorflow:workspace1.bzl", "tf_workspace1")
tf_workspace1()

load("@org_tensorflow//tensorflow:workspace0.bzl", "tf_workspace0")
tf_workspace0()


http_archive(
    name = "build_bazel_rules_apple",
    sha256 = "a5f00fd89eff67291f6cd3efdc8fad30f4727e6ebb90718f3f05bbf3c3dd5ed7",
    url = "https://github.com/bazelbuild/rules_apple/releases/download/0.19.0/rules_apple.0.33.0.tar.gz",
)

load(
    "@build_bazel_rules_apple//apple:repositories.bzl",
    "apple_rules_dependencies",
)

apple_rules_dependencies()

http_archive(
    name = "build_bazel_apple_support",
    sha256 = "c604b75865c07f45828c7fffd5fdbff81415a9e84a68f71a9c1d8e3b2694e547",
    urls = [
        "https://storage.googleapis.com/mirror.tensorflow.org/github.com/bazelbuild/apple_support/releases/download/0.12.1/apple_support.0.12.1.tar.gz",
        "https://github.com/bazelbuild/apple_support/releases/download/0.12.1/apple_support.0.12.1.tar.gz",
    ],
)

load(
    "@build_bazel_apple_support//lib:repositories.bzl",
    "apple_support_dependencies",
)

apple_support_dependencies()

# More iOS deps.

http_archive(
    name = "google_toolbox_for_mac",
    url = "https://github.com/google/google-toolbox-for-mac/archive/v2.3.2.zip",
    sha256 = "1554fa2d90f9005c2111b3ee2693df04787ce42439badb7d7878a442dab3953d",
    strip_prefix = "google-toolbox-for-mac-2.2.1",
    build_file = "@//third_party:google_toolbox_for_mac.BUILD",
)

android_ndk_repository(name = "androidndk")
