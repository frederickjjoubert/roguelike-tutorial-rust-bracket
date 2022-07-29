use rltk::console;
use specs::prelude::*;
use crate::{CombatStats, Name, SufferDamage, WantsToMelee};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CombatStats>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, combat_stats, names, mut suffer_damages, mut wants_to_melees) = data;
        // Iterate through all the Entities that WantToMelee
        for (_entity, combat_stat, name, wants_to_melee) in (&entities, &combat_stats, &names, &mut wants_to_melees).join() {
            if combat_stat.hp > 0 {
                let target_stat = combat_stats.get(wants_to_melee.target).unwrap();
                if target_stat.hp > 0 {
                    let target_name = names.get(wants_to_melee.target).unwrap();
                    let damage = combat_stat.power - target_stat.defense;

                    if damage == 0 {
                        console::log(&format!("{} is unable to hurt {}.", &name.name, &target_name.name));
                    } else {
                        console::log(&format!("{} hits {} for {} damage!", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut suffer_damages, wants_to_melee.target, damage);
                    }
                }
            }
        }
        // Clear the ECS Storage of WantsToMelee of all WantsToMelee components to prepare for the next tick.
        wants_to_melees.clear();
    }
}