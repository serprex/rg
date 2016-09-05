use std::mem;
use smallvec::SmallVec;
use specs::{UnprotectedStorage, Index};

pub struct SuperSparseStorage<T>(SmallVec<[(Index, T); 4]>);

impl<T> UnprotectedStorage<T> for SuperSparseStorage<T> {
	fn new() -> Self {
		SuperSparseStorage(SmallVec::new())
	}
	unsafe fn clean<F>(&mut self, _has: F) where F: Fn(Index) -> bool {
	}
	unsafe fn get(&self, id: Index) -> &T {
		let SuperSparseStorage(ref vec) = *self;
		for &(idx, ref v) in vec.iter() {
			if idx == id { return v }
		}
		unreachable!()
	}
	unsafe fn get_mut(&mut self, id: Index) -> &mut T {
		let SuperSparseStorage(ref mut vec) = *self;
		for &mut (idx, ref mut v) in vec.iter_mut() {
			if idx == id { return v }
		}
		unreachable!()
	}
	unsafe fn insert(&mut self, id: Index, val: T) {
		let SuperSparseStorage(ref mut vec) = *self;
		for &mut (idx, ref mut v) in vec.iter_mut() {
			if idx == id { *v = val; return }
		}
		vec.push((id, val))
	}
	unsafe fn remove(&mut self, id: Index) -> T {
		let SuperSparseStorage(ref mut vec) = *self;
		let mut vidx = mem::uninitialized();
		for (i, &(idx, _)) in vec.iter().enumerate() {
			if idx == id { vidx = i; break }
		}
		vec.remove(vidx).1
	}
}