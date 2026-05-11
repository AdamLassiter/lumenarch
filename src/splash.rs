use bevy::prelude::*;

use crate::state::{SplashRoot, SplashScreenState};

/// Returns whether the splash screen should currently exist in the UI world.
pub(crate) fn splash_active(splash_state: Res<SplashScreenState>) -> bool {
    splash_state.active
}

/// Returns whether splash UI entities should be cleaned up because the splash is done.
pub(crate) fn splash_inactive(splash_state: Res<SplashScreenState>) -> bool {
    !splash_state.active
}

/// Spawns the splash image layer so the game has a branded backdrop during early frontend flow.
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

/// Removes splash entities once the app has moved on to normal frontend presentation.
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

/// Advances the splash timer so time-based transitions can react to it elsewhere.
pub(crate) fn tick_splash_screen(time: Res<Time>, mut splash_state: ResMut<SplashScreenState>) {
    if !splash_state.active {
        return;
    }

    splash_state.remaining_seconds = (splash_state.remaining_seconds - time.delta_secs()).max(0.0);
}

/// Dismisses the splash when the session reaches the docked game flow.
pub(crate) fn dismiss_splash_screen(mut splash_state: ResMut<SplashScreenState>) {
    if splash_state.active {
        splash_state.active = false;
    }
}
