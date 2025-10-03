use bevy::{ecs::entity::EntityHashSet, prelude::*};
use evergreen_relations::prelude::*;

use crate::demo::ai::awareness::{AwarenessData, AwarenessLink};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(bookkeep_awareness_data);
}

fn bookkeep_awareness_data(
    add: On<Insert, AwarenessLink>,
    mut data: Query<(&AwarenessLink, &mut AwarenessData)>,
    name: Query<NameOrEntity>,
) {
    let Ok((friend, mut data)) = data.get_mut(add.entity) else {
        return;
    };
    let friend_set = friend.iter().collect::<EntityHashSet>();
    let data_set = data.keys().copied().collect::<EntityHashSet>();
    let non_data_friends = friend_set
        .difference(&data_set)
        .copied()
        .collect::<EntityHashSet>();
    let non_friend_data = data_set
        .difference(&friend_set)
        .copied()
        .collect::<EntityHashSet>();

    for non_data_friend in non_data_friends {
        let from_name = name.get(add.entity).unwrap();
        let to_name = name.get(non_data_friend).unwrap();
        error!("No AwarenessData for new awareness link from {from_name} to {to_name}");
    }

    for non_friend_data in non_friend_data {
        data.remove(&non_friend_data);
    }
}

#[derive(Relation)]
#[relation(source = __AwarenessRelatable, target = __AwarenessRelatable)]
#[doc(hidden)]
pub(crate) struct __AwarenessRelation;

#[derive(Relatable)]
#[relatable(Vec<Entity> in __AwarenessRelation, opposite = Self)]
#[doc(hidden)]
pub(crate) struct __AwarenessRelatable;
