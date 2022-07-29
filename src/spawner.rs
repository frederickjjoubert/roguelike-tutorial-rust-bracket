use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use crate::MAP_WIDTH;
use super::{BlocksTile, CombatStats, Item, Monster, Name, Player, Potion, Position, Rect, Renderer, Viewshed};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

pub fn fill_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();


    // Scope to keep borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let number_of_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let number_of_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _ in 0..number_of_monsters {
            let mut added = false;
            while !added {
                let random_x = rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let random_y = rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let x = (room.x1 + random_x) as usize;
                let y = (room.y1 + random_y) as usize;
                let index = (y * MAP_WIDTH) + x;
                if !monster_spawn_points.contains(&index) {
                    monster_spawn_points.push(index);
                    added = true;
                }
            }
        }

        for _ in 0..number_of_items {
            let mut added = false;
            while !added {
                let random_x = rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let random_y = rng.roll_dice(1, i32::abs(room.y2 - room.y1));
                let x = (room.x1 + random_x) as usize;
                let y = (room.y1 + random_y) as usize;
                let index = (y * MAP_WIDTH) + x;
                if !item_spawn_points.contains(&index) {
                    item_spawn_points.push(index);
                    added = true;
                }
            }
        }
    }

    // Spawn the Monsters
    for index in monster_spawn_points.iter() {
        let x = (*index % MAP_WIDTH) as i32;
        let y = (*index / MAP_WIDTH) as i32;
        spawn_random_monster(ecs, x, y);
    }

    // Spawn the Items
    for index in item_spawn_points.iter() {
        let x = (*index % MAP_WIDTH) as i32;
        let y = (*index / MAP_WIDTH) as i32;
        spawn_health_potion(ecs, x, y);
    }
}

fn spawn_health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Item {})
        .with(Name { name: "Health Potion".to_string() })
        .with(Potion {
            heal_amount: 8
        })
        .with(Position { x, y })
        .with(Renderer {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
        })
        .build();
}

pub fn spawn_player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
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
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build()
}

pub fn spawn_random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => {
            spawn_orc(ecs, x, y);
        }
        _ => {
            spawn_goblin(ecs, x, y);
        }
    }
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
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();
}