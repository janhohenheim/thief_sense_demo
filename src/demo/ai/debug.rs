use bevy::{color::palettes::tailwind, prelude::*};
use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};

use crate::demo::{ai::vision::visibility::AiVisibility, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_ai_debug_ui);
    app.add_systems(PostUpdate, update_ai_debug_ui);
}

fn setup_ai_debug_ui(add: On<Add, Npc>, mut commands: Commands) {
    // Yes, all of these `Pickable::IGNORE` are needed for the movement gizmo to work :/
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
                    top: px(10.0),
                    left: px(30.0),
                    column_gap: px(10.0),
                    min_height: px(100.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    (
                        HearingBar,
                        Node {
                            height: px(0.0),
                            width: px(10.0),
                            ..default()
                        },
                        BackgroundColor(tailwind::RED_600.into()),
                        Pickable::IGNORE,
                    ),
                    (VisionText, Text::default(), Pickable::IGNORE,)
                ]
            )],
            Pickable::IGNORE,
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
                "{vision_name}:\nlight: {light}\nexp: {exp}\nmov: {mov}",
                vision_name = name.get(vision.entity).unwrap(),
                light = vision.visibility.lighting,
                exp = vision.visibility.exposure,
                mov = vision.visibility.movement
            )
        } else {
            "No vision".to_string()
        };

        hearing_bar.height = if let Some(hearing) = hearing {
            px(hearing.0 * 3000.0)
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
