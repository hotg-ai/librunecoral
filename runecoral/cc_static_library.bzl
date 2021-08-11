# From https://github.com/bazelbuild/bazel/issues/1920

def _cc_static_library_impl(ctx):
    cc_deps = [dep[CcInfo] for dep in ctx.attr.deps]
    libraries = []
    for cc_dep in cc_deps:
        for link_input in cc_dep.linking_context.linker_inputs.to_list():
            for library in link_input.libraries:
                libraries += library.pic_objects
    args = ["r", ctx.outputs.out.path] + [f.path for f in libraries]

    ctx.actions.run(
        inputs = depset(libraries),
        outputs = [ctx.outputs.out],
        executable = "/usr/bin/ar",
        arguments = args,
    )
    return [DefaultInfo()]

cc_static_library = rule(
    implementation = _cc_static_library_impl,
    attrs = {
        "deps": attr.label_list(providers = [CcInfo]),
    },
    outputs = {"out": "lib%{name}.a"},
)
