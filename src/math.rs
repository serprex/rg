#[derive(Clone, Copy)]
pub enum Dir { E, NE, N, NW, W, SW, S, SE }
pub fn ch2dir(c: char) -> Option<Dir> {
	Some(match c {
		'l' => Dir::E,
		'i' => Dir::NE,
		'k' => Dir::N,
		'u' => Dir::NW,
		'h' => Dir::W,
		'n' => Dir::SW,
		'j' => Dir::S,
		'm' => Dir::SE,
		_ => return None,
	})
}
pub fn dir2i32(d: Dir) -> i32 {
	match d {
		Dir::E => 0,
		Dir::NE => 1,
		Dir::N => 2,
		Dir::NW => 3,
		Dir::W => 4,
		Dir::SW => 5,
		Dir::S => 6,
		Dir::SE => 7,
	}
}
pub fn isdiag(d: Dir) -> bool {
	match d {
		Dir::E | Dir::N | Dir::W | Dir::S => false,
		Dir::NE | Dir::NW | Dir::SW | Dir::SE => true,
	}
}
pub fn calcdist2(xy1: (i32, i32), xy2: (i32, i32)) -> i32{
	let (x1, y1) = xy1;
	let (x2, y2) = xy2;
	(x1-x2)*(y1-y2) // TODO calc with diag limitation
}
pub fn rectover(r1: &(i32,i32,i32,i32), r2: &(i32,i32,i32,i32)) -> bool {
	r1.0 <= r2.2 && r1.2 >= r2.0 && r1.1 <= r2.3 && r1.3 >= r2.1
}
//fn dir2rad(d: Dir) -> f64 {
//	(dir2i32(d) as f64)*3.141592
//}
