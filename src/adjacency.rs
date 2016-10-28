use std::collections::hash_map::Entry;
use fnv::{FnvHashSet, FnvHashMap};

pub trait Adjacency {
	fn insert(&mut self, a: usize, b: usize) -> bool;
	fn remove(&mut self, a: usize, b: usize) -> bool;
	fn contains(&self, a: usize, b: usize) -> bool;
	fn arms(&self, a: usize, b: usize) -> Vec<(usize, usize)>;
	fn len(&self) -> usize;
	fn root(&self) -> Option<(usize, usize)>;
}

#[derive(Default, Clone)]
pub struct AdjacencySet(FnvHashSet<(usize, usize)>);

#[derive(Clone)]
pub struct AdjacencyVec(Vec<bool>, usize);

#[derive(Default, Clone)]
pub struct AdjacencyMap(FnvHashMap<usize, FnvHashSet<usize>>);

impl Adjacency for AdjacencySet {
	#[inline(always)]
	fn insert(&mut self, a: usize, b: usize) -> bool {
		let ab = if a > b { (a, b) } else { (b, a) };
		self.0.insert(ab)
	}

	#[inline(always)]
	fn remove(&mut self, a: usize, b: usize) -> bool {
		let ab = if a > b { (a, b) } else { (b, a) };
		self.0.remove(&ab)
	}

	#[inline(always)]
	fn contains(&self, a: usize, b: usize) -> bool {
		let ab = if a > b { (a, b) } else { (b, a) };
		self.0.contains(&ab)
	}

	fn arms(&self, a: usize, b: usize) -> Vec<(usize, usize)> {
		let (a, b) = if a > b { (a, b) } else { (b, a) };
		self.0.iter().cloned().filter(|&(x, y)| (x == a && y != b) || (x != a && y == b)).collect()
	}

	#[inline(always)]
	fn len(&self) -> usize {
		self.0.len()
	}

	#[inline(always)]
	fn root(&self) -> Option<(usize, usize)> {
		self.0.iter().cloned().next()
	}
}

pub fn is_connected<T: Adjacency>(adj: &T) -> bool {
	if let Some((a, b)) = adj.root() {
		let mut seen = Default::default();
		is_connected_core(adj, a, b, &mut seen);
		seen.len() == adj.len()
	} else {
		true
	}
}

fn is_connected_core<T: Adjacency>(adj: &T, a: usize, b: usize, seen: &mut FnvHashSet<(usize, usize)>) {
	seen.insert((a, b));
	let search: Vec<(usize, usize)> = adj.arms(a, b).into_iter().filter(|x| !seen.contains(x)).collect();
	for (x, y) in search {
		is_connected_core(adj, x, y, seen);
	}
}

impl Adjacency for AdjacencyVec {
	fn insert(&mut self, a: usize, b: usize) -> bool {
		let ret = !self.contains(a, b);
		self.0[a + b * self.1] = true;
		self.0[b + a * self.1] = true;
		ret
	}
	fn remove(&mut self, a: usize, b: usize) -> bool {
		let ret = self.contains(a, b);
		self.0[a + b * self.1] = false;
		self.0[b + a * self.1] = false;
		ret
	}
	#[inline(always)]
	fn contains(&self, a: usize, b: usize) -> bool {
		self.0[a + b * self.1]
	}
	fn arms(&self, a: usize, b: usize) -> Vec<(usize, usize)> {
		let mut ret = Vec::new();
		for x in 0..self.1 {
			let (x, y) = if x > a { (x, b) } else { (a, x) };
			if self.contains(x, y) { ret.push((x, y)) }
		}
		ret
	}
	fn len(&self) -> usize {
		let mut ret = 0;
		for x in 0..self.1 {
			for y in 0..(x+1) {
				if self.contains(x, y) {
					ret += 1;
				}
			}
		}
		ret
	}
	fn root(&self) -> Option<(usize, usize)> {
		for x in 0..self.1 {
			for y in 0..(x+1) {
				if self.contains(x, y) {
					return Some((x, y))
				}
			}
		}
		None
	}
}

impl AdjacencyVec {
	#[inline(always)]
	pub fn new(size: usize) -> AdjacencyVec {
		AdjacencyVec(vec![false; size * size], size)
	}
}

impl Adjacency for AdjacencyMap {
	fn insert(&mut self, a: usize, b: usize) -> bool {
		match self.0.entry(a) {
			Entry::Vacant(e) => {
				let mut s = FnvHashSet::default();
				s.insert(b);
				e.insert(s);
			},
			Entry::Occupied(mut e) => {
				e.get_mut().insert(b);
			},
		}
		match self.0.entry(b) {
			Entry::Vacant(e) => {
				let mut s = FnvHashSet::default();
				s.insert(a);
				e.insert(s);
				true
			},
			Entry::Occupied(mut e) => {
				e.get_mut().insert(b)
			},
		}
	}
	fn remove(&mut self, a: usize, b: usize) -> bool {
		if let Some(s) = self.0.get_mut(&a) {
			s.remove(&b);
		}
		if let Some(s) = self.0.get_mut(&b) {
			s.remove(&a)
		} else {
			false
		}
	}
	fn contains(&self, a: usize, b: usize) -> bool {
		if let Some(s) = self.0.get(&a) {
			s.contains(&b)
		} else {
			false
		}
	}
	fn arms(&self, a: usize, b: usize) -> Vec<(usize, usize)> {
		let mut s: FnvHashSet<(usize, usize)> = if let Some(s) = self.0.get(&a) {
			s.iter().map(|&b| (a, b)).filter(|&(a, b)| a <= b).collect()
		} else {
			FnvHashSet::default()
		};
		if let Some(s2) = self.0.get(&b) {
			for b in s2.iter().cloned().filter(|&b| a <= b) {
				s.insert((a, b));
			}
		}
		s.into_iter().collect()
	}
	fn len(&self) -> usize {
		let mut ret = 0;
		for (&a, v) in self.0.iter() {
			for &b in v.iter() {
				if a <= b {
					ret += 1;
				}
			}
		}
		ret
	}
	fn root(&self) -> Option<(usize, usize)> {
		for (&a, v) in self.0.iter() {
			for &b in v.iter() {
				return Some((a, b))
			}
		}
		None
	}
}
