use rltk::{VirtualKeyCode, Rltk};
use specs::prelude::*;
use super::{Position, Player, TileType, xy_idx, State};
use std::cmp::{min, max};

pub fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, position) in (&mut players, &mut positions).join() {
        let x = position.x + dx;
        let y = position.y + dy;
        let idx = xy_idx(x, y);

        // Check the tile isn't blocked
        if map[idx] != TileType::Wall {
            // Check you haven't left the screen.
            position.x = min(79, max(0, position.x + dx));
            position.y = min(49, max(0, position.y + dy));
        }
    }
}

pub fn player_input(game_state: &mut State, context: &mut Rltk) {
    match context.key {
        None => {} // No Input, Do Nothing.
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
                _ => {} // Anything else, Do Nothing.
            }
        }
    }
}

