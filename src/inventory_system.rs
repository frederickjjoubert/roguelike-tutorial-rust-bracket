use specs::prelude::*;
use crate::{CombatStats, WantsToDrinkPotion};
use super::{GameLog, InBackpack, Name, Position, Potion, WantsToDropItem, WantsToPickupItem};

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

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, WantsToDrinkPotion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut game_log,
            names,
            potions,
            mut combat_stats,
            mut wants_to_drink_potions
        ) = data;

        for (entity, wants_to_drink_potion, combat_stat)
        in (&entities, &wants_to_drink_potions, &mut combat_stats).join() {
            let potion = potions.get(wants_to_drink_potion.potion);
            match potion {
                None => { println!("No potion"); }
                Some(potion) => {
                    println!("Some potion selected.");
                    // Restore HP.
                    combat_stat.hp = i32::min(combat_stat.max_hp, combat_stat.hp + potion.heal_amount);
                    // If player, log the interaction.
                    if entity == *player_entity {
                        let potion_name = &names.get(wants_to_drink_potion.potion).unwrap().name;
                        game_log.entries.push(format!("You drink the {}, healing {} hp.", potion_name, potion.heal_amount));
                    }
                    // Delete the Potion Entity
                    entities.delete(wants_to_drink_potion.potion).expect("Delete Entity Failed.")
                    // ^ Note: Since all of the placement information is attached to the potion itself,
                    // there's no need to chase around making sure it is removed from the appropriate backpack:
                    // the entity ceases to exist, and takes its components with it.
                }
            }
        }

        // Clear all WantsToDrinkPotion components for next tick.
        wants_to_drink_potions.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToDropItem>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}