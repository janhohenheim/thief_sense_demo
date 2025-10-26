use std::{iter, ops::Deref};

use bevy::{ecs::query::QueryData, prelude::*};

use crate::{
    GameFixedPreUpdateSystems,
    demo::ai::hearing::{node::InputBuffer, param, rms},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedPreUpdate,
        update_accumulators.in_set(GameFixedPreUpdateSystems::UpdateAccumulators),
    );
    app.add_observer(link_accumulator);
}

#[derive(EntityEvent)]
pub(crate) struct AudioInputsReady(pub(crate) Entity);

impl From<Entity> for AudioInputsReady {
    fn from(entity: Entity) -> Self {
        AudioInputsReady(entity)
    }
}

pub(crate) struct BorrowedAudioInputs<'a> {
    pub(crate) inputs: &'a [f32],
    pub(crate) loudness: f32,
}

#[derive(QueryData)]
pub(crate) struct AudioInputs {
    accumulator: Option<&'static InputBufferAccumulator>,
    is_accumulator: Has<AccumulateAudioInputs>,
    input_buffer: Option<&'static InputBuffer>,
}

static SILENCE: [f32; param::MAX_FRAME_SIZE as usize] = [0.0; _];

impl<'w, 's> AudioInputsItem<'w, 's> {
    pub(crate) fn get<'a>(&'a self) -> Result<BorrowedAudioInputs<'a>> {
        if let Some(accumulator) = self.accumulator {
            Ok(BorrowedAudioInputs {
                inputs: accumulator.inputs.deref(),
                loudness: accumulator.loudness,
            })
        } else if self.is_accumulator {
            Ok(BorrowedAudioInputs {
                inputs: &SILENCE,
                loudness: 0.0,
            })
        } else if let Some(input_buffer) = self.input_buffer {
            Ok(BorrowedAudioInputs {
                inputs: &input_buffer.inputs,
                loudness: input_buffer.loudness,
            })
        } else {
            Err(BevyError::from(
                "Queried entity has neither an accumulator nor an input buffer",
            ))
        }
    }
}

// TODO:
// For this to work, every accumulator must correspond to exactly one audionimbus source.
// Each sample player is
// - SamplePlayer
// - AiAudioPool
// - AiAudible
// - InputBuffer
// - InputBufferOf
// and each accumulator is
// - InputBufferAccumulator
// - AiSource
// otherwise, we process the audio with the SA effects of a single source, which leads to some weird stuff man
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = InputBufferAccumulator)]
struct InputBufferOf(Entity);

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
#[relationship_target(relationship = InputBufferOf, linked_spawn)]
#[require(Transform, GlobalTransform)]
struct InputBufferAccumulator {
    #[relationship]
    input_buffers: Vec<Entity>,
    inputs: DefaultableInputs,
    loudness: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct AccumulateAudioInputs;

/// Needed to satisfy the `Reflect` derive macro.
#[derive(Reflect, Deref, DerefMut, Debug)]
struct DefaultableInputs([f32; param::MAX_FRAME_SIZE as usize]);
impl Default for DefaultableInputs {
    fn default() -> Self {
        Self([0.0; param::MAX_FRAME_SIZE as usize])
    }
}

impl InputBufferAccumulator {
    pub(crate) fn update_loudness(&mut self) {
        self.loudness = rms(self.inputs.deref())
    }
}

fn link_accumulator(
    add: On<Add, InputBuffer>,
    child_of: Query<&ChildOf>,
    accumulators: Query<(), With<AccumulateAudioInputs>>,
    mut commands: Commands,
) {
    let accumulator = iter::once(add.entity)
        .chain(child_of.iter_ancestors(add.entity))
        .find(|e| accumulators.contains(*e));
    let Some(accumulator_entity) = accumulator else {
        commands.entity(add.entity).trigger(AudioInputsReady);
        return;
    };
    commands
        .entity(add.entity)
        .try_insert(InputBufferOf(accumulator_entity));
    commands
        .entity(accumulator_entity)
        .trigger(AudioInputsReady);
}

fn update_accumulators(
    mut accumulators: Query<&mut InputBufferAccumulator>,
    input_buffers: Query<Ref<InputBuffer>>,
) {
    for mut accumulator in accumulators.iter_mut() {
        if input_buffers
            .iter_many(&accumulator.input_buffers)
            .all(|buffer| !buffer.is_changed())
        {
            continue;
        }
        let input_buffer_entities = accumulator.input_buffers.clone();
        let input_buffer_values = input_buffers.iter_many(&input_buffer_entities);
        let accumulator_len = accumulator.inputs.len();
        accumulator.inputs.fill(0.0);

        for input_buffer in input_buffer_values {
            assert_eq!(accumulator_len, input_buffer.inputs.len());
            for (dst, src) in accumulator
                .inputs
                .iter_mut()
                .zip(input_buffer.inputs.iter())
            {
                *dst += src;
            }
        }
        accumulator.update_loudness();
    }
}
