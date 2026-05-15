#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct EffectMaterial {
    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,
    time: f32,
    intensity: f32,
    alpha: f32,
    direction: vec2<f32>,
    pixel_scale: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: EffectMaterial;

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(41.0, 289.0))) * 43758.5453123);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let grid = floor(mesh.uv * vec2<f32>(10.0, 10.0));
    let uv = (grid + vec2<f32>(0.5, 0.5)) / vec2<f32>(10.0, 10.0);
    let jitter = hash21(grid + vec2<f32>(floor(material.time * 14.0), 3.0));
    let bolt = smoothstep(0.13, 0.0, abs(uv.y - (0.5 + sin(uv.x * 19.0 + material.time * 18.0) * 0.16)));
    let spark = step(0.84 - material.intensity * 0.18, jitter);
    let mask = max(bolt * spark, step(0.96, jitter) * material.intensity);
    if mask <= 0.01 {
        discard;
    }
    let color = mix(material.secondary_color.rgb, material.primary_color.rgb, mask);
    return vec4<f32>(color, mask * material.alpha);
}
