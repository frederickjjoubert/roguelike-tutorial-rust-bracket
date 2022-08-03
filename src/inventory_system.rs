use specs::prelude::*;
use super::{
    CombatStats,
    Consumable,
    GameLog,
    InBackpack,
    Name,
    Position,
    ProvidesHealing,
    WantsToDropItem,
    WantsToPickupItem,
    WantsToUseItem,
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToPickupItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            names,
            mut in_backpacks,
            mut positions,
            mut wants_to_pickup_items
        ) = data;

        for wants_to_pickup_item in wants_to_pickup_items.join() {
            // Remove the Item from the World by remove it's Position component.
            positions.remove(wants_to_pickup_item.item);

            // Create InBackpack component
            in_backpacks.insert(
                wants_to_pickup_item.item,
                InBackpack {
                    owner: wants_to_pickup_item.collected_by
                },
            ).expect("Unable to insert backpack entry");

            // If the player picked up the item, log it.
            if wants_to_pickup_item.collected_by == *player_entity {
                let item_name = &names.get(wants_to_pickup_item.item).unwrap().name;
                game_log.entries.push(format!("You pick up the {}", item_name));
            }
        }

        // Clear all WantsToPickupItems for next tick.
        wants_to_pickup_items.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, WantsToUseItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut game_log,
            consumables,
            names,
            provides_healing_components,
            mut combat_stats_components,
            mut wants_to_use_item_components
        ) = data;

        for (entity, wants_to_use_item_component, combat_stats_component)
        in (&entities, &wants_to_use_item_components, &mut combat_stats_components).join()
        {
            // let mut item_used = true;
            let item_entity = wants_to_use_item_component.item;

            // Healing Items
            let healing_item = provides_healing_components.get(item_entity);
            match healing_item {
                None => {}
                Some(healing_item) => {
                    // Restore HP
                    combat_stats_component.hp = i32::min(
                        combat_stats_component.max_hp,
                        combat_stats_component.hp + healing_item.heal_amount,
                    );
                    // If player, log the interaction.
                    if entity == *player_entity {
                        let potion_name = &names.get(wants_to_use_item_component.item).unwrap().name;
                        game_log.entries.push(format!("You drink the {}, healing {} hp.", potion_name, healing_item.heal_amount));
                    }
                }
            }

            // Delete the Item if it is Consumable
            let consumable = consumables.get(item_entity);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(item_entity).expect("Delete Entity Failed.")
                    // ^ Note: Since all of the placement information is attached to the potion itself,
                    // there's no need to chase around making sure it is removed from the appropriate backpack:
                    // the entity ceases to exist, and takes its components with it.
                }
            }
        }

        // Clear all WantsToDrinkPotion components for next tick.
        wants_to_use_item_components.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToDropItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
            player_entity,
            mut game_log,
            names,
            mut in_backpacks,
            mut positions,
            mut wants_to_drop_items) = data;

        for (entity, wants_to_drop_item) in (&entities, &wants_to_drop_items).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(
                wants_to_drop_item.item,
                Position { x: dropper_pos.x, y: dropper_pos.y },
            ).expect("Unable to insert position");


            in_backpacks.remove(wants_to_drop_item.item);

            if entity == *player_entity {
                let item_name = &names.get(wants_to_drop_item.item).unwrap().name;
                game_log.entries.push(format!("You drop the {}.", item_name));
            }
        }

        // Clear all WantsToDropItem components for next tick.
        wants_to_drop_items.clear();
    }
}