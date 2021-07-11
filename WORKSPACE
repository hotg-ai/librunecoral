workspace(name = "runecoral")

# Configure libedgetpu and downstream libraries (TF and Crosstool).
new_local_repository(
    name = "libedgetpu",
    path = "third_party/libedgetpu",
    build_file = "third_party/libedgetpu/BUILD"
)

TENSORFLOW_COMMIT = "ee728bcc18abef50b968c4b4cb06523f64abf7ac"
TENSORFLOW_SHA256 = "d429e78426a08ddfe121514aa132b3e90051055e3a379676ade8058a266b726e"

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
