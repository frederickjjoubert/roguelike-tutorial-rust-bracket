use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use super::{
    CombatStats,
    GameLog,
    Item,
    Map,
    Player,
    Position,
    RunState,
    State,
    Viewshed,
    WantsToMelee,
    WantsToPickupItem};

pub fn player_input(game_state: &mut State, context: &mut Rltk) -> RunState {
    match context.key {
        None => {
            return RunState::AwaitingInput;
        } // No Input, Do Nothing.
        Some(key) => {
            match key {
                // === Movement ===
                // Cardinal Directions
                VirtualKeyCode::Left |
                VirtualKeyCode::Numpad4 |
                VirtualKeyCode::H => try_move_player(-1, 0, &mut game_state.ecs),

                VirtualKeyCode::Right |
                VirtualKeyCode::Numpad6 |
                VirtualKeyCode::L => try_move_player(1, 0, &mut game_state.ecs),

                VirtualKeyCode::Up |
                VirtualKeyCode::Numpad8 |
                VirtualKeyCode::K => try_move_player(0, -1, &mut game_state.ecs),

                VirtualKeyCode::Down |
                VirtualKeyCode::Numpad2 |
                VirtualKeyCode::J => try_move_player(0, 1, &mut game_state.ecs),

                // Diagonals Directions
                VirtualKeyCode::Numpad9 |
                VirtualKeyCode::U => try_move_player(1, -1, &mut game_state.ecs),

                VirtualKeyCode::Numpad7 |
                VirtualKeyCode::Y => try_move_player(-1, -1, &mut game_state.ecs),

                VirtualKeyCode::Numpad3 |
                VirtualKeyCode::N => try_move_player(1, 1, &mut game_state.ecs),

                VirtualKeyCode::Numpad1 |
                VirtualKeyCode::B => try_move_player(-1, 1, &mut game_state.ecs),

                // === Interactions ===
                // Item Pickup
                VirtualKeyCode::G => try_pickup_item(&mut game_state.ecs),

                // === UI ===
                VirtualKeyCode::I => return RunState::ShowInventory,
                VirtualKeyCode::D => return RunState::ShowDropItem,

                // === State ===
                VirtualKeyCode::Escape => return RunState::SaveGame,

                _ => {
                    return RunState::AwaitingInput;
                } // Anything else, Do Nothing.
            }
        }
    }
    RunState::PlayerTurn
}

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut players = ecs.write_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, position, viewshed)
    in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        let x = position.x + dx;
        let y = position.y + dy;
        let idx = map.xy_idx(x, y);

        // Check if the tile contains an Entity with CombatStats
        for potential_target in map.tile_contents[idx].iter() {
            let target = combat_stats.get(*potential_target);
            match target
            {
                // Tile contains an Entity with CombatStats, add a WantsToMelee component to the player, with the potential target as the target.
                Some(_target) => {
                    wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).expect("Add target failed");
                    return;
                }
                None => {}
            }
        }

        // Check the tile isn't blocked
        if !map.blocked_tiles[idx] {
            // Check you haven't left the screen.
            position.x = min(79, max(0, position.x + dx));
            position.y = min(49, max(0, position.y + dy));
            viewshed.dirty = true;
            let mut player_position = ecs.write_resource::<Point>();
            player_position.x = position.x;
            player_position.y = position.y;
        }
    }
}

fn try_pickup_item(ecs: &mut World) {
    // Call into ECS
    let entities = ecs.entities();
    let player_position = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let mut game_log = ecs.fetch_mut::<GameLog>();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();

    // Try to find an item to pick up.
    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_position.x && position.y == player_position.y {
            target_item = Some(item_entity);
        }
    }

    // Check if we found an item to pick up.
    match target_item {
        None => game_log.entries.push("There is nothing to pick up.".to_string()),
        Some(item) => {
            let mut wants_to_pickup_item = ecs.write_storage::<WantsToPickupItem>();
            wants_to_pickup_item.insert(
                *player_entity,
                WantsToPickupItem {
                    collected_by: *player_entity,
                    item,
                },
            ).expect("Unable to insert WantsToPickupItem component.");
        }
    }
}




