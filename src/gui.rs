use rltk::{RGB, Rltk, Point, VirtualKeyCode};
use specs::prelude::*;
use super::{
    CombatStats,
    GameLog,
    InBackpack,
    Map,
    Name,
    Player,
    Position,
    RunState,
    State,
    Viewshed,
};


#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MainMenuResult {
    NoSelection {
        selected: MainMenuSelection
    },
    Selected {
        selected: MainMenuSelection
    },
}

pub fn draw_ui(ecs: &World, context: &mut Rltk) {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for (_player, combat_stat) in (&players, &combat_stats).join() {
        // Health Text with Color
        let health = format!("{} / {}", combat_stat.hp, combat_stat.max_hp);
        context.print_color(
            12, 43,
            RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK),
            &health);
        // Health Bar
        context.draw_bar_horizontal(
            28, 43, 51,
            combat_stat.hp, combat_stat.max_hp,
            RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    // Game Log
    let game_log = ecs.fetch::<GameLog>();
    let mut y_pos = 44;
    for entry in game_log.entries.iter().rev() {
        if y_pos < 49 {
            context.print(2, y_pos, entry);
        }
        y_pos += 1;
    }

    // Mouse Cursor
    let mouse_position = context.mouse_pos();
    context.set_bg(mouse_position.0, mouse_position.1, RGB::named(rltk::MAGENTA));
    // Draw Tooltips
    draw_tooltips(ecs, context);
}

fn draw_tooltips(ecs: &World, context: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let (mouse_x, mouse_y) = context.mouse_pos();
    if mouse_x >= map.width || mouse_y >= map.height { return; }
    let mut tooltips: Vec<String> = Vec::new();

    // Gather all Entities with Names and Positions for the tooltips.
    for (name, position) in (&names, &positions).join() {
        let index = map.xy_idx(position.x, position.y);
        if position.x == mouse_x && position.y == mouse_y && map.visible_tiles[index] {
            tooltips.push(name.name.to_string());
        }
    }

    if !tooltips.is_empty() {
        let mut width: i32 = 0;
        for tooltip in tooltips.iter() {
            // Set the Width of the tooltip to be as long as the longest string.
            if width < tooltip.len() as i32 { width = tooltip.len() as i32; }
        }
        width += 3; // Add some padding

        // Right side of Screen
        if mouse_x > 40 {
            let arrow_position = Point::new(mouse_x - 2, mouse_y);
            let left_x = mouse_x - width;
            let mut y = mouse_y;
            for tooltip in tooltips.iter() {
                context.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), tooltip);
                let padding = (width - tooltip.len() as i32) - 1;
                for i in 0..padding {
                    context.print_color(arrow_position.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), &" ".to_string());
                }
                y += 1;
            }
            context.print_color(arrow_position.x, arrow_position.y, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), &"->".to_string());
        }
        // Left side of Screen
        else {
            let arrow_position = Point::new(mouse_x + 1, mouse_y);
            let left_x = mouse_x + 3;
            let mut y = mouse_y;
            for tooltip in tooltips.iter() {
                context.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), tooltip);
                let padding = (width - tooltip.len() as i32) - 1;
                for i in 0..padding {
                    context.print_color(arrow_position.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), &" ".to_string());
                }
                y += 1;
            }
            context.print_color(arrow_position.x, arrow_position.y, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), &"<-".to_string());
        }
    }
}


pub fn show_inventory(game_state: &mut State, context: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let entities = game_state.ecs.entities();
    let player_entity = game_state.ecs.fetch::<Entity>();
    let names = game_state.ecs.read_storage::<Name>();
    let in_backpacks = game_state.ecs.read_storage::<InBackpack>();

    // Get player inventory
    let player_inventory = (&names, &in_backpacks)
        .join()
        .filter(|item| item.1.owner == *player_entity);
    let num_items = player_inventory.count();

    // Draw UI
    let mut y = (25 - (num_items / 2)) as i32;
    context.draw_box(
        15, y - 2,
        31, (num_items + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    context.print_color(
        18, y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    context.print_color(
        18, y + num_items as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESC to Cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _in_backpack, name) in (&entities, &in_backpacks, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity) {
        // List out all the Items
        context.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        context.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
        context.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        context.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match context.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    // Player has made a valid selection, return the item.
                    if selection > -1 && selection < num_items as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    // Invalid selection, ignore it.
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn drop_item_menu(gs: &mut State, context: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let entities = gs.ecs.entities();
    let player_entity = gs.ecs.fetch::<Entity>();
    let in_backpacks = gs.ecs.read_storage::<InBackpack>();
    let names = gs.ecs.read_storage::<Name>();

    let inventory = (&in_backpacks, &names).join().filter(|item| item.0.owner == *player_entity);
    let num_items = inventory.count();

    let mut y = (25 - (num_items / 2)) as i32;
    context.draw_box(
        15, y - 2,
        31, (num_items + 3) as i32,
        RGB::named(rltk::WHITE), RGB::named(rltk::BLACK),
    );
    context.print_color(
        18, y - 2,
        RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK),
        "Drop Which Item?",
    );
    context.print_color(
        18, y + num_items as i32 + 1,
        RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _in_backpack, name)
    in (&entities, &in_backpacks, &names).join().filter(|item| item.1.owner == *player_entity)
    {
        context.set(
            17, y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        context.set(
            18, y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        context.set(
            19, y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        context.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match context.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < num_items as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn ranged_target(game_state: &mut State, context: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = game_state.ecs.fetch::<Entity>();
    let player_position = game_state.ecs.fetch::<Point>();
    let viewsheds = game_state.ecs.read_storage::<Viewshed>();

    context.print_color(5, 0, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Select Target:");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let optional_viewshed = viewsheds.get(*player_entity);
    if let Some(player_viewshed) = optional_viewshed {
        // We have a viewshed
        for position in player_viewshed.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_position, *position);
            if distance <= range as f32 {
                context.set_bg(position.x, position.y, RGB::named(rltk::BLUE));
                available_cells.push(position);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_position = context.mouse_pos();
    let (mouse_x, mouse_y) = mouse_position;
    let mut is_valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_x && idx.y == mouse_y
        {
            is_valid_target = true;
        }
    }
    if is_valid_target {
        context.set_bg(mouse_x, mouse_y, RGB::named(rltk::CYAN));
        if context.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_x, mouse_y)));
        }
    } else {
        context.set_bg(mouse_x, mouse_y, RGB::named(rltk::RED));
        if context.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(game_state: &mut State, context: &mut Rltk) -> MainMenuResult {
    let save_exists = super::save_load_system::does_save_exist();
    let current_run_state = game_state.ecs.fetch::<RunState>();

    context.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Rust Roguelike Tutorial");

    if let RunState::MainMenu { menu_selection: current_main_menu_selection } = *current_run_state {
        if current_main_menu_selection == MainMenuSelection::NewGame {
            context.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game");
        } else {
            context.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        if save_exists {
            if current_main_menu_selection == MainMenuSelection::LoadGame {
                context.print_color_centered(25, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Load Game");
            } else {
                context.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Load Game");
            }
        }

        if current_main_menu_selection == MainMenuSelection::Quit {
            context.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            context.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match context.key {
            None => return MainMenuResult::NoSelection { selected: current_main_menu_selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => {
                        return MainMenuResult::NoSelection {
                            selected: MainMenuSelection::Quit
                        };
                    }
                    VirtualKeyCode::Up => {
                        let new_main_menu_selection;
                        match current_main_menu_selection {
                            MainMenuSelection::NewGame => new_main_menu_selection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => new_main_menu_selection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => new_main_menu_selection = MainMenuSelection::LoadGame
                        }
                        return MainMenuResult::NoSelection { selected: new_main_menu_selection };
                    }
                    VirtualKeyCode::Down => {
                        let new_main_menu_selection;
                        match current_main_menu_selection {
                            MainMenuSelection::NewGame => new_main_menu_selection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => new_main_menu_selection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => new_main_menu_selection = MainMenuSelection::NewGame
                        }
                        return MainMenuResult::NoSelection { selected: new_main_menu_selection };
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected { selected: current_main_menu_selection },
                    _ => return MainMenuResult::NoSelection { selected: current_main_menu_selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}