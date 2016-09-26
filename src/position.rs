use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::ops::Deref;

use smallvec::SmallVec;
use specs::{World, Entities, Entity, Join, Storage, Allocator, MaskedStorage};

use components::{Pos, NPos};
use util::{FnvHashMap, FnvHashSet};

#[derive(Default)]
pub struct Possy {
	pub floors: FnvHashMap<i16, FnvHashMap<[i16; 2], SmallVec<[Entity; 1]>>>,
	pub e2p: FnvHashMap<Entity, [i16; 3]>,
	pub collisions: FnvHashSet<[i16; 3]>,
}

pub struct PossyNpos<'a, 'b, 'c, A, D>
	where A: 'b + Deref<Target = Allocator>,
		D: 'b + Deref<Target = MaskedStorage<NPos>>
{
	possy: &'a Possy,
	npos: &'b Storage<NPos, A, D>,
	ents: &'c Entities<'c>
}
impl<'a, 'b, 'c, A, D> PossyNpos<'a, 'b, 'c, A, D>
	where A: Deref<Target = Allocator>,
		D: Deref<Target = MaskedStorage<NPos>>
{
	pub fn new(possy: &'a Possy, npos: &'b Storage<NPos, A, D>, ents: &'c Entities<'c>) -> Self {
		PossyNpos {
			possy: possy,
			npos: npos,
			ents: ents,
		}
	}
	pub fn get_pos(&self, e: Entity) -> Option<[i16; 3]> {
		if let Some(&NPos(p)) = self.npos.get(e) {
			Some(p)
		} else {
			self.possy.get_pos(e)
		}
	}
	pub fn get_ents(&'a self, pos: [i16; 3]) -> Cow<[Entity]> {
		let mut sv = self.possy.get_ents(pos).map(|x| Cow::Borrowed(&x[..])).unwrap_or_else(|| Cow::Owned(Vec::new()));
		let len0 = sv.len();
		let mut rme = Vec::new();
		for (&NPos(np), e) in (self.npos, self.ents).iter() {
			for idx in 0..len0 {
				if sv[idx] == e {
					rme.push(idx)
				}
			}
			if np == pos {
				sv.to_mut().push(e);
			}
		}
		rme.sort_by(|a, b| b.cmp(a));
		for idx in rme {
			sv.to_mut().remove(idx);
		}
		sv
	}
	pub fn collisions(&self) -> Vec<([i16; 3], Cow<[Entity]>)> {
		self.npos.iter()
			.map(|&NPos(p)| p)
			.filter(|p| !self.possy.collisions.contains(p))
			.chain(self.possy.collisions.iter().cloned())
			.map(|p| (p, self.get_ents(p)))
			.filter(|&(_, ref ents)| ents.len() > 1)
			.collect::<Vec<_>>()
	}
}

impl Possy {
	pub fn new(w: &mut World) -> Possy {
		let mut poss: Possy = Default::default();
		let mut npos = w.write::<NPos>();
		for (&NPos(p), e) in (&npos, &w.entities()).iter() {
			poss.set_pos(e, p);
		}
		npos.clear();
		poss
	}
	pub fn npos_map<'a, 'b, 'c, A, D>(&'a self, s: &'b Storage<NPos, A, D>, ents: &'c Entities<'c>) -> PossyNpos<'a, 'b, 'c, A, D>
		where A: Deref<Target = Allocator>,
			D: Deref<Target = MaskedStorage<NPos>>
	{
		PossyNpos::new(self, s, ents)
	}
	pub fn get_pos(&self, e: Entity) -> Option<[i16; 3]> {
		self.e2p.get(&e).map(|&x| x)
	}
	pub fn gc(&mut self, w: &World) {
		let pos = w.read::<Pos>();
		let mut rme = Vec::new();
		for (&k, &p) in self.e2p.iter() {
			if pos.get(k).is_none() {
				rme.push((k, p));
			}
		}
		for (k, p) in rme.into_iter() {
			self.e2p.remove(&k);
			self.floors.get(&p[2]).unwrap().get(&p[..2]);
		}
	}
	pub fn get_within(&self, xyz: [i16; 3], r: i16) -> Vec<(Entity, [i16; 2])> {
		let x = xyz[0];
		let y = xyz[1];
		let z = xyz[2];
		match self.floors.get(&z) {
			None => Vec::new(),
			Some(floor) => {
				let mut ents = Vec::new();
				for (k, v) in floor.iter() {
					if (k[0] - x).abs() < r && (k[1] - y).abs() < r {
						for &e in v.iter() {
							ents.push((e, *k))
						}
					}
				}
				ents
			}
		}
	}
	pub fn get_ents(&self, pos: [i16; 3]) -> Option<&SmallVec<[Entity; 1]>> {
		self.floors.get(&pos[2]).and_then(|floor| floor.get(&pos[..2]))
	}
	pub fn contains(&self, pos: [i16; 3]) -> bool {
		self.get_ents(pos).is_some()
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
			let mut floor = self.floors.get_mut(&oldpos[2]).unwrap();
			let eveclen = {
				let mut idx: usize = unsafe { ::std::mem::uninitialized() };
				let mut evec = floor.get_mut(&oldpos[..2]).unwrap();
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
				self.collisions.remove(&oldpos);
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
				match floor.get_mut().entry([pos[0],pos[1]]) {
					Entry::Vacant(try) => {
						let mut sv = try.insert(SmallVec::new());
						sv.push(e)
					},
					Entry::Occupied(mut try) => {
						let mut sv = try.get_mut();
						sv.push(e);
						if sv.len() > 1 {
							self.collisions.insert(pos);
						}
					},
				}
			}
		}
	}
}
