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
    let along = dot(p, dir) + material.time * (0.55 + material.intensity * 0.80);
    let across = dot(p, side);
    let stripe = smoothstep(0.020, 0.0, abs(fract(along * 9.0) - 0.5));
    let lane = smoothstep(0.045, 0.0, abs(fract(across * 5.0) - 0.5));
    let fade = 1.0 - smoothstep(0.18, 0.58, length(p));
    let alpha = stripe * lane * fade * material.alpha;
    if alpha <= 0.01 {
        discard;
    }
    return vec4<f32>(material.primary_color.rgb, alpha);
}
