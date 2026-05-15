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
    return fract(sin(dot(p, vec2<f32>(113.5, 271.9))) * 43758.5453123);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let base_y = p.y + 0.52;
    let columns = floor(mesh.uv.x * 7.0);
    let flicker = hash21(vec2<f32>(columns, floor(material.time * 10.0)));
    let width = 0.18 + flicker * 0.16 + material.intensity * 0.14;
    let flame = smoothstep(width, 0.0, abs(fract(mesh.uv.x * 3.5) - 0.5))
        * smoothstep(-0.42, 0.08 + flicker * 0.32, base_y)
        * (1.0 - smoothstep(0.22 + flicker * 0.28, 0.82, base_y));
    if flame <= 0.02 {
        discard;
    }
    let color = mix(material.secondary_color.rgb, material.primary_color.rgb, 1.0 - mesh.uv.y);
    return vec4<f32>(color, flame * material.alpha);
}
