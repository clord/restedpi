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

#[cfg(test)]
mod tests {
    use crate::i2c::util;

    #[test]
    fn basic() {
        assert_eq!(util::i2be(50u16), 12800i16);
        assert_eq!(util::u2be(50u16), 12800u16);
        assert_eq!(util::iv2be(&[1u8, 2u8]), 258i16);
        assert_eq!(util::uv2be(&[1u8, 2u8]), 258u16);
        assert_eq!(util::iv2be(&[255u8, 10u8]), -246i16);
        assert_eq!(util::uv2be(&[255u8, 10u8]), 65290u16);
    }
}
