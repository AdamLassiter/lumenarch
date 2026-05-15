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
    let ray = max(abs(p.x) * 0.55 + abs(p.y), abs(p.y) * 0.55 + abs(p.x));
    let star = 1.0 - smoothstep(0.10, 0.86, min(r, ray));
    let core = 1.0 - smoothstep(0.0, 0.28, r);
    let spark = max(star, core) * material.intensity;
    if spark <= 0.02 {
        discard;
    }
    let color = mix(material.secondary_color.rgb, material.primary_color.rgb, core);
    return vec4<f32>(color, spark * material.alpha);
}
