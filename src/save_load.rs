use std::fs::File;

use macroquad::prelude::IVec2;
use specs::{
    prelude::*,
    saveload::*,
};
use crate::{comp::*, map::Map};


macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<std::convert::Infallible, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<std::convert::Infallible, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &$data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) {
    {
        let map = ecs.get_mut::<Map>().unwrap().clone();
        let writer = File::create("./saved_map.json").unwrap();
        serde_json::to_writer(writer, &map).unwrap();
    }
    let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>());
    let writer = File::create("./saved_entities.json").unwrap();
    let mut serializer = serde_json::Serializer::new(writer);
    serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Monster, 
        Named, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, 
        AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
        WantsToDropItem, Equippable, Equipped, AttackBonus, DefenseBonus, HungerClock, Nutritious,
        EntryTrigger, SingleActivation
    );
}


pub fn load_game(ecs: &mut World) {
    {
        let data = std::fs::read_to_string("./saved_map.json").unwrap();
        let mut map: Map = serde_json::from_str(&data).unwrap();
        map.realloc_content_index();
        ecs.insert(map);
    }
    ecs.delete_all();
    let data = std::fs::read_to_string("./saved_entities.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (&mut ecs.entities(), 
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(), 
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>());
        
        deserialize_individually!(ecs, de, d, Position, Renderable, Player, Viewshed, Monster, 
            Named, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage, 
            AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
            WantsToDropItem, Equippable, Equipped, AttackBonus, DefenseBonus, HungerClock, Nutritious,
            EntryTrigger, SingleActivation
        );
    }

    let (player, plp) = {
        let entities = ecs.entities();
        let positions = ecs.read_storage::<Position>();
        let players = ecs.read_storage::<Player>();

        let (e, plp, _) = (&entities, &positions, &players).join().next().unwrap();
        (e, *plp)
    };
    ecs.insert(IVec2::new(plp.x, plp.y));
    ecs.insert(player);
}
