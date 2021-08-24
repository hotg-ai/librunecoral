load("cc_static_library.bzl", "cc_static_library")
package(
    default_visibility = ["//visibility:public"],
)

cc_binary(
    name = "librunecoral.so",
    srcs = [
      "runecoral.cpp",
      "runecoral.h",
    ],
    # hdrs = ["runecoral.h"],
    linkopts = select({
        "//conditions:default": ["-shared"],
    }),
   deps = [
       "@libedgetpu//tflite/public:edgetpu_c",
       "@libedgetpu//tflite/public:oss_edgetpu_direct_pci",
       "@org_tensorflow//tensorflow/lite:framework",
       "@org_tensorflow//tensorflow/lite/kernels:builtin_ops",  # BuiltinOpResolver
       "@org_tensorflow//tensorflow/lite/delegates/gpu:delegate"
    ]
)

cc_library(
    name = "_runecoralstatic",
    srcs = [
      "runecoral.cpp",
      "runecoral.h",
      "private/utils.h",
      "private/accelerationbackends.h",
    ],
    # hdrs = ["runecoral.h"],
   deps = [
       "@libedgetpu//tflite/public:edgetpu_c",
       "@libedgetpu//tflite/public:oss_edgetpu_direct_pci",
       "@org_tensorflow//tensorflow/lite:framework",
       "@org_tensorflow//tensorflow/lite/kernels:builtin_ops",  # BuiltinOpResolver
       "@org_tensorflow//tensorflow/lite/delegates/gpu:delegate",
    ],
    local_defines = [
      "RUNECORAL_EDGETPU_BACKEND",
      "RUNECORAL_GPU_BACKEND",
    ]
)

cc_static_library(
    name = "runecoral",
    deps = [
      "_runecoralstatic"
    ]
)

