use specs::prelude::*;
use crate::{AreaOfEffect, Confusion, InflictsDamage, SufferDamage};
use super::{
    CombatStats,
    Consumable,
    GameLog,
    InBackpack,
    Map,
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
        ReadExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, AreaOfEffect>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, WantsToUseItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            map,
            mut game_log,
            area_of_effect_components,
            consumables,
            inflicts_damage_components,
            names,
            provides_healing_components,
            mut combat_stats_components,
            mut confusion_components,
            mut suffer_damage_components,
            mut wants_to_use_item_components
        ) = data;

        for (entity, wants_to_use_item_component)
        in (&entities, &wants_to_use_item_components).join()
        {
            let mut is_item_used = true;
            let item_entity = wants_to_use_item_component.item;

            // Targeting Items
            let mut targets: Vec<Entity> = Vec::new();
            match wants_to_use_item_component.target {
                None => {
                    // No target, so apply to player.
                    targets.push(*player_entity);
                }
                Some(target) => {
                    // There is a point specified. Check for AreaOfEffect component.
                    let possible_area_of_affect_component
                        = area_of_effect_components.get(item_entity);
                    match possible_area_of_affect_component {
                        None => {
                            // Single (Tile) Target
                            let index = map.xy_idx(target.x, target.y);
                            for target in map.tile_contents[index].iter() {
                                targets.push(*target);
                            }
                        }
                        Some(area_of_effect_component) => {
                            // AoE Target
                            let mut affected_tiles = rltk::field_of_view(
                                target,
                                area_of_effect_component.radius,
                                &*map,
                            );
                            affected_tiles.retain(|point|
                                point.x > 0 &&
                                    point.x < map.width - 1 &&
                                    point.y > 0 &&
                                    point.y < map.height - 1
                            );
                            for affected_tile in affected_tiles.iter() {
                                let index = map.xy_idx(affected_tile.x, affected_tile.y);
                                for mob in map.tile_contents[index].iter() {
                                    // Right now this allows you to do AoE damage to items, even though they don't have health.
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            // Healing Items
            let healing_item = provides_healing_components.get(item_entity);
            match healing_item {
                None => {}
                Some(healing_item) => {
                    is_item_used = false;
                    for target in targets.iter() {
                        // Try get CombatStats component.
                        let combat_stats_component = combat_stats_components.get_mut(*target);
                        if let Some(combat_stats_component) = combat_stats_component {
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
                            is_item_used = true;
                        }
                    }
                }
            }

            // Inflicts Damage Items
            let inflicts_damage_item = inflicts_damage_components.get(item_entity);
            match inflicts_damage_item {
                None => {}
                Some(inflicts_damage_item) => {
                    // is_item_used = false;
                    for target in targets.iter() {
                        // Add damage
                        SufferDamage::new_damage(
                            &mut suffer_damage_components,
                            *target,
                            inflicts_damage_item.damage,
                        );
                        if entity == *player_entity {
                            let target_name = names.get(*target).unwrap();
                            let item_name = names.get(item_entity).unwrap();
                            game_log.entries.push(
                                format!(
                                    "You use {} on {}, inflicting {} damage.",
                                    item_name.name,
                                    target_name.name,
                                    inflicts_damage_item.damage,
                                )
                            );
                        }
                    }
                    is_item_used = true;
                }
            }

            // Confusion Items
            let mut confused_targets = Vec::new();
            {
                let causes_confusion_item = confusion_components.get(item_entity);
                match causes_confusion_item {
                    None => {}
                    Some(causes_confusion_item) => {
                        // is_item_used = false;
                        for target in targets.iter() {
                            confused_targets.push((*target, causes_confusion_item.turns));
                            if entity == *player_entity {
                                let target_name = &names.get(*target).unwrap().name;
                                let item_name = &names.get(item_entity).unwrap().name;
                                game_log.entries.push(format!(
                                    "You use {} on {}, confusing them!",
                                    item_name,
                                    target_name
                                ));
                            }
                        }
                        is_item_used = true;
                    }
                }
            }
            for target in confused_targets.iter() {
                confusion_components.insert(
                    target.0,
                    Confusion {
                        turns: target.1
                    },
                ).expect("Unable to insert Confusion component.");
            }

            // Delete the Item if it is Consumable
            if is_item_used {
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