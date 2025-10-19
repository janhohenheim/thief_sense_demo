use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[allow(dead_code)]
pub(crate) trait LogComponentsCommands {
    /// Logs the components of the entity at the [`info`] level.
    fn log_components_pretty(&mut self) -> &mut Self;
}

impl<'a> LogComponentsCommands for EntityCommands<'a> {
    #[allow(dead_code)]
    fn log_components_pretty(&mut self) -> &mut Self {
        self.queue(log_components_pretty())
    }
}

/// An [`EntityCommand`] that logs the components of an entity.
fn log_components_pretty() -> impl EntityCommand {
    move |entity: EntityWorldMut| {
        let name = entity.get::<Name>().map(ToString::to_string);
        let id = entity.id();
        let mut components: Vec<_> = entity
            .world()
            .inspect_entity(id)
            .expect("Entity existence is verified before an EntityCommand is executed")
            .map(|info| info.name().to_string())
            .collect();
        components.sort();

        if let Some(name) = name {
            info!(id=?id, name=?name, ?components, "log_components");
        } else {
            info!(id=?id, ?components, "log_components");
        }
    }
}
