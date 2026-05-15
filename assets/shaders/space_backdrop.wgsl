#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SpaceBackdropMaterial {
    base_color: vec4<f32>,
    haze_color: vec4<f32>,
    galaxy_color: vec4<f32>,
    arena_size: vec2<f32>,
    camera_offset: vec2<f32>,
    time: f32,
    seed: f32,
    star_density: f32,
    dust_density: f32,
    galaxy_strength: f32,
    parallax: f32,
    layer: f32,
    alpha: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: SpaceBackdropMaterial;

const PI: f32 = 3.14159265359;
const TAU: f32 = 6.28318530718;

fn hash11(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7)) + material.seed * 0.017) * 43758.5453123);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        hash21(p + vec2<f32>(17.0, 91.0)),
        hash21(p + vec2<f32>(43.0, 29.0))
    );
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0, 3.0) - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.52;
    var q = p;
    for (var i = 0; i < 5; i = i + 1) {
        value = value + amplitude * value_noise(q);
        q = mat2x2<f32>(1.62, 1.18, -1.18, 1.62) * q + vec2<f32>(13.7, 4.2);
        amplitude = amplitude * 0.52;
    }
    return value;
}

fn rotate2(p: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(p.x * c - p.y * s, p.x * s + p.y * c);
}

fn star_temperature(t: f32) -> vec3<f32> {
    let amber = vec3<f32>(1.0, 0.74, 0.38);
    let white = vec3<f32>(0.96, 0.98, 1.0);
    let blue = vec3<f32>(0.46, 0.60, 1.0);
    let rose = vec3<f32>(1.0, 0.34, 0.84);
    var color = mix(amber, white, smoothstep(0.18, 0.58, t));
    color = mix(color, blue, smoothstep(0.58, 0.88, t));
    color = mix(color, rose, smoothstep(0.91, 1.0, t) * 0.65);
    return color;
}

fn star_field(uv: vec2<f32>, density: f32, time: f32, layer_bias: f32) -> vec3<f32> {
    let cells = 64.0 + density * 136.0 + layer_bias * 48.0;
    let cell = floor(uv * cells);
    let local = fract(uv * cells);
    let rnd = hash21(cell + vec2<f32>(layer_bias * 19.0, layer_bias * 31.0));
    let gate = smoothstep(0.985 - density * 0.020, 0.9995, rnd);
    let star_pos = hash22(cell) * 0.76 + vec2<f32>(0.12, 0.12);
    let dist = length(local - star_pos);
    let size = mix(0.032, 0.105, pow(hash11(rnd * 79.3), 6.0));
    let core = 1.0 - smoothstep(size * 0.22, size, dist);
    let glow = (1.0 - smoothstep(size, size * 2.85, dist)) * 0.28;
    let twinkle = 0.88 + 0.12 * sin(time * (0.09 + rnd * 0.14) + rnd * TAU);
    let color = star_temperature(hash11(rnd * 113.0));
    return color * gate * (core + glow) * twinkle * (0.7 + rnd * 1.35);
}

fn clustered_spiral_galaxy(p: vec2<f32>, time: f32) -> vec3<f32> {
    let center = vec2<f32>(-0.68, 0.32 + 0.04 * sin(material.seed * 0.013));
    let q = rotate2((p - center) * vec2<f32>(1.42, 1.05), -0.34 + time * 0.0018);
    let r = length(q) + 0.0001;
    let theta = atan2(q.y, q.x);
    let winding = theta + log(r + 0.055) * 4.7;
    let arm_a = 1.0 - smoothstep(0.020, 0.22, abs(sin(winding * 2.0)));
    let arm_b = 1.0 - smoothstep(0.012, 0.17, abs(sin(winding * 3.0 + 1.35)));
    let arm_mask = max(arm_a, arm_b * 0.62);
    let disk = exp(-r * 4.6);
    let core = exp(-r * 18.0);
    let dust = smoothstep(0.42, 0.84, fbm(q * 8.0 + material.seed * 0.001));

    let cells = 96.0;
    let cell = floor((q + vec2<f32>(1.35, 1.35)) * cells);
    let local = fract((q + vec2<f32>(1.35, 1.35)) * cells);
    let rnd = hash21(cell + vec2<f32>(31.0, 7.0));
    let cluster_weight = clamp(arm_mask * disk * 3.8 + core * 1.2, 0.0, 1.0);
    let gate = smoothstep(0.995 - cluster_weight * 0.080, 0.9996, rnd);
    let star_pos = hash22(cell + vec2<f32>(5.0, 23.0)) * 0.78 + vec2<f32>(0.11, 0.11);
    let dist = length(local - star_pos);
    let size = mix(0.035, 0.13, pow(hash11(rnd * 57.0), 4.0));
    let sparkle = gate * (1.0 - smoothstep(size * 0.18, size, dist));
    let unresolved = (arm_mask * disk * (0.20 + dust * 0.22) + core * 0.85);
    let color_a = mix(material.galaxy_color.rgb, vec3<f32>(0.84, 0.38, 1.0), 0.50);
    let color_b = mix(vec3<f32>(1.0, 0.42, 0.12), material.galaxy_color.rgb, 0.20);
    let population = mix(color_b, color_a, smoothstep(0.12, 0.74, r));
    let hot_stars = star_temperature(hash11(rnd * 181.0));
    return (population * unresolved + hot_stars * sparkle * 2.8) * material.galaxy_strength;
}

fn elliptical_galaxy(p: vec2<f32>, center: vec2<f32>, angle: f32, tint: vec3<f32>, scale: vec2<f32>) -> vec3<f32> {
    let q = rotate2(p - center, angle) * scale;
    let r = length(q);
    let halo = exp(-r * 2.6);
    let core = exp(-r * 12.0);
    let shell = (1.0 - smoothstep(0.07, 0.42, abs(r - 0.42))) * 0.13;
    let grain = 0.78 + fbm(q * 18.0 + material.seed * 0.002) * 0.38;
    let warm_core = vec3<f32>(1.0, 0.72, 0.34);
    return (tint * halo * 0.72 + warm_core * core * 1.2 + tint * shell) * grain * material.galaxy_strength;
}

fn barred_spiral_galaxy(p: vec2<f32>, time: f32) -> vec3<f32> {
    let q = rotate2((p - vec2<f32>(0.72, -0.36)) * vec2<f32>(1.18, 1.18), 0.62 - time * 0.0012);
    let r = length(q) + 0.0001;
    let theta = atan2(q.y, q.x);
    let bar = exp(-abs(q.y) * 16.0) * (1.0 - smoothstep(0.02, 0.56, abs(q.x))) * 0.95;
    let winding = theta - log(r + 0.08) * 3.6;
    let arms = (1.0 - smoothstep(0.04, 0.32, abs(sin(winding * 2.0 + 0.7)))) * exp(-r * 3.0);
    let knots = smoothstep(0.64, 0.92, fbm(q * 14.0 + vec2<f32>(2.0, 9.0)));
    let blue_arms = vec3<f32>(0.42, 0.50, 1.0);
    let gold_bar = vec3<f32>(1.0, 0.46, 0.12);
    let violet = vec3<f32>(0.92, 0.28, 1.0);
    return (gold_bar * bar + mix(blue_arms, violet, knots) * arms * (0.45 + knots)) * material.galaxy_strength;
}

fn irregular_galaxy(p: vec2<f32>, center: vec2<f32>, tint_a: vec3<f32>, tint_b: vec3<f32>) -> vec3<f32> {
    let q = p - center;
    let r = length(q * vec2<f32>(1.1, 1.45));
    let cloud_a = smoothstep(0.40, 0.88, fbm(q * 6.5 + material.seed * 0.004));
    let cloud_b = smoothstep(0.50, 0.94, fbm(q * 11.0 + vec2<f32>(6.0, 1.0)));
    let body = exp(-r * 3.2) * (cloud_a + cloud_b * 0.64);
    let hot_knots = smoothstep(0.76, 0.98, fbm(q * 28.0 + vec2<f32>(13.0, 7.0))) * exp(-r * 4.2);
    return (mix(tint_a, tint_b, cloud_b) * body + vec3<f32>(1.0, 0.30, 0.52) * hot_knots * 0.9) * material.galaxy_strength;
}

fn globular_cluster(p: vec2<f32>, center: vec2<f32>, radius: f32, tint: vec3<f32>) -> vec3<f32> {
    let q = p - center;
    let r = length(q) / radius;
    let haze = exp(-r * 4.5) * 0.45;
    let cells = 70.0 / radius;
    let cell = floor((q + vec2<f32>(radius, radius)) * cells);
    let local = fract((q + vec2<f32>(radius, radius)) * cells);
    let rnd = hash21(cell + center * 113.0);
    let density = clamp(exp(-r * 3.8), 0.0, 1.0);
    let gate = smoothstep(0.990 - density * 0.055, 0.9994, rnd);
    let pos = hash22(cell + vec2<f32>(8.0, 71.0)) * 0.80 + vec2<f32>(0.10, 0.10);
    let spark = gate * (1.0 - smoothstep(0.018, 0.075, length(local - pos)));
    return tint * haze + star_temperature(hash11(rnd * 44.0)) * spark * 1.9;
}

fn nebula(p: vec2<f32>, time: f32) -> f32 {
    let slow = vec2<f32>(time * 0.003, -time * 0.0022);
    let gas = fbm(p * 2.3 + slow + material.seed * 0.0007);
    let filament = fbm(p * 5.8 - slow * 1.6 + vec2<f32>(7.1, 2.6));
    let large_cloud = smoothstep(0.34, 0.88, gas);
    let wisps = smoothstep(0.44, 0.78, filament) * 0.42;
    return (large_cloud + wisps) * material.dust_density;
}

fn dust_lane(p: vec2<f32>, time: f32) -> f32 {
    let lane_p = p * vec2<f32>(3.8, 1.4) + vec2<f32>(time * 0.002, material.seed * 0.0003);
    let lane = fbm(lane_p);
    let band = 1.0 - smoothstep(0.05, 0.34, abs(p.y + sin(p.x * 2.1) * 0.12));
    return smoothstep(0.48, 0.78, lane) * band * 0.55;
}

fn colored_nebula(p: vec2<f32>, time: f32) -> vec3<f32> {
    let gas = nebula(p, time);
    let veil = smoothstep(0.44, 0.90, fbm(p * 3.8 + vec2<f32>(-time * 0.0015, time * 0.002)));
    let violet = mix(material.haze_color.rgb, vec3<f32>(0.64, 0.18, 1.0), 0.62);
    let magenta = mix(material.galaxy_color.rgb, vec3<f32>(1.0, 0.14, 0.74), 0.58);
    let amber = vec3<f32>(1.0, 0.38, 0.08);
    let color = mix(mix(violet, magenta, veil), amber, smoothstep(0.70, 1.0, fbm(p * 7.0 + 3.0)) * 0.46);
    return color * gas * (0.32 + veil * 0.38);
}

fn pixelated_uv(uv: vec2<f32>) -> vec2<f32> {
    let virtual_resolution = max(material.arena_size / 4, vec2<f32>(320.0, 320.0));
    return (floor(uv * virtual_resolution) + vec2<f32>(0.5, 0.5)) / virtual_resolution;
}

fn posterize_color(color: vec3<f32>) -> vec3<f32> {
    let levels = vec3<f32>(28.0, 24.0, 28.0);
    let compressed = color / (color + vec3<f32>(0.55, 0.55, 0.55));
    return floor(compressed * levels + vec3<f32>(0.5, 0.5, 0.5)) / levels;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = max(0.25, material.arena_size.x / max(1.0, material.arena_size.y));
    let camera_uv = material.camera_offset / max(material.arena_size, vec2<f32>(1.0, 1.0));
    let uv = pixelated_uv(mesh.uv + camera_uv * (0.18 + material.parallax * 0.16));
    let p = (uv - vec2<f32>(0.5, 0.5)) * vec2<f32>(aspect, 1.0) * 2.35;
    let vignette = 1.0 - smoothstep(0.78, 1.78, length(p));

    let haze = mix(material.haze_color.rgb, vec3<f32>(0.42, 0.12, 0.58), 0.34);
    let galaxy = mix(material.galaxy_color.rgb, vec3<f32>(1.0, 0.42, 0.16), 0.22);

    let stars = star_field(uv + vec2<f32>(material.seed * 0.00013, 0.0), material.star_density, material.time, material.layer);
    let spiral_cluster = clustered_spiral_galaxy(p, material.time);
    let barred = barred_spiral_galaxy(p, material.time);
    let elliptical_a = elliptical_galaxy(
        p,
        vec2<f32>(-0.12, -0.70),
        -0.36,
        mix(material.galaxy_color.rgb, vec3<f32>(1.0, 0.70, 0.36), 0.34),
        vec2<f32>(1.05, 2.20)
    );
    let elliptical_b = elliptical_galaxy(
        p,
        vec2<f32>(0.98, 0.54),
        0.82,
        vec3<f32>(0.72, 0.56, 1.0),
        vec2<f32>(1.65, 3.40)
    ) * 0.58;
    let irregular_a = irregular_galaxy(
        p,
        vec2<f32>(-1.04, -0.50),
        vec3<f32>(0.20, 0.95, 0.82),
        vec3<f32>(0.94, 0.34, 1.0)
    );
    let irregular_b = irregular_galaxy(
        p,
        vec2<f32>(0.10, 0.74),
        vec3<f32>(0.22, 0.48, 1.0),
        vec3<f32>(1.0, 0.45, 0.18)
    ) * 0.42;
    let clusters = globular_cluster(p, vec2<f32>(-0.34, 0.58), 0.18, vec3<f32>(0.96, 0.78, 0.48)) +
        globular_cluster(p, vec2<f32>(0.48, 0.16), 0.13, vec3<f32>(0.58, 0.82, 1.0)) +
        globular_cluster(p, vec2<f32>(1.15, -0.03), 0.10, vec3<f32>(1.0, 0.58, 0.74));
    let galaxy_scene = spiral_cluster + barred + elliptical_a + elliptical_b + irregular_a + irregular_b + clusters;
    let cloud = nebula(p + vec2<f32>(0.18, -0.06), material.time);
    let cloud_mask = smoothstep(0.72, 1.12, cloud);
    let color_cloud = colored_nebula(p + vec2<f32>(0.18, -0.06), material.time);
    let lane = dust_lane(p, material.time);

    var color = vec3<f32>(0.0, 0.0, 0.0);
    var alpha = 0.0;
    if material.layer < 0.5 {
        let remote_galaxies = elliptical_galaxy(p, vec2<f32>(-1.18, 0.84), 0.24, vec3<f32>(0.90, 0.42, 1.0), vec2<f32>(2.4, 5.2)) * 0.32 +
            elliptical_galaxy(p, vec2<f32>(1.26, -0.76), -0.66, vec3<f32>(1.0, 0.44, 0.18), vec2<f32>(2.8, 7.0)) * 0.25;
        color = stars * 0.70 + remote_galaxies;
        alpha = material.alpha * clamp(length(color) * 0.42, 0.0, 1.0);
    } else if material.layer < 1.5 {
        color = color_cloud * cloud_mask * 0.34 + galaxy_scene * 1.26;
        color = color * (1.0 - lane * 0.62) + stars * 0.12;
        alpha = material.alpha * clamp(length(color) * 0.32 * (0.58 + vignette * 0.32), 0.0, 0.72);
    } else {
        let sparkle = star_field(uv * 1.41 + vec2<f32>(0.37, 0.19), material.star_density * 0.78, material.time + 37.0, 2.0);
        let fine_gas = smoothstep(0.68, 0.96, fbm(p * 9.0 + material.time * 0.004));
        let foreground_clusters = globular_cluster(p, vec2<f32>(-0.86, 0.08), 0.11, vec3<f32>(0.68, 1.0, 0.82)) * 0.34 +
            globular_cluster(p, vec2<f32>(0.88, 0.78), 0.08, vec3<f32>(1.0, 0.84, 0.46)) * 0.30;
        color = sparkle * 0.82 + mix(haze, galaxy, 0.64) * fine_gas * 0.055 + foreground_clusters;
        alpha = material.alpha * clamp(length(color) * 0.28, 0.0, 0.24);
    }

    color = color * (0.52 + vignette * 0.42);
    color = posterize_color(color);
    if alpha <= 0.01 {
        discard;
    }
    return vec4<f32>(color, alpha);
}
