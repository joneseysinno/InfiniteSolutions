//! [`level`] — zoom, resolved to a number of significant address bits (spec §7.1).

/// How many bits of an address are significant at this magnification.
///
/// One bit per doubling of zoom. That is not a measured constant and it is not a
/// guess: **level is address truncation** (spec §7.2), and doubling the magnification
/// is the unique zoom that reveals one more bit. `todo!` lived here because the
/// skeleton could not say that before the function existed.
pub fn level(zoom: f64) -> u32 {
    if !zoom.is_finite() || zoom <= 1.0 {
        return 0;
    }
    zoom.log2().floor() as u32
}

#[cfg(test)]
mod tests {
    use super::level;

    #[test]
    fn unity_zoom_is_level_zero() {
        assert_eq!(level(1.0), 0);
        assert_eq!(level(0.5), 0);
    }

    #[test]
    fn each_doubling_adds_one_bit() {
        assert_eq!(level(2.0), 1);
        assert_eq!(level(4.0), 2);
        assert_eq!(level(8.0), 3);
    }
}
