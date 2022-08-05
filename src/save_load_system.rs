use std::fs::File;
use std::path::Path;
use std::fs;

use specs::prelude::*;
use specs::saveload::{
    SimpleMarker,
    SimpleMarkerAllocator,
    SerializeComponents,
    DeserializeComponents,
    MarkedBuilder,
};
use specs::error::NoError;

use super::components::*;
use super::map::{Map, MAP_COUNT};

// The short version of what this macro does is that it takes your ECS as the first parameter,
// and a tuple with your entity store and "markers" stores in it (you'll see this in a moment).
// Every parameter after that is a type - listing a type stored in your ECS.
// These are repeating rules, so it issues one SerializeComponent::serialize call per type.
// It's not as efficient as doing them all at once, but it works - and doesn't fall over when you exceed 16 types!
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

// Essentially the reverse of the macro above.
macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocator
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) {
    // Create helper
    let map_copy = ecs.get_mut::<Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    // Perform serialization
    {
        let entities = ecs.entities();
        let serialize_me_components = ecs.read_storage::<SimpleMarker<SerializeMe>>();
        let data = (
            entities,
            serialize_me_components
        );

        let file_writer = File::create("./save_game.json").unwrap();
        let mut serializer = serde_json::Serializer::new(file_writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            // Components
            AreaOfEffect,
            BlocksTile,
            CombatStats,
            Confusion,
            Consumable,
            InBackpack,
            InflictsDamage,
            Item,
            Monster,
            Name,
            Player,
            Position,
            ProvidesHealing,
            Ranged,
            Renderer,
            SerializationHelper,
            SufferDamage,
            Viewshed,
            WantsToDropItem,
            WantsToMelee,
            WantsToPickupItem,
            WantsToUseItem
        );
    }

    // Clean up
    ecs.delete_entity(save_helper).expect(
        "Crash on Cleanup - Unable to delete Serialization Helper Entity"
    );
}

pub fn load_game(ecs: &mut World) {
    // Delete everything
    {
        let mut entities_to_delete = Vec::new();
        for entity in ecs.entities().join() {
            entities_to_delete.push(entity);
        }
        for entity in entities_to_delete.iter() {
            ecs.delete_entity(*entity).expect("Unable to delete Entity");
        }
    }

    // Read and Deserialize Save Game to Restore all entities & their components
    let save_file_string = fs::read_to_string("./save_game.json").unwrap();
    let mut deserialized_save_file = serde_json::Deserializer::from_str(&save_file_string);

    {
        let entities = &mut ecs.entities();
        let simple_marker_serialize_me_components = &mut ecs.write_storage::<SimpleMarker<SerializeMe>>();
        let simple_marker_allocator_serialize_me_components = &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>();
        let mut data = (
            entities,
            simple_marker_serialize_me_components,
            simple_marker_allocator_serialize_me_components,
        );
        deserialize_individually!(
            ecs,
            deserialized_save_file,
            data,
            // Components
            AreaOfEffect,
            BlocksTile,
            CombatStats,
            Confusion,
            Consumable,
            InBackpack,
            InflictsDamage,
            Item,
            Monster,
            Name,
            Player,
            Position,
            ProvidesHealing,
            Ranged,
            Renderer,
            SerializationHelper,
            SufferDamage,
            Viewshed,
            WantsToDropItem,
            WantsToMelee,
            WantsToPickupItem,
            WantsToUseItem
        );
    }

    let mut entity_to_delete: Option<Entity> = None;
    {
        // Restore map
        let entities = ecs.entities();
        let serialization_helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let positions = ecs.read_storage::<Position>();
        for (entity, helper)
        in (&entities, &serialization_helper).join() {
            let mut world_map = ecs.write_resource::<Map>();
            *world_map = helper.map.clone();
            world_map.tile_contents = vec![Vec::new(); MAP_COUNT]; // Since we aren't serializing tile_content, we replace it with an empty set of vectors.
            entity_to_delete = Some(entity);
        }
        // Add (the loaded) player entity and position resources to ECS.
        for (entity, _player, position)
        in (&entities, &player, &positions).join() {
            let mut player_position = ecs.write_resource::<rltk::Point>();
            *player_position = rltk::Point::new(position.x, position.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = entity;
        }
    }

    // Clean up
    ecs.delete_entity(entity_to_delete.unwrap()).expect(
        "Crash on Cleanup - Unable to delete Serialization Helper Entity"
    )
}

pub fn delete_save() {
    if Path::new("./save_game.json").exists() {
        fs::remove_file("./save_game.json").expect("Unable to delete file");
    }
}

pub fn does_save_exist() -> bool {
    Path::new("./save_game.json").exists()
}