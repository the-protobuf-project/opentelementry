"""
"""

def _gen_env_file(ctx):
    out = ctx.outputs.out
    ctx.actions.run(
        outputs = [
            out,
        ],
        arguments = [
            out.path,
        ],
        executable = ctx.executable._generator,
        use_default_shell_env = True,
    )

gen_env_file = rule(
    implementation = _gen_env_file,
    attrs = {
        "out": attr.output(mandatory = True),
        "_generator": attr.label(default = "//bazel/lib/private:gen_env_file", executable = True, cfg = "exec"),
    },
)
