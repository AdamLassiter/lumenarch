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
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let grid = max(8.0, material.pixel_scale);
    let cell = floor(mesh.uv * vec2<f32>(grid, grid));
    let noise = hash21(cell + vec2<f32>(floor(material.time * 8.0), 13.0));
    let lift = fract(mesh.uv.y + material.time * (0.18 + material.intensity * 0.22));
    let mask = step(0.68, noise) * smoothstep(0.04, 0.30, lift) * (1.0 - smoothstep(0.74, 1.0, lift));
    if mask <= 0.0 {
        discard;
    }
    let color = mix(material.secondary_color.rgb, material.primary_color.rgb, noise);
    return vec4<f32>(color, mask * material.alpha);
}
