use bevy::{color::palettes::tailwind, prelude::*};
use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};

use crate::demo::{
    ai::{
        alertness::{
            Alertness, ChangeAlertness, ChangeOrRegainAwarenessObject, HighAlertnessToPlayer,
        },
        vision::visibility::AiVisibility,
    },
    npc::Npc,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_ai_debug_ui);
    app.add_systems(Update, update_ai_debug_ui);

    app.add_observer(on_alertness_change)
        .add_observer(on_awareness_change)
        .add_observer(on_high_alertness_to_player);
}

fn setup_ai_debug_ui(add: On<Add, Npc>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .insert(AnchoredUiNodes::spawn_one((
            AnchorUiConfig {
                anchorpoint: AnchorPoint::bottommid(),
                offset: Some(Vec3::new(0.0, 0.5, 0.0)),
                ..default()
            },
            children![(
                Node {
                    bottom: px(30.0),
                    row_gap: px(10.0),
                    flex_direction: FlexDirection::ColumnReverse,
                    min_width: px(100.0),
                    ..default()
                },
                children![
                    (
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(10.0),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Hearing:"),
                                TextColor::from(tailwind::SLATE_200),
                                TextFont::from_font_size(15.0)
                            ),
                            (
                                Node {
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                children![
                                    (
                                        Node {
                                            height: px(10.0),
                                            width: px(100.0),
                                            position_type: PositionType::Absolute,
                                            ..default()
                                        },
                                        BackgroundColor(tailwind::RED_950.into()),
                                    ),
                                    (
                                        HearingBar,
                                        Node {
                                            height: px(10.0),
                                            width: px(0.0),
                                            position_type: PositionType::Absolute,
                                            ..default()
                                        },
                                        BackgroundColor(tailwind::RED_500.into()),
                                        ZIndex(1)
                                    )
                                ]
                            )
                        ]
                    ),
                    (
                        VisionText,
                        Text::default(),
                        TextColor::from(tailwind::SLATE_200),
                        TextFont::from_font_size(15.0)
                    ),
                ]
            )],
        )));
}

fn update_ai_debug_ui(
    npcs: Query<
        (
            Option<&DebugVision>,
            Option<&DebugHearing>,
            &AnchoredUiNodes,
        ),
        With<Npc>,
    >,
    children: Query<&Children>,
    name: Query<NameOrEntity>,
    mut vision_text: Query<&mut Text, With<VisionText>>,
    mut hearing_bar: Query<&mut Node, With<HearingBar>>,
) -> Result {
    for (vision, hearing, ui) in npcs.iter() {
        let ui = ui.iter().next().ok_or("No anchored UI node found")?;

        let mut vision_text = vision_text.get_mut(
            children
                .iter_descendants(ui)
                .find(|c| vision_text.contains(*c))
                .ok_or("No Vision Debug found")?,
        )?;

        let mut hearing_bar = hearing_bar.get_mut(
            children
                .iter_descendants(ui)
                .find(|c| hearing_bar.contains(*c))
                .ok_or("No Hearing Debug found")?,
        )?;

        vision_text.0 = if let Some(vision) = vision {
            format!(
                "Seeing {vision_name}:\n  light: {light}\n  exp: {exp}\n  mov: {mov}",
                vision_name = name.get(vision.entity).unwrap(),
                light = vision.visibility.lighting,
                exp = vision.visibility.exposure,
                mov = vision.visibility.movement
            )
        } else {
            "Seeing None".to_string()
        };

        hearing_bar.width = if let Some(hearing) = hearing {
            px(hearing.0 * 100.0)
        } else {
            px(0.0)
        };
    }
    Ok(())
}

#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub(crate) struct DebugVision {
    pub(crate) entity: Entity,
    pub(crate) visibility: AiVisibility,
}

#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub(crate) struct DebugHearing(pub(crate) f32);

#[derive(Component)]
struct VisionText;
#[derive(Component)]
struct HearingBar;

fn on_awareness_change(
    change_awareness: On<ChangeOrRegainAwarenessObject>,
    alertness: Query<&Alertness>,
) {
    info!(change_awareness=?change_awareness.event(), alertness=?alertness.get(change_awareness.npc).unwrap());
}

fn on_high_alertness_to_player(
    high_alertness: On<HighAlertnessToPlayer>,
    alertness: Query<&Alertness>,
) {
    info!(high_alertness=?high_alertness.event(), alertness=?alertness.get(high_alertness.npc).unwrap());
}

fn on_alertness_change(change_alertness: On<ChangeAlertness>, alertness: Query<&Alertness>) {
    info!(change_alertness=?change_alertness.event(), alertness=?alertness.get(change_alertness.npc).unwrap());
}
