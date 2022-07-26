use rltk::{Rltk, GameState, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{min, max};
use specs_derive::Component;

// An empty Component is called a Tag Component
#[derive(Component)]
struct LeftMover {}

// A System
struct LeftWalker {}

// impl<'a> System<'a> for LeftWalker means we are implementing Specs' System trait
// for our LeftWalker structure. The 'a are lifetime specifiers: the system is saying
// that the components it uses must exist long enough for the system to run.
impl<'a> System<'a> for LeftWalker {
    // This system needs Read Access from LeftMover, and Write Access to Position
    type SystemData = (
        ReadStorage<'a, LeftMover>,
        WriteStorage<'a, Position>
    );

    // fn run is the actual trait implementation, required by the impl System.
    // It takes itself, and the SystemData we defined.
    fn run(&mut self, (lefty, mut position): Self::SystemData) {
        for (_, position) in (&lefty, &mut position).join() {
            position.x -= 1;
            if position.x < 0 {
                position.x = 79;
            }
        }
    }
}

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut left_walker = LeftWalker {}; // Create a new LeftWalker system.
        left_walker.run_now(&self.ecs); // Run the System.
        self.ecs.maintain(); // Tells Specs to apply any changes that are queued up.
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls(); // CLS: Clear the Screen.

        // Here the ECS is calling out our functions
        self.run_systems(); // Within run_systems(...)

        // Here we're calling into the ECS to perform the Rendering
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (position, renderable) in (&positions, &renderables).join() {
            context.set(position.x, position.y, renderable.fg, renderable.bg, renderable.glyph)
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;
    let mut game_state = State {
        ecs: World::new()
    };
    // Register Components with ECS.
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<LeftMover>();
    // Create Entities
    game_state.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .build();

    for i in 0..10 {
        game_state.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }
    // Run the main game loop.
    rltk::main_loop(context, game_state)
}
