use rand::Rng;
use rand::distributions::{IndependentSample, Range};

use util::rectoverinc;

/* Given a set of rects, grow them as much as possible
   Return adjacency matrix */
pub fn grow<R: Rng>(rng: &mut R, rxy: &mut [[i16;4]], xyz: [i16; 3], w: i16, h: i16) -> Vec<bool> {
	let bet4 = Range::new(0, 4);
	let rc = rxy.len();
	let mut adjacent = vec![false; rc*rc];
	let mut done = [false; 4];
	loop {
		let mut cangrow: Vec<bool> = Vec::with_capacity(rc);
		let b4 = bet4.ind_sample(rng);
		for (i, rect) in rxy.iter().enumerate() {
			let mut newrect = *rect;
			let oob = match b4 {
				0|1 => {
					newrect[b4] -= 1;
					newrect[b4] < xyz[if b4 == 0 { 0 } else { 1 }]
				},
				2|3 => {
					newrect[b4] += 1;
					newrect[b4] >= if b4 == 2 { w } else { h }
				},
				_ => unreachable!(),
			};
			if oob {
				cangrow.push(false);
				continue
			}
			let mut grow = true;
			for (j, rect2) in rxy.iter().enumerate() {
				if i != j && rectoverinc(newrect, *rect2){
					grow = false;
					adjacent[i+j*rc] = true;
					adjacent[i*rc+j] = true;
				}
			}
			cangrow.push(grow)
		}
		if cangrow.iter().all(|&x| !x) {
			done[b4] = true;
			if done.iter().all(|&x| x) { return adjacent }
		}
		for (ref mut xywh, &grow) in rxy.iter_mut().zip(&cangrow) {
			if !grow { continue }
			xywh[b4] += match b4 {
				0|1 => -1,
				2|3 => 1,
				_ => unreachable!(),
			}
		}
	}
}
