use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use crate::{RunState, Viewshed};
use super::{Position, Player, TileType, State, Map};

pub fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut players = ecs.write_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();

    let map = ecs.fetch::<Map>();

    for (_player, position, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let x = position.x + dx;
        let y = position.y + dy;
        let idx = map.xy_idx(x, y);

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
                VirtualKeyCode::Left |
                VirtualKeyCode::Numpad4 => {
                    try_move_player(-1, 0, &mut game_state.ecs);
                }
                VirtualKeyCode::Right |
                VirtualKeyCode::Numpad6 => {
                    try_move_player(1, 0, &mut game_state.ecs);
                }
                VirtualKeyCode::Up |
                VirtualKeyCode::Numpad8 => {
                    try_move_player(0, -1, &mut game_state.ecs);
                }
                VirtualKeyCode::Down |
                VirtualKeyCode::Numpad2 => {
                    try_move_player(0, 1, &mut game_state.ecs);
                }
                _ => {
                    return RunState::Paused;
                } // Anything else, Do Nothing.
            }
        }
    }
    RunState::Running
}

