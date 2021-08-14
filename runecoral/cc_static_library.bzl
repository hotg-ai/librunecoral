# From https://github.com/bazelbuild/bazel/issues/1920
load("@bazel_tools//tools/cpp:toolchain_utils.bzl", "find_cpp_toolchain")

def _cc_static_library_impl(ctx):
    cc_deps = [dep[CcInfo] for dep in ctx.attr.deps]
    libraries = []
    for cc_dep in cc_deps:
        for link_input in cc_dep.linking_context.linker_inputs.to_list():
            for library in link_input.libraries:
                libraries += library.pic_objects
    args = ["r", ctx.outputs.out.path] + [f.path for f in libraries]

    cc_toolchain = find_cpp_toolchain(ctx)
    # TODO: Make sure this call works on Windows environment too
    ctx.actions.run(
        inputs = depset(libraries),
        outputs = [ctx.outputs.out],
        executable = cc_toolchain.ar_executable,
        arguments = args,
    )
    return [DefaultInfo()]

cc_static_library = rule(
    implementation = _cc_static_library_impl,
    attrs = {
        "_cc_toolchain": attr.label(
            default = Label("@bazel_tools//tools/cpp:current_cc_toolchain")
        ),
        "deps": attr.label_list(providers = [CcInfo]),
    },
    fragments = ["cpp"],
    outputs = {"out": "lib%{name}.a"},
)
