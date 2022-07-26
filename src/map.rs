use rltk::{RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point};
use super::{Rect};
use std::cmp::{max, min};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}

impl Map {
    /// This is simple: it multiplies the y position by the map width (80), and adds x.
    /// This guarantees one tile per location, and efficiently maps it in memory for left-to-right reading.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
        };

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
            for other_room in map.rooms.iter() {
                if new_room.intersects(other_room) {
                    rooms_intersect = true;
                    break;
                }
            }

            if !rooms_intersect {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_corridor(prev_x, new_x, prev_y);
                        map.apply_vertical_corridor(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_corridor(prev_y, new_y, prev_x);
                        map.apply_horizontal_corridor(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_horizontal_corridor(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_vertical_corridor(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }
}

pub fn draw_map(ecs: &World, context: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (index, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[index] == true {
            let mut fg_color;
            let bg_color;
            let glyph;
            // Set variables for rendering
            match tile {
                TileType::Floor => {
                    fg_color = RGB::from_f32(0.0, 0.5, 0.5);
                    bg_color = RGB::from_f32(0.0, 0.0, 0.0);
                    glyph = rltk::to_cp437('.');
                }
                TileType::Wall => {
                    fg_color = RGB::from_f32(0.0, 1.0, 0.0);
                    bg_color = RGB::from_f32(0.0, 0.0, 0.0);
                    glyph = rltk::to_cp437('#');
                }
            }
            if !map.visible_tiles[index] {
                fg_color = fg_color.to_greyscale();
            }
            // Draw Tile
            context.set(x, y, fg_color, bg_color, glyph);
        }

        // Move coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
