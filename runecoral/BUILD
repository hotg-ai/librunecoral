package(
    default_visibility = ["//visibility:public"],
)

cc_library(
    name = "runecoral",
    srcs = [
        "runecoral.cpp",
    ],
    hdrs = ["runecoral.h"],
    data = select({
        "//conditions:default": [],
    }),
    linkopts = select({
        "//conditions:default": [],
    }),
    deps = [
        "@com_google_absl//absl/flags:flag",
        "@com_google_absl//absl/flags:parse",
        "@libcoral//coral:tflite_utils",
        "@libcoral//coral/classification:adapter",
        "@libedgetpu//tflite/public:edgetpu",
        "@org_tensorflow//tensorflow/lite:stateful_error_reporter",
        "@org_tensorflow//tensorflow/lite/c:common",
        "@glog"
    ],
)
