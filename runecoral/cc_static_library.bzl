# From https://github.com/bazelbuild/bazel/issues/1920
load("@bazel_tools//tools/cpp:toolchain_utils.bzl", "find_cpp_toolchain")

def _cc_static_library_impl(ctx):
    cc_deps = [dep[CcInfo] for dep in ctx.attr.deps]
    libraries = []
    for cc_dep in cc_deps:
        for link_input in cc_dep.linking_context.linker_inputs.to_list():
            for library in link_input.libraries:
                libraries += library.objects

    cc_toolchain = find_cpp_toolchain(ctx)
    # On Windows: Ugly workaround: https://github.com/bazelbuild/bazel/issues/9209
    if  ctx.configuration.host_path_separator == ";":
        params_file = ctx.actions.declare_file(ctx.label.name + ".params")
        out_file = ctx.actions.declare_file(ctx.label.name + ".lib")
        ctx.actions.write(output = params_file, content = "\r\n".join([f.path for f in libraries]))
        args = ["/NOLOGO", "/LTCG", "/MACHINE:X64",
                    "/OUT:{}".format(out_file.path)] + ["@" + params_file.path]
        ctx.actions.run(
            inputs = depset(libraries),
            outputs = [out_file],
            executable = cc_toolchain.ar_executable,
            arguments = args,
        )
        return [DefaultInfo(files = depset([params_file, out_file]))]
    else:
        out_file = ctx.actions.declare_file("lib" + ctx.label.name + ".a")
        args = ["r", out_file.path] + [f.path for f in libraries]
        ctx.actions.run(
            inputs = depset(libraries),
            outputs = [out_file],
            executable = cc_toolchain.ar_executable,
            arguments = args,
        )
        return [DefaultInfo(files = depset([out_file]))]

cc_static_library = rule(
    implementation = _cc_static_library_impl,
    attrs = {
        "_cc_toolchain": attr.label(
            default = Label("@bazel_tools//tools/cpp:current_cc_toolchain")
        ),
        "deps": attr.label_list(providers = [CcInfo]),
    },
    fragments = ["cpp"]
)
