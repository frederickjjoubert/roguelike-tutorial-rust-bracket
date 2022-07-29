use specs::prelude::*;
use super::{CombatStats, SufferDamage};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut combat_stats, mut suffer_damages) = data;
        for (mut combat_stat, suffer_damage) in (&mut combat_stats, &suffer_damages).join() {
            combat_stat.hp -= suffer_damage.amount.iter().sum::<i32>();
        }
        // Clear the ECS Storage of SufferDamage of all SufferDamage components to prepare for the next tick.
        suffer_damages.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut the_dead: Vec<Entity> = Vec::new();

    // Nested Scope to make the Borrow Checker happy.
    // Otherwise it complains about line 39 since we do an immutable borrow on line 29.
    {
        let entities = ecs.entities(); // Immutable Borrow of ecs.
        let combat_stats = ecs.read_storage::<CombatStats>();
        for (entity, combat_stat) in (&entities, &combat_stats).join() {
            if combat_stat.hp <= 0 {
                the_dead.push(entity);
            }
        }
    }

    for victim in the_dead {
        ecs.delete_entity(victim).expect("Unable to delete Entity"); // Mutating ecs.
    }
}