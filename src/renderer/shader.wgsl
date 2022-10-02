struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Drawable {
    transform: mat2x2<f32>,
    translation: vec2<f32>,
};
@group(1)
@binding(0)
var<uniform> r_drawable: Drawable;

@group(1)
@binding(1)
var r_color: texture_2d<f32>;

@group(1)
@binding(2)
var r_sampler: sampler;

struct Stage {
    size: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> r_stage: Stage;

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    var K: vec4<f32> = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    var p: vec4<f32> = mix(vec4<f32>(c.bg, K.wz), vec4<f32>(c.gb, K.xy), vec4<f32>(step(c.b, c.g)));
    var q: vec4<f32> = mix(vec4<f32>(p.xyw, c.r), vec4<f32>(c.r, p.yzx), vec4<f32>(step(p.x, c.r)));

    var d: f32 = q.x - min(q.w, q.y);
    var e: f32 = 1.0e-10;
    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
    //return vec3<f32>(0.0, 0.0, 0.0);
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    out.position = vec4<f32>(((r_drawable.transform * position) + r_drawable.translation) / (r_stage.size * 0.5), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(rgb_to_hsv(vec3<f32>(in.tex_coord.x, 1.0, 1.0)), 1.0);
    //return vec4<f32>(in.tex_coord.x, in.tex_coord.y, 0.0, 1.0);
    return textureSample(r_color, r_sampler, in.tex_coord);
}
