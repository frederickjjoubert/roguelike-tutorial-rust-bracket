use rltk::{RGB, Rltk, RandomNumberGenerator};
use super::{Rect};
use std::cmp::{min, max};
use std::f32::MIN;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

/// This is simple: it multiplies the y position by the map width (80), and adds x.
/// This guarantees one tile per location, and efficiently maps it in memory for left-to-right reading.
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

/// Makes a map with solid boundaries and 400 randomly placed walls.
/// No guarantees that it won't look awful.
pub fn new_test_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80 * 50];

    // Make the boundaries solid walls.
    for i in 0..80 {
        map[xy_idx(i, 0)] = TileType::Wall;
        map[xy_idx(i, 49)] = TileType::Wall;
    };
    for j in 0..50 {
        map[xy_idx(0, j)] = TileType::Wall;
        map[xy_idx(79, j)] = TileType::Wall;
    };

    // Create RNG
    let mut rng = RandomNumberGenerator::new();

    // Randomly place some walls to test the map generation.
    for _ in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80 * 50];

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, 80 - w - 1) - 1;
        let y = rng.roll_dice(1, 50 - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut rooms_intersect = false;
        for other_room in rooms.iter() {
            if new_room.intersects(other_room) {
                rooms_intersect = true;
                break;
            }
        }
        if !rooms_intersect {
            apply_room_to_map(&new_room, &mut map);


            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rng.range(0, 2) == 1 {
                    apply_horizontal_corridor(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_corridor(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_corridor(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_corridor(&mut map, prev_x, new_x, new_y);
                }
            }
            // Special Case, this is the first room
            else {}

            rooms.push(new_room);
        }
    }

    (rooms, map)
}

pub fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = xy_idx(x, y);
            map[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_corridor(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80 * 50 {
            map[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_corridor(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80 * 50 {
            map[idx] = TileType::Floor;
        }
    }
}

pub fn draw_map(map: &[TileType], context: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render Tile
        match tile {
            TileType::Floor => {
                let fg_color = RGB::from_f32(0.5, 0.5, 0.5);
                let bg_color = RGB::from_f32(0.0, 0.0, 0.0);
                let glyph = rltk::to_cp437('.');
                context.set(x, y, fg_color, bg_color, glyph);
            }
            TileType::Wall => {
                let fg_color = RGB::from_f32(0.0, 1.0, 0.0);
                let bg_color = RGB::from_f32(0.0, 0.0, 0.0);
                let glyph = rltk::to_cp437('#');
                context.set(x, y, fg_color, bg_color, glyph);
            }
        }

        // Move coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}