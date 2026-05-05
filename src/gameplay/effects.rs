use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

pub(crate) const ENGINE_FLAME_SHADER_PATH: &str = "shaders/engine_flame.wgsl";
pub(crate) const REACTOR_GLOW_SHADER_PATH: &str = "shaders/reactor_glow.wgsl";

pub(crate) struct GameplayEffectsPlugin;

impl Plugin for GameplayEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<EngineFlameMaterial>::default(),
            Material2dPlugin::<ReactorGlowMaterial>::default(),
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
