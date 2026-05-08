use bevy::prelude::*;

use crate::state::{SplashRoot, SplashScreenState};

pub(crate) fn splash_active(splash_state: Res<SplashScreenState>) -> bool {
    splash_state.active
}

pub(crate) fn splash_inactive(splash_state: Res<SplashScreenState>) -> bool {
    !splash_state.active
}

pub(crate) fn spawn_splash_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    splash_state: Res<SplashScreenState>,
    query: Query<Entity, With<SplashRoot>>,
) {
    if !splash_state.active || !query.is_empty() {
        return;
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(-100),
            SplashRoot,
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(asset_server.load("splash.png")),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
}

pub(crate) fn cleanup_splash_ui(
    mut commands: Commands,
    query: Query<Entity, With<SplashRoot>>,
    splash_state: Res<SplashScreenState>,
) {
    if splash_state.active {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn tick_splash_screen(time: Res<Time>, mut splash_state: ResMut<SplashScreenState>) {
    if !splash_state.active {
        return;
    }

    splash_state.remaining_seconds = (splash_state.remaining_seconds - time.delta_secs()).max(0.0);
}

pub(crate) fn dismiss_splash_screen(mut splash_state: ResMut<SplashScreenState>) {
    if splash_state.active {
        splash_state.active = false;
    }
}
