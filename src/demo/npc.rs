use crate::asset_tracking::LoadResource as _;
use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Npc>();
    app.load_asset::<Gltf>(MODEL_PATH);
    app.add_observer(spawn_npc);
}

const MODEL_PATH: &'static str = "models/npc.glb";

#[point_class(model(MODEL_PATH))]
pub struct Npc;

fn spawn_npc(trigger: Trigger<OnAdd, Npc>, mut commands: Commands, assets: Res<AssetServer>) {
    commands.entity(trigger.target()).with_child(SceneRoot(
        assets.load(GltfAssetLabel::Scene(0).from_asset(MODEL_PATH)),
    ));
}
