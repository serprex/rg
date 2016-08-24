use std::mem;
use rand::{self, Rng};
use specs::*;
use super::components::*;
use super::util::*;

pub fn attack(src: Entity, w: &World) {
	let weapons = w.read::<Weapon>();
	let cch = w.read::<Chr>();
	let watk = w.read::<Atk<Weapon>>();
	/*
	if let Some(&Weapon(went)) = weapons.get(ent) {
		if let Some(&wch) = cch.get(went) {
			if let Some(&wdir) =  wdirection.get(ent) {
				let ach = getch();
				let wstats = *watk.get(went).unwrap_or(&Atk::<Weapon>::new(0, 0, 0));
				newent.push((w.create_later(), wch, Ai::new(AiState::Melee(wstats.dur, wstats.dmg), 1), Pos(bp)));
				ai.tick = if wstats.spd < 0 {
					let spd = (-wstats.spd) as u8;
					if spd < ai.tick { ai.tick - spd } else { 0 }
				} else {
					ai.tick + wstats.spd as u8
				};
			}
		}
	}*/
}

pub fn heal(src: Entity, w: &World) {

}
