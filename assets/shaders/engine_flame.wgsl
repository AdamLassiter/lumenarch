#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct EngineFlameMaterial {
    cool_color: vec4<f32>,
    hot_color: vec4<f32>,
    warm_color: vec4<f32>,
    ash_color: vec4<f32>,
    time: f32,
    growth: f32,
    intensity: f32,
    alpha: f32,
    pixel_scale: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: EngineFlameMaterial;

fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn flame_gradient(t: f32) -> vec3<f32> {
    let cool_to_hot = smoothstep(0.04, 0.24, t);
    let hot_to_warm = smoothstep(0.22, 0.58, t);
    let warm_to_ash = smoothstep(0.60, 0.92, t);
    let core = mix(material.cool_color.rgb, material.hot_color.rgb, cool_to_hot);
    let warm = mix(core, material.warm_color.rgb, hot_to_warm);
    return mix(warm, material.ash_color.rgb, warm_to_ash);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let grid = vec2<f32>(max(6.0, material.pixel_scale * 0.6), max(10.0, material.pixel_scale * 1.1));
    let cell_id = floor(mesh.uv * grid);
    let uv = (cell_id + vec2<f32>(0.5, 0.5)) / grid;

    let along = 1.0 - uv.y;
    let x = uv.x * 2.0 - 1.0;
    let along_step = floor(along * 8.0) / 8.0;
    let cone_width = mix(0.46, 1.34, pow(along_step, 0.84));
    let center_falloff = clamp(1.0 - abs(x) / cone_width, 0.0, 1.0);
    let center_step = floor(center_falloff * 4.0) / 4.0;
    if center_falloff <= 0.01 {
        discard;
    }

    let flicker = 0.88
        + 0.12 * sin(material.time * 20.0 + along * 18.0)
        + 0.08 * sin(material.time * 11.0 + x * 7.0);
    let smoke_noise = hash21(cell_id + vec2<f32>(floor(material.time * 3.0), 17.0));
    let base_core = step(0.22, center_step) * (0.42 + 0.58 * (1.0 - along_step)) * flicker;
    let base_haze = step(0.50, center_step) * step(0.38, along_step) * step(0.54, smoke_noise) * (0.24 + 0.46 * along_step);

    var particle_energy = 0.0;
    var particle_heat = 0.0;
    for (var i: i32 = 0; i < 1; i = i + 1) {
        let fi = f32(i);
        let seed = hash21(vec2<f32>(fi * 13.7, fi * 5.9));
        let seed_b = hash21(vec2<f32>(fi * 3.1 + 9.0, fi * 19.7 + 2.0));
        let speed = mix(0.32, 1.18, seed);
        let travel = fract(seed_b + material.time * speed * (0.38 + material.intensity * 0.52));
        let particle_y = travel;
        let cone_spread = mix(0.10, 0.98, pow(particle_y, 0.80));
        let drift = (seed - 0.5) * cone_spread + sin(material.time * (4.0 + seed * 9.0) + fi) * 0.10 * particle_y;
        let particle_x = drift;
        let size_x = mix(0.32, 0.12, particle_y);
        let size_y = mix(0.24, 0.10, particle_y);
        let dx = abs(x - particle_x) / max(size_x, 0.001);
        let dy = abs(along - particle_y) / max(size_y, 0.001);
        let shape = step(max(dx, dy), 1.0);
        particle_energy += shape * mix(0.35, 1.0, seed);
        particle_heat += shape * clamp(particle_y + seed * 0.22, 0.0, 1.0);
    }

    particle_energy = clamp(particle_energy / 1.5, 0.0, 1.0);
    particle_heat = clamp(particle_heat / 1.35, 0.0, 1.0);

    let plume_mask = max(base_core, particle_energy * 1.25);
    let smoke_mask = max(base_haze, particle_energy * 0.35 * along_step);
    if plume_mask <= 0.03 && smoke_mask <= 0.02 {
        discard;
    }

    let tail_fade = 1.0 - smoothstep(0.68, 0.96, along);
    let edge_fade = 1.0 - smoothstep(0.90, 1.0, abs(x) / max(cone_width, 0.001));
    let growth_head = material.growth;
    let growth_fade = 1.0 - smoothstep(growth_head, growth_head + 0.16, along);

    let gradient_t = clamp(mix(along, particle_heat, particle_energy * 0.7), 0.0, 1.0);
    var flame_rgb = flame_gradient(gradient_t);
    flame_rgb = mix(flame_rgb, vec3<f32>(0.86, 0.94, 1.0), smoothstep(0.0, 0.12, along) * smoothstep(0.72, 1.0, particle_energy));
    flame_rgb = mix(flame_rgb, material.ash_color.rgb, smoke_mask * smoothstep(0.48, 1.0, along) * 0.55);

    let alpha_steps = floor(clamp((plume_mask * 0.82 + smoke_mask * 0.30) * 4.0, 0.0, 4.0)) / 4.0;
    let alpha = material.alpha
        * clamp(
            alpha_steps
                * (0.55 + material.intensity * 0.85)
                * tail_fade
                * edge_fade,
            0.0,
            1.0,
        );
    let final_alpha = alpha * growth_fade;
    if final_alpha <= 0.01 {
        discard;
    }
    return vec4<f32>(flame_rgb, final_alpha);
}
