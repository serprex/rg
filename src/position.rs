use std::collections::hash_map::Entry;

use fnv::FnvHashMap;
use smallvec::SmallVec;
use specs::{Entity, World};

#[derive(Default)]
pub struct Possy {
	pub floors: FnvHashMap<i16, FnvHashMap<[i16; 2], SmallVec<[Entity; 1]>>>,
	pub e2p: FnvHashMap<Entity, [i16; 3]>,
}

impl Possy {
	pub fn get_pos(&self, e: Entity) -> Option<[i16; 3]> {
		self.e2p.get(&e).map(|&x| x)
	}
	pub fn gc(&mut self, w: &World) {
		let mut rme = Vec::new();
		for &k in self.e2p.keys() {
			if !w.is_alive(k) {
				rme.push(k);
			}
		}
		for k in rme.into_iter() {
			self.remove(k);
		}
	}
	pub fn remove(&mut self, ent: Entity) {
		if let Some(p) = self.e2p.remove(&ent) {
			if let Some(floor) = self.floors.get_mut(&p[2]) {
				if let Some(sv) = floor.get_mut(&p[..2]) {
					let mut idx: usize = unsafe { ::std::mem::uninitialized() };
					for (i, &ie) in sv.iter().enumerate() {
						if ent == ie {
							idx = i;
							break
						}
					}
					sv.remove(idx);
				}
			}
		}
	}
	pub fn get_within(&self, xyz: [i16; 3], r: i16) -> Vec<(Entity, [i16; 2])> {
		let x = xyz[0];
		let y = xyz[1];
		let z = xyz[2];
		if let Some(floor) = self.floors.get(&z) {
			let mut ents = Vec::new();
			for (&k, v) in floor.iter() {
				if (k[0] - x).abs() < r && (k[1] - y).abs() < r {
					for &e in v.iter() {
						ents.push((e, k))
					}
				}
			}
			ents
		} else {
			Vec::new()
		}
	}
	pub fn get_ents(&self, pos: [i16; 3]) -> &[Entity] {
		if let Some(sv) = self.floors.get(&pos[2]).and_then(|floor| floor.get(&pos[..2])) {
			&sv[..]
		} else {
			&[]
		}
	}
	pub fn contains(&self, pos: [i16; 3]) -> bool {
		!self.get_ents(pos).is_empty()
	}
	pub fn set_pos(&mut self, e: Entity, pos: [i16; 3]) {
		let oldpos = match self.e2p.entry(e) {
			Entry::Vacant(try) => {
				try.insert(pos);
				None
			},
			Entry::Occupied(mut try) => {
				Some(try.insert(pos))
			}
		};
		if let Some(oldpos) = oldpos {
			let floor = self.floors.get_mut(&oldpos[2]).unwrap();
			let eveclen = {
				let mut idx: usize = unsafe { ::std::mem::uninitialized() };
				let evec = floor.get_mut(&oldpos[..2]).unwrap();
				for (i, &ie) in evec.iter().enumerate() {
					if e == ie {
						idx = i;
						break
					}
				}
				evec.remove(idx);
				evec.len()
			};
			if eveclen < 2 {
				if eveclen == 0 {
					floor.remove(&oldpos[..2]);
				}
			}
		}
		match self.floors.entry(pos[2]) {
			Entry::Vacant(floor) => {
				let mut fmap = FnvHashMap::default();
				let mut sv = SmallVec::new();
				sv.push(e);
				fmap.insert([pos[0], pos[1]], sv);
				floor.insert(fmap);
			},
			Entry::Occupied(mut floor) => {
				floor.get_mut().entry([pos[0],pos[1]]).or_insert_with(SmallVec::new).push(e);
			}
		}
	}
}
