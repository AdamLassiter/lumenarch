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

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let r = length(p);
    if r > 0.92 {
        discard;
    }
    let band = abs(fract((mesh.uv.y + material.time * 0.45) * 4.0) - 0.5);
    let pulse = smoothstep(0.35, 0.02, band) * (1.0 - smoothstep(0.72, 0.94, r));
    let core = (1.0 - smoothstep(0.0, 0.80, r)) * (0.35 + 0.65 * material.intensity);
    let alpha = max(core * 0.45, pulse) * material.alpha;
    if alpha <= 0.01 {
        discard;
    }
    let color = mix(material.secondary_color.rgb, material.primary_color.rgb, pulse + core);
    return vec4<f32>(color, alpha);
}
