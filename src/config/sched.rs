/// Compute the number of hours for a given day of the year (from jan 1) and at given latitude
pub fn day_length_hrs(lat: f64, day_of_year: f64) -> f64 {
    let ha = hour_angle_sunrise(lat.to_radians(), noon_decl_sun(day_of_year));
    2.0_f64 * ha.to_degrees() / (360.0f64 / 24.0f64)
}

/// Declination of the sun at noon given a day of the year in radians, accurate to within ± 0.2°
/// Noon declination δ is the angle of the sun to the horizon at solar noon for a location.
pub fn noon_decl_sun(day_of_year: f64) -> f64 {
    let rads_per_day = 0.98565_f64.to_radians();
    (0.39779_f64
        * ((rads_per_day * (day_of_year + 10.0f64))
            + 1.914_f64.to_radians() * (rads_per_day * (day_of_year - 2f64)).sin())
        .cos())
    .asin()
}

/// Compute the hour-angle of sunrise for a given latitude and declination in radians.
///   lat: position on earth north/south expressed as an angle in radians
///     δ: declination of the sun in radians (determined by time of year)
pub fn hour_angle_sunrise(lat: f64, decl: f64) -> f64 {
    (((90.833_f64.to_radians()).cos() / lat.cos() * decl.cos()) - lat.tan() * decl.tan()).acos()
}
