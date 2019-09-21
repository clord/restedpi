pub fn iv2be(r: &[u8]) -> i16 {
    let rs = ((r[1] as u16) << 8) + (r[0] as u16);
    i2be(rs)
}

pub fn uv2be(r: &[u8]) -> u16 {
    let rs = ((r[1] as u16) << 8) + (r[0] as u16);
    rs.to_be()
}

pub fn i2be(r: u16) -> i16 {
    let a = r as i16;
    a.to_be()
}

pub fn u2be(r: u16) -> u16 {
    r.to_be()
}
