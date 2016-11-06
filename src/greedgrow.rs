use std::cmp;

use fnv::FnvHashSet;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};

use adjacency::{Adjacency, AdjacencySet};
use util::{R, rectover, rectoverinc};

pub fn init(rng: &mut R, n: usize, minwh: i16, dim: [i16; 4]) -> Vec<[i16; 4]> {
	let w = dim[2];
	let h = dim[3];
	let n = cmp::min(n, (w as usize * h as usize) / (minwh as usize * minwh as usize));
	if n < 2 {
		vec![dim]
	} else {
		let betwh = Range::new(0, (w-2)*(h-2));
		let mut rxy = Vec::with_capacity(n);
		while rxy.len() < n {
			let xy = betwh.ind_sample(rng);
			let (rx, ry) = (dim[0] + xy % (w-2), dim[1] + xy / (w-2));
			let candy = [rx, ry, rx+minwh-1, ry+minwh-1];
			if !rxy.iter().any(|&a| rectover(candy, a))
				{ rxy.push(candy) }
		}
		rxy
	}
}

/* Given a set of rects, grow them as much as possible
   Return adjacency matrix */
pub fn grow(rng: &mut R, rxy: &mut [[i16;4]], dim: [i16; 4])
-> AdjacencySet {
	let bet4 = Range::new(0, 4);
	let rc = rxy.len();
	let mut adjacent = AdjacencySet::default();
	let mut done = [false; 4];
	loop {
		let mut cangrow: Vec<bool> = Vec::with_capacity(rc);
		let b4 = bet4.ind_sample(rng);
		for (i, rect) in rxy.iter().enumerate() {
			let mut newrect = *rect;
			let oob = match b4 {
				0|1 => {
					newrect[b4] -= 1;
					newrect[b4] < dim[if b4 == 0 { 0 } else { 1 }]
				},
				2|3 => {
					newrect[b4] += 1;
					newrect[b4] >= dim[if b4 == 2 { 2 } else { 3 }]
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
					adjacent.insert(i, j);
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

pub fn joinlist<A: Adjacency>(rng: &mut R, adj: &A, rc: usize)
-> Vec<(usize, usize)> {
	let mut ret = Vec::with_capacity(rc);
	if rc == 0 { return ret }
	let mut nzgrps: FnvHashSet<usize> = (1..rc).into_iter().collect();
	let mut zgrps: FnvHashSet<usize> = FnvHashSet::with_capacity_and_hasher(rc, Default::default());
	zgrps.insert(0);
	loop {
		let nthzi = rng.gen_range(0, zgrps.len());
		let &iszi = zgrps.iter().skip(nthzi).next().unwrap();
		let adjs = (0..rc).into_iter()
			.filter(|&i| adj.contains(i, iszi))
			.collect::<Vec<_>>();
		if let Some(&aidx) = rng.choose(&adjs) {
			ret.push((iszi, aidx));
			zgrps.insert(aidx);
			nzgrps.remove(&aidx);
			if nzgrps.is_empty() {
				return ret
			}
		} else {
			return ret
		}
	}
}

pub fn doors<D>(rng: &mut R, connset: D, rxy: &[[i16; 4]])
-> (FnvHashSet<[i16; 2]>, usize)
where D: Iterator<Item = (usize, usize)>
{
	let mut lastaidx = 0;
	let mut ret = FnvHashSet::default();
	for (iszi, aidx) in connset {
		lastaidx = aidx;
		let r1 = rxy[iszi];
		let r2 = rxy[aidx];
		for &(r1i, r2i, mxi, mni) in &[(0, 2, 1, 3), (2, 0, 1, 3), (1, 3, 0, 2), (3, 1, 0, 2)] {
			if r1[r1i] == r2[r2i] {
				let mn = cmp::max(r1[mxi], r2[mxi])+1;
				let mx = cmp::min(r1[mni], r2[mni]);
				if mn != mx {
					let mnx = rng.gen_range(mn, mx);
					ret.insert(if mxi == 1 { [r1[r1i],mnx] } else { [mnx,r1[r1i]] });
				}
				break
			}
		}
	}
	(ret, lastaidx)
}
