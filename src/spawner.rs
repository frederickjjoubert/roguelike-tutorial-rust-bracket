use std::collections::HashMap;

use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use super::{
    AreaOfEffect,
    BlocksTile,
    CombatStats,
    Confusion,
    Consumable,
    InflictsDamage,
    Item,
    MAP_WIDTH,
    Monster,
    Name,
    Player,
    Position,
    ProvidesHealing,
    SerializeMe,
    Ranged,
    Rect,
    Renderer,
    RandomTable,
    Viewshed,
};

const MAX_MONSTERS: i32 = 4;

pub fn fill_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut random_number_generator = ecs.write_resource::<RandomNumberGenerator>();
        let total_spawns = random_number_generator.roll_dice(1, MAX_MONSTERS + 3) - 3;

        for _i in 0..total_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + random_number_generator.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + random_number_generator.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let index = (y * MAP_WIDTH) + x;
                if !spawn_points.contains_key(&index) {
                    spawn_points.insert(index, spawn_table.roll(&mut random_number_generator));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAP_WIDTH) as i32;
        let y = (*spawn.0 / MAP_WIDTH) as i32;

        match spawn.1.as_ref() {
            "Goblin" => spawn_goblin(ecs, x, y),
            "Orc" => spawn_orc(ecs, x, y),
            "Health Potion" => spawn_health_potion(ecs, x, y),
            "Fireball Scroll" => spawn_fireball_scroll(ecs, x, y),
            "Confusion Scroll" => spawn_confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => spawn_magic_missile_scroll(ecs, x, y),
            _ => {}
        }
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
}

fn spawn_confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        // Components
        .with(Confusion {
            turns: 4
        })
        .with(Consumable {})
        .with(Item {})
        .with(Name { name: "Confusion Scroll".to_string() })
        .with(Position { x, y })
        .with(Ranged {
            range: 6,
        })
        .with(Renderer {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        // Markers
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn spawn_fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        // Components
        .with(AreaOfEffect {
            radius: 3
        })
        .with(Consumable {})
        .with(InflictsDamage {
            damage: 20
        })
        .with(Item {})
        .with(Name { name: "Fireball Scroll".to_string() })
        .with(Position { x, y })
        .with(Ranged {
            range: 6,
        })
        .with(Renderer {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        // Markers
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn spawn_magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        // Components
        .create_entity()
        .with(Consumable {})
        .with(InflictsDamage {
            damage: 8
        })
        .with(Item {})
        .with(Name { name: "Magic Missile Scroll".to_string() })
        .with(Position { x, y })
        .with(Ranged {
            range: 6,
        })
        .with(Renderer {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        // Markers
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn spawn_health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        // Components
        .with(Consumable {})
        .with(Item {})
        .with(Name { name: "Health Potion".to_string() })
        .with(Position { x, y })
        .with(ProvidesHealing {
            heal_amount: 8
        })
        .with(Renderer {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        // Markers
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
        // Components
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Name {
            name: "Player".to_string()
        })
        .with(Player {})
        .with(Position { x: player_x, y: player_y })
        .with(Renderer {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        // Markers
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn spawn_goblin(ecs: &mut World, x: i32, y: i32) {
    spawn_monster(
        ecs,
        x, y,
        rltk::to_cp437('g'),
        "Goblin".to_string());
}

fn spawn_orc(ecs: &mut World, x: i32, y: i32) {
    spawn_monster(
        ecs,
        x, y,
        rltk::to_cp437('o'),
        "Orc".to_string());
}

fn spawn_monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs
        // Components
        .create_entity()
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(Monster {})
        .with(Name {
            name: format!("{}", name.to_string())
        })
        .with(Position { x, y })
        .with(Renderer {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        // Markers
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}