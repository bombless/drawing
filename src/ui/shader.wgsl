// 顶点着色器

struct ColorUniform {
    color: vec3f,
};
@group(1) @binding(0) // 1.
var<uniform> color: ColorUniform;

struct TransformUniform {
    transform: mat4x4f,
};
@group(0) @binding(0) // 1.
var<uniform> transform: TransformUniform;

struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = transform.transform * vec4f(model.position, 0.0, 1.0);
    return out;
}

// 片元着色器

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(color.color, 1.0);
}
