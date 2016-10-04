use specs::{Entity, Join, World};
use smallvec::SmallVec;

use components::*;
use position::Possy;
pub fn moveloop(w: &mut World) {
	{
	let (npos, mut mort, portal, mut ai, mut solid, ents) =
		(w.read::<NPos>(), w.write::<Mortal>(), w.read::<Portal>(), w.write::<Ai>(), w.write::<Solid>(), w.entities());
	let Walls(ref walls) = *w.read_resource::<Walls>();
	let mut possy = w.write_resource::<Possy>();

	'newposloop:
	for (&NPos(n), ent) in (&npos, &ents).iter() {
		if walls.contains_key(&n) {
			continue 'newposloop
		}
		for &e in possy.npos_map(&npos, &ents).get_ents(n).into_iter() {
			if e != ent && solid.get(e).is_some() {
				continue 'newposloop
			}
		}
		possy.set_pos(ent, n);
	}

	let mut rmai = SmallVec::<[Entity; 2]>::new();
	let mut spos = SmallVec::<[(Entity, [i16; 3]); 2]>::new();
	for (_xyz, col) in possy.npos_map(&npos, &ents).collisions() {
		for e in col.iter().cloned() {
			for ce in col.iter().cloned().filter(|&ce| ce != e) {
				if let Some(&Portal(porx)) = portal.get(e) {
					spos.push((ce, porx));
				}
				if let Some(aie) = ai.get(e) {
					match aie.state {
						AiState::Missile(_, dmg) => {
							if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
								if *mce <= dmg {
									*mce = 0;
									solid.remove(ce);
									rmai.push(ce);
								} else {
									*mce -= dmg;
								}
							}
							if let Some(_) = solid.get(ce) {
								w.delete_later(e)
							}
						},
						AiState::Melee(_, dmg) => {
							if let Some(&mut Mortal(ref mut mce)) = mort.get_mut(ce) {
								if *mce <= dmg {
									*mce = 0;
									solid.remove(ce);
									rmai.push(ce);
								} else {
									*mce -= dmg;
								}
							}
						},
						_ => (),
					}
				}
			}
		}
	}
	for e in rmai {
		ai.remove(e);
	}
	for (e, p) in spos {
		possy.set_pos(e, p);
	}
	}
	w.write::<NPos>().clear();
}
