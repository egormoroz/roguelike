use specs::prelude::*;
use crate::{
    util::GameLog,
    state::RunState,
    comp::*,
};

pub struct HungerSystem;

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, HungerClock>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, state, mut log, players, 
            mut hunger_clocks, mut suffer_damage) = data;

        for (e, hc, player) 
            in (&entities, &mut hunger_clocks, players.maybe()).join() 
        {
            match *state {
                RunState::PlayerTurn if player.is_some() => (),
                RunState::MonsterTurn if player.is_none() => (),
                _ => continue,
            };
            hc.duration -= 1;
            if hc.duration > 0 { continue; }

            use HungerState::*;
            let old_state = hc.state;
            let new_state = match old_state {
                WellFed => Normal,
                Normal => Hungry,
                Hungry => Starving,
                Starving => {
                    SufferDamage::new_damage(&mut suffer_damage, e, 1);
                    Starving
                }
            };
            hc.duration = 200;
            hc.state = new_state;

            if player.is_some() {
                use std::io::Write;
                write!(log.new_entry(), "{}", match new_state {
                    Normal => "You are no longer well fed.",
                    Hungry => "You are hungry.",
                    Starving if old_state == Starving => "Your hunger pangs are getting painful! You suffer 1 hp damage.",
                    Starving => "You are starving.",
                    _ => unreachable!(),
                }).unwrap();
            }
        }
    }
}
