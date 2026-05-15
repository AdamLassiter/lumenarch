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
    let dir = normalize(material.direction + vec2<f32>(0.001, 0.001));
    let side = vec2<f32>(-dir.y, dir.x);
    let p = mesh.uv - vec2<f32>(0.5, 0.5);
    let along = dot(p, dir) + material.time * (0.35 + material.intensity * 0.45);
    let across = dot(p, side);
    let stripe = smoothstep(0.035, 0.0, abs(fract(along * 7.0) - 0.5)) * smoothstep(0.42, 0.02, abs(across));
    if stripe <= 0.01 {
        discard;
    }
    let color = mix(material.secondary_color.rgb, material.primary_color.rgb, stripe);
    return vec4<f32>(color, stripe * material.alpha);
}
