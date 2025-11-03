use bevy::{
    ecs::{lifecycle::HookContext, relationship::Relationship as _, world::DeferredWorld},
    prelude::*,
};
use bitflags::bitflags;
use std::time::Duration;
use strum::EnumCount as _;

use crate::Player;
use crate::demo::{
    ai::awareness::{Awareness, AwarenessFlags, AwarenessLevel, AwarenessToObject, NpcToAwareness},
    npc::Npc,
    team::{Team, TeamRelation},
};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, Alertness>();
    app.register_required_components::<Npc, FreeKnowledgeDurations>();
}

pub(crate) fn update_alertness(In(npc): In<Entity>, world: &mut World) -> Result {
    let highest_awareness: Option<_> = world.run_system_cached_with(
        get_highest_awareness,
        HighestAwarenessInput::new(npc).with_flags(HighestAwarenessFlags::ALERTING),
    )?;
    () = world.run_system_cached_with(
        update_alertness_to,
        UpdateAlertnessToInput {
            npc,
            awareness: highest_awareness,
        },
    )?;

    Ok(())
}

/// Only triggered the *first time* the NPC becomes highly alert to the player.
#[derive(EntityEvent, Debug)]
pub(crate) struct HighAlertnessToPlayer {
    #[event_target]
    pub(crate) npc: Entity,
}

impl From<Entity> for HighAlertnessToPlayer {
    fn from(entity: Entity) -> Self {
        Self { npc: entity }
    }
}

#[derive(EntityEvent, Debug)]
pub(crate) struct ChangeAlertness {
    #[event_target]
    pub(crate) npc: Entity,
    pub(crate) object: Option<Entity>,
    pub(crate) previous_level: AwarenessLevel,
}

#[derive(EntityEvent, Debug)]
pub(crate) struct ChangeOrRegainAwarenessObject {
    #[event_target]
    pub(crate) npc: Entity,
    pub(crate) object: Option<Entity>,
    pub(crate) previous_level: AwarenessLevel,
}

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
#[require(FreeKnowledgeDurations)]
#[component(on_add = Alertness::on_add)]
pub(crate) struct Alertness {
    pub(crate) level: AwarenessLevel,
    pub(crate) peak: AwarenessLevel,
    pub(crate) free_knowledge: Duration,
    pub(crate) ever_got_high_alerted_by_player: bool,
    pub(crate) lost_contact_to_awareness_object: bool,
    pub(crate) last_awareness_object: Option<Entity>,
}

impl Default for Alertness {
    fn default() -> Self {
        Self {
            level: AwarenessLevel::default(),
            peak: AwarenessLevel::default(),
            free_knowledge: FreeKnowledgeDurations::default()[AwarenessLevel::default() as usize],
            ever_got_high_alerted_by_player: false,
            lost_contact_to_awareness_object: false,
            last_awareness_object: None,
        }
    }
}

impl Alertness {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let durations = *world.get::<FreeKnowledgeDurations>(ctx.entity).unwrap();
        let mut alertness = world.get_mut::<Alertness>(ctx.entity).unwrap();
        alertness.free_knowledge = durations[alertness.level as usize];
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect, Deref, DerefMut)]
#[reflect(Component)]
// Note: original also multiplies this by 1.666666 when in combat.
pub(crate) struct FreeKnowledgeDurations([Duration; AwarenessLevel::COUNT]);

impl Default for FreeKnowledgeDurations {
    fn default() -> Self {
        Self([
            Duration::from_millis(1500), // x 1.0
            Duration::from_millis(1500), // x 1.0
            Duration::from_millis(1875), // x 1.25
            Duration::from_millis(3000), // x 2.0
        ])
    }
}

#[derive(Debug)]
struct HighestAwarenessInput {
    entity: Entity,
    flags: HighestAwarenessFlags,
    max_last_contact: Duration,
}

impl HighestAwarenessInput {
    fn new(entity: Entity) -> Self {
        Self {
            entity,
            flags: HighestAwarenessFlags::default(),
            max_last_contact: Duration::MAX,
        }
    }

    fn with_flags(mut self, flags: HighestAwarenessFlags) -> Self {
        self.flags = flags;
        self
    }
}

fn get_highest_awareness(
    In(HighestAwarenessInput {
        entity: npc,
        flags,
        max_last_contact,
    }): In<HighestAwarenessInput>,
    npcs: Query<(&GlobalTransform, &Team, Option<&NpcToAwareness>)>,
    objects: Query<&Team>,
    awarenesses: Query<(&Awareness, &AwarenessToObject)>,
    players: Query<(), With<Player>>,
) -> Result<Option<(Entity, Awareness)>> {
    let (npc_transform, npc_team, npc_to_awareness) = npcs.get(npc)?;
    let Some(npc_to_awareness) = npc_to_awareness else {
        return Ok(None);
    };
    let mut highest_awareness = None;
    for (awareness, awareness_to_object) in awarenesses.iter_many(npc_to_awareness.get()) {
        let object = awareness_to_object.get();
        let team = match objects.get(object) {
            Ok(team) => team,
            Err(_) => {
                error!("Object does not belong to any team");
                continue;
            }
        };
        let team_rel = team.relation_to(*npc_team);
        if flags.contains(HighestAwarenessFlags::OPPONENTS_AND_ALARMS)
            && team_rel != TeamRelation::Enemy
        {
            continue;
        }

        // TODO: this also checks if
        // - The AI should only notice players, and continue if this is not a player
        // - If it also notices non-players, continue if it's a teammate that is either not dead or the AI doesn't notice bodies
        // Though idk if we need to care about that? Why can an alerting AI not notice alive teammates? I guess it's handled somewhere else?
        // Do we need to skip indifferents? Why does the `::ALERTING` flag even exist?
        if flags.contains(HighestAwarenessFlags::ALERTING) && team_rel == TeamRelation::Indifferent
        {
            continue;
        }
        // TODO: skip dead enemies too

        if awareness.last_contact.elapsed() > max_last_contact {
            continue;
        }

        if flags.contains(HighestAwarenessFlags::FIRST_HAND)
            && !awareness.flags.contains(AwarenessFlags::FIRST_HAND)
        {
            continue;
        }
        let (highest_object, highest_awareness) =
            highest_awareness.get_or_insert((object, awareness.clone()));

        let mut new_highest = false;
        if awareness.level > highest_awareness.level {
            new_highest = true;
        } else if awareness.level == highest_awareness.level {
            match highest_awareness.level {
                AwarenessLevel::Lowest | AwarenessLevel::Low | AwarenessLevel::High => {
                    if players.contains(object) {
                        new_highest = true;
                    }
                }
                AwarenessLevel::Moderate => {
                    if awareness.last_contact.elapsed() < highest_awareness.last_contact.elapsed()
                        || npc_transform
                            .translation()
                            .distance_squared(awareness.last_pos)
                            < npc_transform
                                .translation()
                                .distance_squared(highest_awareness.last_pos)
                    {
                        new_highest = true;
                    }
                }
            }
        }
        if new_highest {
            *highest_object = object;
            *highest_awareness = awareness.clone();
        }
    }
    Ok(highest_awareness)
}

struct UpdateAlertnessToInput {
    npc: Entity,
    awareness: Option<(Entity, Awareness)>,
}

fn update_alertness_to(
    In(UpdateAlertnessToInput { npc, awareness }): In<UpdateAlertnessToInput>,
    mut commands: Commands,
    mut npcs: Query<(&mut Alertness, &FreeKnowledgeDurations)>,
    players: Query<(), With<Player>>,
) -> Result {
    let (mut alertness, free_knowledge_durations) = npcs.get_mut(npc)?;
    let previous_level = alertness.level;
    let (object, target_level, awareness) = if let Some((object, awareness)) = awareness {
        // Original now forces our alertness to be high if it was high before and a hardcoded 30 s have not yet passed since the last contact.
        // But imo that's already taken care of by the cap, which is 45 s by default. Sure, this does not
        // Also, it does *not* force the alertness when there happens to be no awareness of any object, which seems like a bug.
        (Some(object), awareness.level, Some(awareness))
    } else {
        (None, AwarenessLevel::Lowest, None)
    };
    // original now clamps alertness per-npc, but I can't imagine actually overriding that

    alertness.level = if alertness.peak == AwarenessLevel::High {
        target_level.max(AwarenessLevel::Low)
    } else {
        target_level
    };
    alertness.free_knowledge = free_knowledge_durations[alertness.level as usize];
    alertness.peak = alertness.peak.max(alertness.level);

    if alertness.level == AwarenessLevel::High
        && object.is_some_and(|object| players.contains(object))
        && !alertness.ever_got_high_alerted_by_player
    {
        alertness.ever_got_high_alerted_by_player = true;
        commands.entity(npc).trigger(HighAlertnessToPlayer::from);
    }

    if previous_level != alertness.level {
        commands.entity(npc).trigger(|npc| ChangeAlertness {
            npc,
            object,
            previous_level,
        });
    }

    if awareness.as_ref().is_some_and(|awareness| {
        awareness.last_contact.elapsed() != Duration::ZERO
            || !awareness.flags.contains(AwarenessFlags::SEEN)
    }) {
        alertness.lost_contact_to_awareness_object = true;
    }

    if object != alertness.last_awareness_object
        || (alertness.lost_contact_to_awareness_object
            && awareness.as_ref().is_some_and(|awareness| {
                awareness.last_contact.elapsed() == Duration::ZERO
                    && awareness.flags.contains(AwarenessFlags::SEEN)
            }))
    {
        alertness.last_awareness_object = object;
        commands
            .entity(npc)
            .trigger(|npc| ChangeOrRegainAwarenessObject {
                npc,
                object,
                previous_level,
            });
    }

    if awareness.as_ref().is_some_and(|awareness| {
        awareness.last_contact.elapsed() == Duration::ZERO
            && awareness.flags.contains(AwarenessFlags::SEEN)
    }) {
        alertness.lost_contact_to_awareness_object = false;
    }

    Ok(())
}

bitflags! {
    #[derive(Debug)]
    struct HighestAwarenessFlags: u8 {
        const OPPONENTS_AND_ALARMS = 1 << 0;
        const FIRST_HAND = 1 << 1;
        const ALERTING = 1 << 2;
    }
}

impl Default for HighestAwarenessFlags {
    fn default() -> Self {
        Self::OPPONENTS_AND_ALARMS
    }
}
