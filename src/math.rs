pub fn rectover(r1: [i16; 4], r2: [i16; 4]) -> bool {
	r1[0] <= r2[2] && r1[2] >= r2[0] && r1[1] <= r2[3] && r1[3] >= r2[1]
}
pub fn rectoverinc(r1: [i16; 4], r2: [i16; 4]) -> bool {
	r1[0] < r2[2] && r1[2] > r2[0] && r1[1] < r2[3] && r1[3] > r2[1]
}
