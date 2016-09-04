use std::cmp;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use std::time::Duration;
use std::io::{self, Read};
use fnv::FnvHasher;
use x1b;

use components::Dir;

pub type FnvHashSet<T> = HashSet<T, BuildHasherDefault<FnvHasher>>;
pub type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;
pub type Char = x1b::Char<()>;

pub fn rectover(r1: [i16; 4], r2: [i16; 4]) -> bool {
	r1[0] <= r2[2] && r1[2] >= r2[0] && r1[1] <= r2[3] && r1[3] >= r2[1]
}

pub fn rectoverinc(r1: [i16; 4], r2: [i16; 4]) -> bool {
	r1[0] < r2[2] && r1[2] > r2[0] && r1[1] < r2[3] && r1[3] > r2[1]
}

pub static EXITGAME: AtomicBool = ATOMIC_BOOL_INIT;

pub fn dur_as_string(dur: Duration) -> String {
	let mut st = {
		if dur.as_secs() == 0 {
			let mut st = String::with_capacity(5);
			st.push_str("0.");
			st
		} else {
			let mut st = dur.as_secs().to_string();
			st.reserve(4);
			st.push('.');
			st
		}
	};
	let dc = (dur.subsec_nanos() / 1000000).to_string();
	for _ in dc.len()..3 {
		st.push('0');
	}
	st.push_str(&dc);
	st
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
	if !EXITGAME.load(Ordering::Relaxed) {
		let stdin = io::stdin();
		let sin = stdin.lock();
		let mut sinchars = sin.bytes();
		match sinchars.next() {
			Some(Ok(ch)) if ch != 0x1b => return ch as char,
			_ => EXITGAME.store(true, Ordering::Relaxed)
		}
	}
	'\x1b'
}

pub fn char_as_dir(ch: char) -> Result<Dir, char> {
	Ok(match ch {
		'h' => Dir::H,
		'j' => Dir::J,
		'k' => Dir::K,
		'l' => Dir::L,
		_ => return Err(ch),
	})
}

pub fn dir_as_off(dir: Dir) -> [i16; 2] {
	match dir {
		Dir::H => [-1, 0],
		Dir::J => [0, 1],
		Dir::K => [0, -1],
		Dir::L => [1, 0],
	}
}

pub fn xy_incr_dir(xy: &mut [i16], dir: Dir) {
	match dir {
		Dir::H => xy[0] -= 1,
		Dir::J => xy[1] += 1,
		Dir::K => xy[1] -= 1,
		Dir::L => xy[0] += 1,
	}
}

pub fn xy_plus_dir(mut xy: [i16; 2], dir: Dir) -> [i16; 2] {
	xy_incr_dir(&mut xy, dir);
	xy
}

pub fn xyz_plus_dir(mut xyz: [i16; 3], dir: Dir) -> [i16; 3] {
	xy_incr_dir(&mut xyz, dir);
	xyz
}
