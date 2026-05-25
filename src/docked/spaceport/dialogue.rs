use super::*;

pub(crate) fn docked_keyboard_interaction_system(
    keys: Res<ButtonInput<KeyCode>>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    scene_state: Option<Res<DockedSpaceportState>>,
    mut dialogue_state: ResMut<DockedDialogueState>,
    mut pending_meta: ResMut<PendingLocalMetaCommand>,
) {
    if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Escape) {
        if dialogue_state.active_npc_id.is_some() {
            clear_docked_dialogue_state(&mut dialogue_state);
        }
        return;
    }
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Some(scene_state) = scene_state else {
        return;
    };
    let focus = focused_tile(scene_state.avatar);
    if scene_state.cockpit_tile == Some(focus) {
        pending_meta.0 = Some(PendingMetaCommand {
            op: RollbackMetaOp::OpenSectorMap,
            ..Default::default()
        });
        return;
    }
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    if let Some(npc) = station_npcs(&station.ship)
        .into_iter()
        .find(|npc| (npc.grid_x, npc.grid_y) == focus)
    {
        dialogue_state.active_npc_id = Some(npc.id);
        dialogue_state.selected_surface_hint = Some(npc.service_action);
    }
}

/// Keeps the active NPC backed by an actual Yarn Spinner dialogue runner when the project is loaded.
pub(crate) fn sync_docked_yarn_runner(
    mut commands: Commands,
    yarn_project: Option<Res<YarnProject>>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    mut dialogue_state: ResMut<DockedDialogueState>,
) {
    let Some(npc_id) = dialogue_state.active_npc_id.clone() else {
        if let Some(entity) = dialogue_state.runner_entity.take() {
            commands.entity(entity).despawn();
        }
        dialogue_state.active_yarn_node = None;
        dialogue_state.yarn_speaker = None;
        dialogue_state.yarn_line = None;
        dialogue_state.yarn_options.clear();
        return;
    };
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    let Some(npc) = station_npcs(&station.ship)
        .into_iter()
        .find(|npc| npc.id == npc_id)
    else {
        return;
    };
    if dialogue_state.active_yarn_node.as_deref() == Some(npc.dialogue_start_node.as_str())
        && dialogue_state.runner_entity.is_some()
    {
        return;
    }
    if dialogue_state.active_yarn_node.as_deref() == Some(npc.dialogue_start_node.as_str())
        && yarn_project.is_none()
    {
        return;
    }

    if let Some(entity) = dialogue_state.runner_entity.take() {
        commands.entity(entity).despawn();
    }
    dialogue_state.active_yarn_node = Some(npc.dialogue_start_node.clone());
    dialogue_state.yarn_speaker = None;
    dialogue_state.yarn_line = Some(format!(
        "Loading Yarn node `{}`...",
        npc.dialogue_start_node
    ));
    dialogue_state.yarn_options.clear();

    let Some(yarn_project) = yarn_project else {
        return;
    };
    let mut runner = yarn_project.create_dialogue_runner(&mut commands);
    if runner.try_start_node(&npc.dialogue_start_node).is_err() {
        dialogue_state.yarn_line = Some(format!(
            "Yarn node `{}` could not be started.",
            npc.dialogue_start_node
        ));
        dialogue_state.active_yarn_node = None;
        return;
    }
    let entity = commands.spawn((runner, DockedYarnRunner)).id();
    dialogue_state.runner_entity = Some(entity);
}

/// Spawns or removes the local NPC dialogue panel so docked conversations stay presentation-only.
pub(crate) fn sync_docked_dialogue_overlay(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dialogue_state: Res<DockedDialogueState>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    existing_query: Query<Entity, With<DockedDialogueRoot>>,
) {
    if !(dialogue_state.is_changed()
        || dialogue_state.active_npc_id.is_some() && existing_query.is_empty())
    {
        return;
    }
    for entity in &existing_query {
        commands.entity(entity).despawn();
    }
    let Some(npc_id) = dialogue_state.active_npc_id.as_deref() else {
        return;
    };
    let Some(station) = stations::current_station(&stations.0, &sector_state) else {
        return;
    };
    let Some(npc) = station_npcs(&station.ship)
        .into_iter()
        .find(|npc| npc.id == npc_id)
    else {
        return;
    };
    spawn_dialogue_overlay(&mut commands, &asset_server, &dialogue_state, station, &npc);
}

pub(super) fn spawn_dialogue_overlay(
    commands: &mut Commands,
    asset_server: &AssetServer,
    dialogue_state: &DockedDialogueState,
    station: &StationDefinition,
    npc: &DockedStationNpc,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let contact = station.contacts.iter().find(|contact| contact.id == npc.id);
    let fallback_body = contact
        .map(|contact| format!("{}\n\n{}", contact.bio, contact.brief))
        .unwrap_or_else(|| format!("Yarn node `{}` is ready.", npc.dialogue_start_node));
    let speaker = dialogue_state
        .yarn_speaker
        .as_deref()
        .unwrap_or(npc.name.as_str());
    let body = dialogue_state.yarn_line.clone().unwrap_or(fallback_body);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(28.0),
                bottom: Val::Px(28.0),
                width: Val::Px(420.0),
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.07, 0.10, 0.96)),
            DockedDialogueRoot,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(format!("{speaker} - {}", npc.role)),
                TextFont {
                    font: title_font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            panel.spawn((
                Text::new(format!(
                    "Yarn node: {}\n\n{}",
                    npc.dialogue_start_node, body
                )),
                TextFont {
                    font: mono_font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.88, 0.92)),
            ));
            for option in dialogue_state
                .yarn_options
                .iter()
                .filter(|option| option.is_available)
            {
                dialogue_yarn_option_button(panel, option, &title_font);
            }
            if dialogue_state.yarn_options.is_empty() {
                dialogue_button(
                    panel,
                    "Continue",
                    &title_font,
                    DockedDialogueYarnOptionButton {
                        option_id: OptionId(usize::MAX),
                        label: "Continue".to_string(),
                    },
                );
            }

            match npc.service_action {
                StationNpcServiceAction::Engineer => {
                    dialogue_button(
                        panel,
                        "Repair Ship",
                        &title_font,
                        DockedDialogueRepairButton,
                    );
                    dialogue_button(panel, "Refit Ship", &title_font, DockedDialogueRefitButton);
                }
                StationNpcServiceAction::Contracts => {
                    dialogue_button(
                        panel,
                        "Open Contracts",
                        &title_font,
                        DockedDialogueContractsButton,
                    );
                }
                StationNpcServiceAction::Archives => {
                    dialogue_button(
                        panel,
                        "Open Archives",
                        &title_font,
                        DockedDialogueArchivesButton,
                    );
                }
            }
            dialogue_button(panel, "Close", &title_font, DockedDialogueCloseButton);
        });
}

pub(super) fn dialogue_button<T: Component>(
    panel: &mut ChildSpawnerCommands<'_>,
    label: &str,
    font: &Handle<Font>,
    marker: T,
) {
    panel
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(34.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.32, 0.48)),
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(super) fn dialogue_yarn_option_button(
    panel: &mut ChildSpawnerCommands<'_>,
    option: &DockedYarnOption,
    font: &Handle<Font>,
) {
    dialogue_button(
        panel,
        &option.label,
        font,
        DockedDialogueYarnOptionButton {
            option_id: option.option_id,
            label: option.label.clone(),
        },
    );
}

/// Applies local docked dialogue button choices to overlays or existing rollback meta commands.
pub(crate) fn docked_dialogue_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&DockedDialogueCloseButton>,
            Option<&DockedDialogueRepairButton>,
            Option<&DockedDialogueRefitButton>,
            Option<&DockedDialogueContractsButton>,
            Option<&DockedDialogueArchivesButton>,
            Option<&DockedDialogueYarnOptionButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut runner_query: Query<&mut DialogueRunner, With<DockedYarnRunner>>,
    mut dialogue_state: ResMut<DockedDialogueState>,
    mut docked_state: ResMut<DockedState>,
    mut pending_meta: ResMut<PendingLocalMetaCommand>,
    mut editor_session: ResMut<EditorSessionState>,
) {
    for (interaction, mut background, close, repair, refit, contracts, archives, yarn_option) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.14, 0.24, 0.38));
                if close.is_some() {
                    clear_docked_dialogue_state(&mut dialogue_state);
                } else if repair.is_some() {
                    pending_meta.0 = Some(PendingMetaCommand {
                        op: RollbackMetaOp::RepairShip,
                        ..Default::default()
                    });
                } else if refit.is_some() {
                    editor_session.mode = EditorMode::Player;
                    pending_meta.0 = Some(PendingMetaCommand {
                        op: RollbackMetaOp::OpenEditor,
                        ..Default::default()
                    });
                } else if contracts.is_some() {
                    docked_state.selected_surface = DockedSurface::Contracts;
                    dialogue_state.selected_surface_hint = Some(StationNpcServiceAction::Contracts);
                } else if archives.is_some() {
                    docked_state.selected_surface = DockedSurface::Archives;
                    dialogue_state.selected_surface_hint = Some(StationNpcServiceAction::Archives);
                } else if let Some(yarn_option) = yarn_option {
                    apply_dialogue_service_label(
                        &yarn_option.label,
                        &mut docked_state,
                        &mut pending_meta,
                        &mut editor_session,
                    );
                    if let Some(runner_entity) = dialogue_state.runner_entity
                        && let Ok(mut runner) = runner_query.get_mut(runner_entity)
                    {
                        if yarn_option.option_id == OptionId(usize::MAX) {
                            if runner.is_running() && !runner.is_waiting_for_option_selection() {
                                runner.continue_in_next_update();
                            }
                        } else if runner.is_running() && runner.is_waiting_for_option_selection() {
                            let _ = runner.select_option(yarn_option.option_id);
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.26, 0.42, 0.62));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.20, 0.32, 0.48));
            }
        }
    }
}

pub(super) fn apply_dialogue_service_label(
    label: &str,
    docked_state: &mut DockedState,
    pending_meta: &mut PendingLocalMetaCommand,
    editor_session: &mut EditorSessionState,
) {
    match label.trim().to_ascii_lowercase().as_str() {
        "repair ship" => {
            pending_meta.0 = Some(PendingMetaCommand {
                op: RollbackMetaOp::RepairShip,
                ..Default::default()
            });
        }
        "refit ship" => {
            editor_session.mode = EditorMode::Player;
            pending_meta.0 = Some(PendingMetaCommand {
                op: RollbackMetaOp::OpenEditor,
                ..Default::default()
            });
        }
        "open contracts" => {
            docked_state.selected_surface = DockedSurface::Contracts;
        }
        "open archives" => {
            docked_state.selected_surface = DockedSurface::Archives;
        }
        _ => {}
    }
}

/// Captures Yarn lines from the active docked dialogue runner into the local overlay state.
pub(crate) fn handle_docked_yarn_line(
    event: On<PresentLine>,
    stations: Res<StationCatalogResource>,
    sector_state: Res<SectorState>,
    mut dialogue_state: ResMut<DockedDialogueState>,
    mut runner_query: Query<&mut DialogueRunner, With<DockedYarnRunner>>,
) {
    if dialogue_state.runner_entity != Some(event.entity) {
        return;
    }
    let line = event.line.text_without_character_name();
    let should_auto_continue =
        opening_contact_line_matches(&stations, &sector_state, &dialogue_state, &line);
    dialogue_state.yarn_speaker = event.line.character_name().map(str::to_string);
    dialogue_state.yarn_line = Some(line);
    dialogue_state.yarn_options.clear();
    if should_auto_continue
        && let Ok(mut runner) = runner_query.get_mut(event.entity)
        && runner.is_running()
        && !runner.is_waiting_for_option_selection()
    {
        runner.continue_in_next_update();
    }
}

/// Captures Yarn options from the active docked dialogue runner into clickable overlay buttons.
pub(crate) fn handle_docked_yarn_options(
    event: On<PresentOptions>,
    mut dialogue_state: ResMut<DockedDialogueState>,
) {
    if dialogue_state.runner_entity != Some(event.entity) {
        return;
    }
    dialogue_state.yarn_options = event
        .options
        .iter()
        .map(|option| DockedYarnOption {
            option_id: option.id,
            label: option.line.text_without_character_name(),
            is_available: option.is_available,
        })
        .collect();
}

/// Marks a completed Yarn conversation as idle while leaving the last displayed line visible.
pub(crate) fn handle_docked_yarn_dialogue_completed(
    event: On<DialogueCompleted>,
    mut dialogue_state: ResMut<DockedDialogueState>,
) {
    if dialogue_state.runner_entity != Some(event.entity) {
        return;
    }
    dialogue_state.runner_entity = None;
    dialogue_state.active_yarn_node = None;
    dialogue_state.yarn_options.clear();
}
