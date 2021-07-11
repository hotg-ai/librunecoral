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
    ],
)