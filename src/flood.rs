use std::cmp;
use fnv::FnvHashSet;

#[derive(Copy, Clone)]
struct Segment(pub i16, pub i16);

impl Segment {
	pub fn grow<F>(&mut self, set: &mut FnvHashSet<[i16; 2]>, x0: i16, x1: i16, y: i16, pred: &F)
		where F: Fn(i16, i16) -> bool
	{
		while self.0 - 1 >= x0 && !set.contains(&[self.0 - 1, y]) && !pred(self.0 - 1, y) {
			self.0 -= 1;
			set.insert([self.0, y]);
		}
		while self.1 + 1 < x1 && !set.contains(&[self.1 + 1, y]) && !pred(self.1 + 1, y) {
			self.1 += 1;
			set.insert([self.1, y]);
		}
	}

	pub fn roll<F>(self, set: &mut FnvHashSet<[i16; 2]>, y: i16, x0: i16, x1: i16, canup: bool, pred: &F, up: &mut Vec<(i16, Segment)>, down: &mut Vec<(i16, Segment)>)
		where F: Fn(i16, i16) -> bool
	{
		let lx = if set.contains(&[self.0, y]) || pred(self.0, y) {
			let lx;
			'lxanyloop: loop {
				for x in self.0+1..self.1+1 {
					if !set.contains(&[x, y]) && !pred(x, y) {
						set.insert([x, y]);
						lx = x;
						break 'lxanyloop
					}
				}
				return
			}
			lx
		} else {
			let mut lx = self.0;
			set.insert([lx, y]);
			while lx - 1 >= x0 && !set.contains(&[lx - 1, y]) && !pred(lx - 1, y) {
				lx -= 1;
				set.insert([lx, y]);
			}
			if lx <= self.0 - 2 {
				down.push((y, Segment(lx, self.0 - 2)));
			}
			lx
		};
		let rx = if lx < self.1 && self.0 != self.1 && (set.contains(&[self.1, y]) || pred(self.1, y)) {
			let rx;
			'rxanyloop: loop {
				for x in (lx+1..self.1).rev() {
					if !pred(x, y) {
						set.insert([x, y]);
						rx = x;
						break 'rxanyloop
					}
				}
				if canup {
					up.push((y, Segment(lx, lx)));
				}
				return
			}
			rx
		} else {
			let mut rx = self.1;
			set.insert([rx, y]);
			while rx + 1 < x1 && !set.contains(&[rx + 1, y]) && !pred(rx + 1, y) {
				rx += 1;
				set.insert([rx, y]);
			}
			if rx >= self.1 + 2 {
				down.push((y, Segment(self.1 + 2, rx)));
			}
			rx
		};
		let mut lastpred = Some(lx);
		for x in cmp::max(lx, self.0)+1..cmp::min(rx, self.1) {
			if !pred(x, y) {
				set.insert([x, y]);
				if lastpred.is_none() {
					lastpred = Some(x);
				}
			} else if let Some(lp) = lastpred {
				if canup {
					up.push((y, Segment(lp, x - 1)));
				}
				lastpred = None;
			}
		}
		if canup {
			up.push((y, Segment(
				if let Some(lp) = lastpred { lp } else { cmp::min(rx, self.1) }, rx
			)));
		}
	}
}

pub fn fill<F>(set: &mut FnvHashSet<[i16; 2]>, x: i16, y: i16, x0: i16, y0: i16, x1: i16, y1: i16, pred: &F)
	where F: Fn(i16, i16) -> bool
{
	let mut xx0 = x;
	let mut xx1 = x;
	set.insert([x, y]);
	let mut seg = Segment(x, x);
	seg.grow(set, x0, x1, y, pred);
	let mut up = vec![];
	let mut down = vec![];
	if y + 1 < y1 {
		seg.roll(set, y + 1, x0, x1, y + 2 < y1, pred, &mut down, &mut up);
	}
	if y - 1 >= y0 {
		seg.roll(set, y - 1, x0, x1, y - 2 >= y0, pred, &mut up, &mut down);
	}
	loop {
		if down.len() > up.len() {
			if let Some((y, seg)) = down.pop() {
				seg.roll(set, y + 1, x0, x1, y + 2 < y1, pred, &mut down, &mut up);
			} else {
				unreachable!();
			}
		} else if let Some((y, seg)) = up.pop() {
			seg.roll(set, y - 1, x0, x1, y - 2 >= y0, pred, &mut up, &mut down);
		} else {
			break
		}
	}
}

#[allow(dead_code)]
pub fn basicfill<F>(set: &mut FnvHashSet<[i16; 2]>, x: i16, y: i16, x0: i16, y0: i16, x1: i16, y1: i16, pred: &F)
	where F: Fn(i16, i16) -> bool
{
	let mut ffs = vec![[x, y]];
	while let Some(xy) = ffs.pop() {
		set.insert(xy);
		for &(x, y, b) in &[(xy[0]+1, xy[1], xy[0]+1 < x1),
			(xy[0], xy[1]+1, xy[1]+1 < y1),
			(xy[0]-1, xy[1], xy[0]-1 >= x0),
			(xy[0], xy[1]-1, xy[1]-1 >= y0)
		] {
			if b && !set.contains(&[x, y]) && !pred(x, y) {
				ffs.push([x, y]);
			}
		}
	}
}

pub fn holecandy<F>(set: &FnvHashSet<[i16; 2]>, x0: i16, y0: i16, x1: i16, y1: i16, pred: &F) -> Vec<(i16, i16)>
	where F: Fn(i16, i16) -> bool
{
	let mut candy = Vec::new();
	// TODO we won't detect a 2-tile thick wall divide. FIX scan for first non-wall tile
	for x in x0..x1 {
		for y in y0..y1 {
			if pred(x, y) {
				for &(xd, yd, x1d, y1d, x2d, y2d, x3d, y3d) in &[
					(0, 1, 0, -1, 1, 0, -1, 0),
					(1, 0, 0, -1, 0, 1, -1, 0),
					(-1, 0, 0, 1, 1, 0, 0, -1),
					(0, -1, 0, 1, 1, 0, -1, 0),
				] {
					if !set.contains(&[x+xd, y+yd]) && !pred(x+xd, y+yd) && (
						set.contains(&[x+x1d, y+y1d]) ||
						set.contains(&[x+x2d, y+y2d]) ||
						set.contains(&[x+x3d, y+y3d]))
					{
						candy.push((x, y));
						break
					}
				}
			}
		}
	}
	candy
}
