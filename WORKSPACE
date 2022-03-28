workspace(name = "runecoral")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Configure libedgetpu and downstream libraries (TF and Crosstool).
new_local_repository(
    name = "libedgetpu",
    path = "third_party/libedgetpu",
    build_file = "third_party/libedgetpu/BUILD"
)

TENSORFLOW_COMMIT = "3f878cff5b698b82eea85db2b60d65a2e320850e"
TENSORFLOW_SHA256 = "21d919ad6d96fcc0477c8d4f7b1f7e4295aaec2986e035551ed263c2b1cd52ee"

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

git_repository(
    name = "build_bazel_rules_apple",
    remote = "https://github.com/bazelbuild/rules_apple.git",
    tag = "0.34.0",
)

load(
    "@build_bazel_rules_apple//apple:repositories.bzl",
    "apple_rules_dependencies",
)

apple_rules_dependencies()

git_repository(
    name = "build_bazel_apple_support",
    remote = "https://github.com/bazelbuild/apple_support.git",
    tag = "0.13.0",
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
