load("cc_static_library.bzl", "cc_static_library")

package(
    default_visibility = ["//visibility:public"],
)

cc_binary(
    name = "librunecoral.so",
    srcs = [
      "runecoral.cpp",
      "runecoral.h",
      "private/utils.h",
      "private/accelerationbackends.h",
    ],
    # hdrs = ["runecoral.h"],
    linkopts = select({
        "//conditions:default": ["-shared"],
    }),
   deps = [
       "@org_tensorflow//tensorflow/lite:framework",
       "@org_tensorflow//tensorflow/lite/kernels:builtin_ops",
    ]
    + select({
        ":edgetpu_acceleration": [
          "@libedgetpu//tflite/public:edgetpu_c",
          "@libedgetpu//tflite/public:oss_edgetpu_direct_pci",
        ],
        "//conditions:default": []
    })
    + select({
      ":gpu_acceleration": ["@org_tensorflow//tensorflow/lite/delegates/gpu:delegate"],
      "//conditions:default": []
    }),
    local_defines = select({
      ":edgetpu_acceleration" : ["RUNECORAL_EDGETPU_ACCELERATION"],
      "//conditions:default": []
    })
    +
    select({
      ":gpu_acceleration" : ["RUNECORAL_GPU_ACCELERATION"],
      "//conditions:default": []
    })
)

cc_library(
    name = "_runecoralstatic",
    srcs = [
      "runecoral.cpp",
      "runecoral.h",
      "private/utils.h",
      "private/accelerationbackends.h",
    ],
   deps = [
       "@org_tensorflow//tensorflow/lite:framework",
       "@org_tensorflow//tensorflow/lite/kernels:builtin_ops",
    ]
    + select({
        ":edgetpu_acceleration": [
          "@libedgetpu//tflite/public:edgetpu_c",
          "@libedgetpu//tflite/public:oss_edgetpu_direct_pci",
        ],
        "//conditions:default": []
    })
    + select({
      ":gpu_acceleration": ["@org_tensorflow//tensorflow/lite/delegates/gpu:delegate"],
      "//conditions:default": []
    }),
    local_defines = select({
      ":edgetpu_acceleration" : ["RUNECORAL_EDGETPU_ACCELERATION"],
      "//conditions:default": []
    })
    +
    select({
      ":gpu_acceleration" : ["RUNECORAL_GPU_ACCELERATION"],
      "//conditions:default": []
    })
)

cc_static_library(
    name = "runecoral",
    deps = [
      "_runecoralstatic"
    ]
)

# TODO: Make this more user friendly using bool_flag ?
config_setting(
  name = "edgetpu_acceleration",
  values = {
    "define": "edgetpu_acceleration=true"
  }
)

config_setting(
  name = "gpu_acceleration",
  values = {
    "define": "gpu_acceleration=true"
  }
)