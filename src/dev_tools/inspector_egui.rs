use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<InspectorActive>();
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::new().run_if(is_inspector_active),
    ));
    app.add_systems(
        Update,
        toggle_inspector.run_if(input_just_pressed(KeyCode::F3)),
    );
}

#[derive(Resource, Debug, Default, Eq, PartialEq)]
struct InspectorActive(bool);

fn is_inspector_active(inspector_active: Res<InspectorActive>) -> bool {
    inspector_active.0
}

fn toggle_inspector(mut inspector_active: ResMut<InspectorActive>) {
    inspector_active.0 = !inspector_active.0;
}
