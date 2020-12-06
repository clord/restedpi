/// vec of two bytes to big endian i16
///
/// ```
/// assert_eq!(librpi::rpi::i2c::util::iv2be(&[1u8, 2u8]), 258i16);
/// assert_eq!(librpi::rpi::i2c::util::iv2be(&[255u8, 10u8]),-246i16);
/// ```
pub fn iv2be(r: &[u8]) -> i16 {
    let rs = ((r[1] as u16) << 8) + (r[0] as u16);
    i2be(rs)
}

/// vec of two bytes to big endian u16
///
/// ```
/// assert_eq!(librpi::rpi::i2c::util::uv2be(&[1u8, 2u8]), 258u16);
/// assert_eq!(librpi::rpi::i2c::util::uv2be(&[255u8, 10u8]), 65290u16);
/// ```
pub fn uv2be(r: &[u8]) -> u16 {
    let rs = ((r[1] as u16) << 8) + (r[0] as u16);
    rs.to_be()
}

/// i16 to u16
///
/// ```
/// assert_eq!(librpi::rpi::i2c::util::i2be(50u16), 12800i16);
/// ```
pub fn i2be(r: u16) -> i16 {
    let a = r as i16;
    a.to_be()
}

/// u16 to big-endian u16
///
/// ```
/// assert_eq!(librpi::rpi::i2c::util::u2be(50u16), 12800u16);
/// ```
pub fn u2be(r: u16) -> u16 {
    r.to_be()
}
