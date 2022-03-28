"""Provides a rule that outputs a monolithic static library."""

# Reference: https://gist.github.com/oquenchil/3f88a39876af2061f8aad6cdc9d7c045

load("@bazel_tools//tools/cpp:toolchain_utils.bzl", "find_cpp_toolchain")
load("@build_bazel_apple_support//lib:apple_support.bzl", "apple_support")
load("@bazel_skylib//lib:dicts.bzl","dicts")

TOOLS_CPP_REPO = "@bazel_tools"

def _cc_static_library_impl_win(ctx):
    cc_deps = [dep[CcInfo] for dep in ctx.attr.deps]
    libraries = []
    for cc_dep in cc_deps:
        for link_input in cc_dep.linking_context.linker_inputs.to_list():
            for library in link_input.libraries:
                if len(library.pic_objects) > 0:
                    libraries += library.pic_objects
                else:
                    libraries += library.objects

    # TODO: Also collect and append static libraries that are being linked like in the unixy versions
    cc_toolchain = find_cpp_toolchain(ctx)
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


def _cc_static_library_impl(ctx):
    # On Windows: Ugly workaround: https://github.com/bazelbuild/bazel/issues/9209
    if  ctx.configuration.host_path_separator == ";":
        return _cc_static_library_impl_win(ctx)

    # For Unixy platforms
    output_lib = ctx.actions.declare_file("lib{}.a".format(ctx.attr.name))
    output_flags = ctx.actions.declare_file("{}.link".format(ctx.attr.name))

    cc_toolchain = find_cpp_toolchain(ctx)

    # Aggregate linker inputs of all dependencies
    lib_sets = []
    for dep in ctx.attr.deps:
        lib_sets.append(dep[CcInfo].linking_context.linker_inputs)
    input_depset = depset(transitive = lib_sets)

    # Collect user link flags and make sure they are unique
    unique_flags = {}
    for inp in input_depset.to_list():
        unique_flags.update({
            flag: None
            for flag in inp.user_link_flags
        })
    link_flags = unique_flags.keys()

    # Collect static libraries
    libs = []
    for inp in input_depset.to_list():
        for lib in inp.libraries:
            if lib.pic_static_library:
                libs.append(lib.pic_static_library)
            elif lib.static_library:
                libs.append(lib.static_library)

    ctx.actions.write(
        output = output_flags,
        content = "\n".join(link_flags) + "\n",
    )
    ar_path = cc_toolchain.ar_executable
    libs_string = " ".join([lib.path for lib in libs])
    # OS X wtf
    print(ar_path)
    if ar_path.find("libtool", 0) != -1:
        apple_support.run(
            ctx,
            executable = ar_path,
            arguments = ["-static", "-o", output_lib.path] + [lib.path for lib in libs],
            inputs = libs + cc_toolchain.all_files.to_list(),
            outputs = [output_lib],
            mnemonic = "LibtoolMerge",
        )
    else:
        script_file = ctx.actions.declare_file("{}.mri".format(ctx.attr.name))
        commands = ["create {}".format(output_lib.path)]
        for lib in libs:
            commands.append("addlib {}".format(lib.path))
        commands.append("save")
        commands.append("end")
        ctx.actions.write(
            output = script_file,
            content = "\r\n".join(commands) + "\r\n",
        )
        ctx.actions.run_shell(
            command = "\"{}\" -M < {}".format(ar_path, script_file.path),
            inputs = [script_file] + libs + cc_toolchain.all_files.to_list(),
            outputs = [output_lib],
            mnemonic = "ArMerge",
            progress_message = "Merging static library {}".format(output_lib.path),
        )
    return [ DefaultInfo(files = depset([output_flags, output_lib])) ]

cc_static_library = rule(
    implementation = _cc_static_library_impl,
    attrs = dicts.add(apple_support.action_required_attrs(), {
        "deps": attr.label_list(),
        "_cc_toolchain": attr.label(
            default = TOOLS_CPP_REPO + "//tools/cpp:current_cc_toolchain",
        ),
    }),
    toolchains = [TOOLS_CPP_REPO + "//tools/cpp:toolchain_type"],
    incompatible_use_toolchain_transition = True,
    fragments = ["apple"]
)

