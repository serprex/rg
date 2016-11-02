use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::hash_map::Entry;
use fnv::FnvHashMap;

use actions::Action;

pub struct Ticker {
	pub tick: u32,
	tracking: FnvHashMap<u32, u32>,
	events: BinaryHeap<TickAction>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tick {
	pub tick: u32,
	pub nth: u32,
}

struct TickAction {
	pub tick: Tick,
	pub act: Action,
}

impl PartialEq for TickAction {
	fn eq(&self, other: &TickAction) -> bool {
		self.tick == other.tick
	}
}

impl Eq for TickAction {}

impl PartialOrd for TickAction {
	fn partial_cmp(&self, other: &TickAction) -> Option<Ordering> {
		other.tick.partial_cmp(&self.tick)
	}
}

impl Ord for TickAction {
	fn cmp(&self, other: &TickAction) -> Ordering {
		other.tick.cmp(&self.tick)
	}
}

impl Default for Ticker {
	fn default() -> Self {
		Ticker {
			tick: 1,
			tracking: Default::default(),
			events: Default::default(),
		}
	}
}

impl Ticker {
	pub fn tick(&mut self) {
		self.tracking.remove(&self.tick);
		self.tick += 1;
	}
	fn tock(&mut self, n: u32) -> Tick {
		Tick {
			tick: self.tick + n,
			nth: match self.tracking.entry(self.tick) {
				Entry::Vacant(entry) => {
					entry.insert(0);
					0
				},
				Entry::Occupied(mut entry) => {
					let mut entry = entry.get_mut();
					*entry += 1;
					*entry
				},
			}
		}
	}
	pub fn push(&mut self, n: u32, act: Action) {
		let tock = self.tock(n);
		self.events.push(TickAction { tick: tock, act: act });
	}
	pub fn pop(&mut self) -> Vec<Action> {
		let mut rmk = Vec::new();
		loop {
			if let Some(ta) = self.events.peek() {
				if ta.tick.tick != self.tick {
					break
				}
			} else {
				break
			};
			if let Some(ta) = self.events.pop() {
				rmk.push(ta.act);
			}
		}
		rmk
	}
}

