use std::cmp;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use std::time::Duration;
use std::io::{self, Read};
use fnv::FnvHasher;

pub type FnvHashSet<T> = HashSet<T, BuildHasherDefault<FnvHasher>>;
pub type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

pub fn rectover(r1: [i16; 4], r2: [i16; 4]) -> bool {
	r1[0] <= r2[2] && r1[2] >= r2[0] && r1[1] <= r2[3] && r1[3] >= r2[1]
}

pub fn rectoverinc(r1: [i16; 4], r2: [i16; 4]) -> bool {
	r1[0] < r2[2] && r1[2] > r2[0] && r1[1] < r2[3] && r1[3] > r2[1]
}

pub static EXITGAME: AtomicBool = ATOMIC_BOOL_INIT;

pub fn dur_as_f64(dur: Duration) -> f64 {
	dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1e9
}

pub fn cmpi<T, U>(a: T, b: T, lt: U, eq: U, gt: U) -> U
	where T: cmp::Ord
{
	match a.cmp(&b) {
		cmp::Ordering::Less => lt,
		cmp::Ordering::Equal => eq,
		cmp::Ordering::Greater => gt,
	}
}

pub fn getch() -> char {
	let stdin = io::stdin();
	let sin = stdin.lock();
	let mut sinchars = sin.bytes();
	let ch = sinchars.next().map(|next| next.unwrap_or(0x1b) as char).unwrap_or('\x1b');
	if ch == '\x1b' { EXITGAME.store(true, Ordering::Relaxed) }
	ch
}
