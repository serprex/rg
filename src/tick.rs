use std::collections::hash_map::Entry;
use fnv::FnvHashMap;

use actions::Action;

#[derive(Default)]
pub struct Ticker {
	pub tick: u32,
	events: FnvHashMap<u32, Vec<Action>>,
}

impl Ticker {
	pub fn tick(&mut self) {
		self.events.remove(&self.tick);
		self.tick += 1;
	}
	pub fn push(&mut self, n: u32, act: Action) {
		match self.events.entry(self.tick + n) {
			Entry::Vacant(entry) => {
				entry.insert(vec![act]);
			},
			Entry::Occupied(mut entry) => {
				entry.get_mut().push(act);
			},
		}
	}
	pub fn pop(&mut self) -> Vec<Action> {
		self.events.remove(&self.tick).unwrap_or_else(Vec::new)
	}
}

