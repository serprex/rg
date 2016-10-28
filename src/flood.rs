use fnv::FnvHashSet;

pub fn fill<F>(set: &mut FnvHashSet<[i16; 2]>, x: i16, y: i16, x0: i16, y0: i16, x1: i16, y1: i16, pred: F)
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

pub fn holecandy<F>(set: &FnvHashSet<[i16; 2]>, x0: i16, y0: i16, x1: i16, y1: i16, pred: F) -> Vec<(i16, i16)>
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
					if !pred(x+xd, y+yd) && !set.contains(&[x+xd, y+yd]) && (
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
