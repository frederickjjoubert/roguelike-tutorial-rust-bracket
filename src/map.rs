use rltk::{RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point, SmallVec};
use super::{Rect};
use std::cmp::{max, min};
use specs::prelude::*;
use serde::{Serialize, Deserialize};

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 43;
pub const MAP_COUNT: usize = MAP_WIDTH * MAP_HEIGHT;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Downstairs,
    Wall,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_contents: Vec<Vec<Entity>>,
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

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal Directions
        if self.is_exit_valid(x - 1, y) { exits.push((idx - 1, 1.0)); }
        if self.is_exit_valid(x + 1, y) { exits.push((idx + 1, 1.0)); }
        if self.is_exit_valid(x, y - 1) { exits.push((idx - w, 1.0)); }
        if self.is_exit_valid(x, y + 1) { exits.push((idx + w, 1.0)); }

        // Diagonal Directions
        if self.is_exit_valid(x - 1, y - 1) { exits.push(((idx - w) - 1, 1.45)); }
        if self.is_exit_valid(x + 1, y - 1) { exits.push(((idx - w) + 1, 1.45)); }
        if self.is_exit_valid(x - 1, y + 1) { exits.push(((idx + w) - 1, 1.45)); }
        if self.is_exit_valid(x + 1, y + 1) { exits.push(((idx + w) + 1, 1.45)); }
        // ^ Note to self on the above, I believe the tuple above is (index, distance)

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let point_1 = Point::new(idx1 % w, idx1 / w);
        let point_2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(point_1, point_2)
    }
}

impl Map {
    /// This is simple: it multiplies the y position by the map width (80), and adds x.
    /// This guarantees one tile per location, and efficiently maps it in memory for left-to-right reading.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    // Helper for "get_available_exits(...)"
    pub fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 { return false; };
        let index = self.xy_idx(x, y);
        !self.blocked_tiles[index]
    }

    pub fn calculate_blocked_tiles(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked_tiles[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_all_tiles_contents(&mut self) {
        for content in self.tile_contents.iter_mut() {
            content.clear();
        }
    }

    pub fn new_map_rooms_and_corridors(new_depth: i32) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAP_COUNT],
            rooms: Vec::new(),
            width: MAP_WIDTH as i32,
            height: MAP_HEIGHT as i32,
            depth: new_depth,
            revealed_tiles: vec![false; MAP_COUNT],
            visible_tiles: vec![false; MAP_COUNT],
            blocked_tiles: vec![false; MAP_COUNT],
            tile_contents: vec![Vec::new(); MAP_COUNT],
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, MAP_WIDTH as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, MAP_HEIGHT as i32 - h - 1) - 1;
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

        let (stair_x, stair_y) = map.rooms[map.rooms.len() - 1].center();
        let stair_index = map.xy_idx(stair_x, stair_y);
        map.tiles[stair_index] = TileType::Downstairs;

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
            if idx > 0 && idx < MAP_WIDTH * MAP_HEIGHT {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_vertical_corridor(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAP_WIDTH * MAP_HEIGHT {
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
                TileType::Downstairs => {
                    fg_color = RGB::from_f32(0.0, 1.0, 1.0);
                    bg_color = RGB::from_f32(0.0, 0.0, 0.0);
                    glyph = rltk::to_cp437('>');
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
