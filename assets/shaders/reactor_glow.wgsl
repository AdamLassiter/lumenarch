#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ReactorGlowMaterial {
    core_color: vec4<f32>,
    edge_color: vec4<f32>,
    spark_color: vec4<f32>,
    haze_color: vec4<f32>,
    time: f32,
    intensity: f32,
    alpha: f32,
    pixel_scale: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ReactorGlowMaterial;

fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(41.0, 289.0));
    return fract(sin(h) * 43758.5453123);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let grid = vec2<f32>(max(10.0, material.pixel_scale), max(10.0, material.pixel_scale));
    let cell_id = floor(mesh.uv * grid);
    let uv = (cell_id + vec2<f32>(0.5, 0.5)) / grid;
    let p = uv * 2.0 - vec2<f32>(1.0, 1.0);
    let r = length(p);
    if r > 0.98 {
        discard;
    }

    let haze = clamp(1.0 - r, 0.0, 1.0);
    let pulse_wave = 0.5 + 0.5 * sin(material.time * 5.3 + r * 7.5);
    let pulse = 0.72 + pulse_wave * 0.46;
    let noise = hash21(cell_id + vec2<f32>(floor(material.time * 6.0), floor(material.time * 11.0)));
    let ring = (1.0 - smoothstep(0.40, 0.72, r)) * smoothstep(0.16, 0.28, r);
    let hot_spark = mix(material.spark_color.rgb, vec3<f32>(0.94, 1.0, 0.96), 0.60);

    var color = mix(material.edge_color.rgb, material.core_color.rgb, clamp(haze * 1.35, 0.0, 1.0));
    color = mix(material.haze_color.rgb, color, clamp(haze * 1.1, 0.0, 1.0));
    color = mix(color, hot_spark, ring * (0.18 + pulse_wave * 0.16) * material.intensity);

    if noise > 0.84 && r < 0.48 {
        color = mix(color, hot_spark, 0.36);
    } else if noise > 0.67 && r < 0.68 + material.intensity * 0.12 {
        color = hot_spark;
    }

    let alpha = material.alpha
        * clamp((haze * pulse + ring * (0.20 + pulse_wave * 0.12)) * (0.86 + material.intensity * 0.58), 0.0, 0.92)
        * (1.0 - smoothstep(0.18, 0.98, r));
    return vec4<f32>(color.rgb, alpha);
}
