use fnv::FnvHashSet;

#[derive(Copy, Clone)]
struct Segment {
	pub y: i16,
	pub x0: i16,
	pub x1: i16,
}

impl Segment {
	pub fn grow<F>(&mut self, set: &mut FnvHashSet<[i16; 2]>, x0: i16, x1: i16, pred: &F)
		where F: Fn(i16, i16) -> bool
	{
		while self.x0 - 1 >= x0 && !pred(self.x0 - 1, self.y) {
			self.x0 -= 1;
			set.insert([self.x0, self.y]);
		}
		while self.x1 + 1 < x1 && !pred(self.x1 + 1, self.y) {
			self.x1 += 1;
			set.insert([self.x1, self.y]);
		}
	}

	pub fn roll<F>(self, set: &mut FnvHashSet<[i16; 2]>, y: i16, x0: i16, x1: i16, pred: &F, up: &mut Vec<Segment>, down: &mut Vec<Segment>)
		where F: Fn(i16, i16) -> bool
	{
		let lx = if set.contains(&[self.x0, y]) || pred(self.x0, y) {
			set.insert([self.x0, y]);
			let lx;
			'lxanyloop: loop {
				for x in self.x0+1..self.x1+1 {
					if !pred(x, y) {
						set.insert([x, y]);
						lx = x;
						break 'lxanyloop
					}
				}
				return
			}
			lx
		} else {
			let mut lx = self.x0 - 1;
			while lx >= x0 && !set.contains(&[lx, y]) && !pred(lx, y) {
				set.insert([lx, y]);
				lx -= 1;
			}
			let lx = lx + 1;
			if lx != self.x0 {
				down.push(Segment { x0: lx, x1: self.x0 - 1, y: y });
			}
			lx
		};
		let rx = if set.contains(&[self.x1, y]) || pred(self.x1, y) {
			set.insert([self.x1, y]);
			let rx;
			'rxanyloop: loop {
				for x in (lx+1..self.x1).rev() {
					if !set.contains(&[x, y]) && !pred(x, y) {
						set.insert([x, y]);
						rx = x;
						break 'rxanyloop
					}
				}
				up.push(Segment { x0: lx, x1: lx, y: y });
				return
			}
			rx
		} else {
			let mut rx = self.x1 + 1;
			while rx < x1 && !set.contains(&[rx, y]) && !pred(rx, y) {
				set.insert([rx, y]);
				rx += 1;
			}
			let rx = rx - 1;
			if rx != self.x1 {
				down.push(Segment { x0: self.x1 + 1, x1: rx, y: y });
			}
			rx
		};
		let mut lastpred = Some(lx);
		for x in self.x0..self.x1 + 1 {
			if !set.contains(&[x, y]) && !pred(x, y) {
				set.insert([x, y]);
				if lastpred.is_none() {
					lastpred = Some(x);
				}
			} else if let Some(lp) = lastpred {
				up.push(Segment { x0: lp, x1: x - 1, y: y });
				lastpred = None;
			}
		}
		if let Some(lp) = lastpred {
			up.push(Segment { x0: lp, x1: rx, y: y });
		}
	}
}

pub fn fill<F>(set: &mut FnvHashSet<[i16; 2]>, x: i16, y: i16, x0: i16, y0: i16, x1: i16, y1: i16, pred: &F)
	where F: Fn(i16, i16) -> bool
{
	let mut xx0 = x;
	let mut xx1 = x;
	set.insert([x, y]);
	let mut seg0 = Segment { y: y, x0: x, x1: x };
	seg0.grow(set, x0, x1, pred);
	let mut up = vec![];
	let mut down = vec![seg0];
	loop {
		while let Some(seg) = down.pop() {
			if seg.y > y1 { continue }
			seg.roll(set, seg.y + 1, x0, x1, pred, &mut down, &mut up);
		}
		while let Some(seg) = up.pop() {
			if seg.y < y0 { continue }
			seg.roll(set, seg.y - 1, x0, x1, pred, &mut up, &mut down);
		}
		if down.is_empty() { return }
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
