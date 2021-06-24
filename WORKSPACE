workspace(name = "runecoral")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

TENSORFLOW_COMMIT = "48c3bae94a8b324525b45f157d638dfd4e8c3be1"
# Command to calculate: curl -L <FILE-URL> | sha256sum | awk '{print $1}'
TENSORFLOW_SHA256 = "363420a67b4cfa271cd21e5c8fac0d7d91b18b02180671c3f943c887122499d8"

# These values come from the Tensorflow workspace. If the TF commit is updated,
# these should be updated to match.
IO_BAZEL_RULES_CLOSURE_COMMIT = "308b05b2419edb5c8ee0471b67a40403df940149"
IO_BAZEL_RULES_CLOSURE_SHA256 = "5b00383d08dd71f28503736db0500b6fb4dda47489ff5fc6bed42557c07c6ba9"

CORAL_CROSSTOOL_COMMIT = "142e930ac6bf1295ff3ba7ba2b5b6324dfb42839"
CORAL_CROSSTOOL_SHA256 = "088ef98b19a45d7224be13636487e3af57b1564880b67df7be8b3b7eee4a1bfc"

# Configure libedgetpu and downstream libraries (TF and Crosstool).
new_local_repository(
    name = "libedgetpu",
    path = "third_party/libcoral/libedgetpu",
    build_file = "third_party/libcoral/libedgetpu/BUILD"
)

load("@libedgetpu//:workspace.bzl", "libedgetpu_dependencies")
libedgetpu_dependencies(TENSORFLOW_COMMIT, TENSORFLOW_SHA256,
                        IO_BAZEL_RULES_CLOSURE_COMMIT,IO_BAZEL_RULES_CLOSURE_SHA256,
                        CORAL_CROSSTOOL_COMMIT,CORAL_CROSSTOOL_SHA256)

load("@org_tensorflow//tensorflow:workspace.bzl", "tf_workspace")
tf_workspace(tf_repo_name = "org_tensorflow")

load("@coral_crosstool//:configure.bzl", "cc_crosstool")
cc_crosstool(name = "crosstool")

http_archive(
    name = "com_google_glog",
    sha256 = "6fc352c434018b11ad312cd3b56be3597b4c6b88480f7bd4e18b3a3b2cf961aa",
    strip_prefix = "glog-3ba8976592274bc1f907c402ce22558011d6fc5e",
    urls = [
        "https://github.com/google/glog/archive/3ba8976592274bc1f907c402ce22558011d6fc5e.tar.gz",
    ],
    build_file_content = """
licenses(['notice'])
exports_files(['CMakeLists.txt'])
load(':bazel/glog.bzl', 'glog_library')
glog_library(with_gflags=0)
""",
)

new_local_repository(
    name = "libcoral",
    path = "third_party/libcoral",
    build_file = "third_party/libcoral/coral/BUILD"
)

new_local_repository(
    name = "glog",
    path = "third_party/libcoral/third_party/glog",
    build_file = "third_party/libcoral/third_party/glog/BUILD",
)
