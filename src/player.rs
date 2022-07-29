use rltk::{console, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use crate::{CombatStats, RunState, Viewshed, WantsToMelee};
use super::{Position, Player, State, Map};

pub fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let entities = ecs.entities();

    let map = ecs.fetch::<Map>();

    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut players = ecs.write_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, position, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
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

pub fn player_input(game_state: &mut State, context: &mut Rltk) -> RunState {
    match context.key {
        None => {
            return RunState::Paused;
        } // No Input, Do Nothing.
        Some(key) => {
            match key {
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

                _ => {
                    return RunState::Paused;
                } // Anything else, Do Nothing.
            }
        }
    }
    RunState::Running
}

