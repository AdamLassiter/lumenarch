use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

pub(crate) const ENGINE_FLAME_SHADER_PATH: &str = "shaders/engine_flame.wgsl";
pub(crate) const REACTOR_GLOW_SHADER_PATH: &str = "shaders/reactor_glow.wgsl";
pub(crate) const TURRET_FLASH_SHADER_PATH: &str = "shaders/turret_flash.wgsl";
pub(crate) const BATTERY_PULSE_SHADER_PATH: &str = "shaders/battery_pulse.wgsl";
pub(crate) const FABRICATOR_DUST_SHADER_PATH: &str = "shaders/fabricator_dust.wgsl";
pub(crate) const AIR_LINES_SHADER_PATH: &str = "shaders/air_lines.wgsl";
pub(crate) const SPEED_LINES_SHADER_PATH: &str = "shaders/speed_lines.wgsl";
pub(crate) const ELECTRIC_ARCS_SHADER_PATH: &str = "shaders/electric_arcs.wgsl";
pub(crate) const SMALL_FLAMES_SHADER_PATH: &str = "shaders/small_flames.wgsl";
pub(crate) const SPACE_BACKDROP_SHADER_PATH: &str = "shaders/space_backdrop.wgsl";
pub(crate) const SPACE_BACKDROP_FALLBACK_SEED: f32 = 12_648_430.0;

pub(crate) struct GameplayEffectsPlugin;

impl Plugin for GameplayEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<EngineFlameMaterial>::default(),
            Material2dPlugin::<ReactorGlowMaterial>::default(),
            Material2dPlugin::<TurretFlashMaterial>::default(),
            Material2dPlugin::<BatteryPulseMaterial>::default(),
            Material2dPlugin::<FabricatorDustMaterial>::default(),
            Material2dPlugin::<AirLinesMaterial>::default(),
            Material2dPlugin::<SpeedLinesMaterial>::default(),
            Material2dPlugin::<ElectricArcsMaterial>::default(),
            Material2dPlugin::<SmallFlamesMaterial>::default(),
            Material2dPlugin::<SpaceBackdropMaterial>::default(),
        ));
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub(crate) struct EngineFlameUniform {
    pub(crate) cool_color: Vec4,
    pub(crate) hot_color: Vec4,
    pub(crate) warm_color: Vec4,
    pub(crate) ash_color: Vec4,
    pub(crate) time: f32,
    pub(crate) growth: f32,
    pub(crate) intensity: f32,
    pub(crate) alpha: f32,
    pub(crate) pixel_scale: f32,
}

impl Default for EngineFlameUniform {
    fn default() -> Self {
        Self {
            cool_color: Vec4::new(0.24, 0.58, 1.00, 1.0),
            hot_color: Vec4::new(0.92, 0.18, 0.12, 1.0),
            warm_color: Vec4::new(1.00, 0.58, 0.16, 1.0),
            ash_color: Vec4::new(0.44, 0.46, 0.50, 1.0),
            time: 0.0,
            growth: 0.0,
            intensity: 0.0,
            alpha: 0.0,
            pixel_scale: 18.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub(crate) struct EngineFlameMaterial {
    #[uniform(0)]
    pub(crate) params: EngineFlameUniform,
}

impl Material2d for EngineFlameMaterial {
    fn fragment_shader() -> ShaderRef {
        ENGINE_FLAME_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub(crate) struct ReactorGlowUniform {
    pub(crate) core_color: Vec4,
    pub(crate) edge_color: Vec4,
    pub(crate) spark_color: Vec4,
    pub(crate) haze_color: Vec4,
    pub(crate) time: f32,
    pub(crate) intensity: f32,
    pub(crate) alpha: f32,
    pub(crate) pixel_scale: f32,
}

impl Default for ReactorGlowUniform {
    fn default() -> Self {
        Self {
            core_color: Vec4::new(0.24, 0.90, 0.46, 1.0),
            edge_color: Vec4::new(0.08, 0.30, 0.14, 1.0),
            spark_color: Vec4::new(0.80, 0.98, 0.84, 1.0),
            haze_color: Vec4::new(0.12, 0.58, 0.28, 1.0),
            time: 0.0,
            intensity: 0.0,
            alpha: 0.0,
            pixel_scale: 16.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub(crate) struct ReactorGlowMaterial {
    #[uniform(0)]
    pub(crate) params: ReactorGlowUniform,
}

impl Material2d for ReactorGlowMaterial {
    fn fragment_shader() -> ShaderRef {
        REACTOR_GLOW_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub(crate) struct EffectUniform {
    pub(crate) primary_color: Vec4,
    pub(crate) secondary_color: Vec4,
    pub(crate) time: f32,
    pub(crate) intensity: f32,
    pub(crate) alpha: f32,
    pub(crate) direction: Vec2,
    pub(crate) pixel_scale: f32,
}

impl Default for EffectUniform {
    fn default() -> Self {
        Self {
            primary_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            secondary_color: Vec4::new(0.35, 0.70, 1.0, 1.0),
            time: 0.0,
            intensity: 0.0,
            alpha: 0.0,
            direction: Vec2::Y,
            pixel_scale: 16.0,
        }
    }
}

macro_rules! effect_material {
    ($material:ident, $shader_path:ident) => {
        #[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
        pub(crate) struct $material {
            #[uniform(0)]
            pub(crate) params: EffectUniform,
        }

        impl Material2d for $material {
            fn fragment_shader() -> ShaderRef {
                $shader_path.into()
            }

            fn alpha_mode(&self) -> AlphaMode2d {
                AlphaMode2d::Blend
            }
        }
    };
}

effect_material!(TurretFlashMaterial, TURRET_FLASH_SHADER_PATH);
effect_material!(BatteryPulseMaterial, BATTERY_PULSE_SHADER_PATH);
effect_material!(FabricatorDustMaterial, FABRICATOR_DUST_SHADER_PATH);
effect_material!(AirLinesMaterial, AIR_LINES_SHADER_PATH);
effect_material!(SpeedLinesMaterial, SPEED_LINES_SHADER_PATH);
effect_material!(ElectricArcsMaterial, ELECTRIC_ARCS_SHADER_PATH);
effect_material!(SmallFlamesMaterial, SMALL_FLAMES_SHADER_PATH);

#[derive(Clone, Copy, Debug, ShaderType)]
pub(crate) struct SpaceBackdropUniform {
    pub(crate) base_color: Vec4,
    pub(crate) haze_color: Vec4,
    pub(crate) galaxy_color: Vec4,
    pub(crate) arena_size: Vec2,
    pub(crate) camera_offset: Vec2,
    pub(crate) time: f32,
    pub(crate) seed: f32,
    pub(crate) star_density: f32,
    pub(crate) dust_density: f32,
    pub(crate) galaxy_strength: f32,
    pub(crate) parallax: f32,
    pub(crate) layer: f32,
    pub(crate) alpha: f32,
}

impl Default for SpaceBackdropUniform {
    fn default() -> Self {
        Self {
            base_color: Vec4::new(0.035, 0.045, 0.075, 1.0),
            haze_color: Vec4::new(0.12, 0.17, 0.24, 1.0),
            galaxy_color: Vec4::new(0.58, 0.72, 0.94, 1.0),
            arena_size: Vec2::new(1024.0, 768.0),
            camera_offset: Vec2::ZERO,
            time: 0.0,
            seed: SPACE_BACKDROP_FALLBACK_SEED,
            star_density: 0.5,
            dust_density: 0.5,
            galaxy_strength: 0.3,
            parallax: 0.2,
            layer: 0.0,
            alpha: 1.0,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub(crate) struct SpaceBackdropMaterial {
    #[uniform(0)]
    pub(crate) params: SpaceBackdropUniform,
}

impl Material2d for SpaceBackdropMaterial {
    fn fragment_shader() -> ShaderRef {
        SPACE_BACKDROP_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
