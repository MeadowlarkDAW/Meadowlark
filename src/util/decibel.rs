/// Returns the raw amplitude (coefficient) from the given decibel value.
#[inline]
pub fn db_to_coeff_f32(db: f32) -> f32 {
    10.0f32.powf(0.05 * db)
}

/// Returns the decibel value from the raw amplitude (coefficient).
#[inline]
pub fn coeff_to_db_f32(coeff: f32) -> f32 {
    20.0 * coeff.log(10.0)
}

/// Returns the raw amplitude (coefficient) from the given decibel value.
///
/// If `db <= -90.0`, then 0.0 will be returned instead (negative infinity gain).
#[inline]
pub fn db_to_coeff_clamped_neg_90_db_f32(db: f32) -> f32 {
    if db <= -90.0 {
        0.0
    } else {
        db_to_coeff_f32(db)
    }
}

/// Returns the raw amplitude (coefficient) from the given decibel value.
///
/// If `coeff <= 0.00003162278`, then the minimum of `-90.0` dB will be
/// returned instead (representing negative infinity gain when paired with
/// `db_to_coeff_clamped_neg_90_db_f32`).
#[inline]
pub fn coeff_to_db_clamped_neg_90_db_f32(coeff: f32) -> f32 {
    if coeff <= 0.00003162278 {
        -90.0
    } else {
        coeff_to_db_f32(coeff)
    }
}

/// Returns the raw amplitude (coefficient) from the given decibel value.
#[inline]
pub fn db_to_coeff_f64(db: f64) -> f64 {
    10.0f64.powf(0.05 * db)
}

/// Returns the decibel value from the raw amplitude (coefficient).
#[inline]
pub fn coeff_to_db_f64(coeff: f64) -> f64 {
    20.0 * coeff.log(10.0)
}

/// Returns the raw amplitude (coefficient) from the given decibel value.
///
/// If `db <= -90.0`, then 0.0 will be returned instead (negative infinity gain).
#[inline]
pub fn db_to_coeff_clamped_neg_90_db_f64(db: f64) -> f64 {
    if db <= -90.0 {
        0.0
    } else {
        db_to_coeff_f64(db)
    }
}

/// Returns the raw amplitude (coefficient) from the given decibel value.
///
/// If `coeff <= 0.00003162278`, then the minimum of `-90.0` dB will be
/// returned instead (representing negative infinity gain when paired with
/// `db_to_coeff_clamped_neg_90_db_f64`).
#[inline]
pub fn coeff_to_db_clamped_neg_90_db_f64(coeff: f64) -> f64 {
    if coeff <= 0.00003162278 {
        -90.0
    } else {
        coeff_to_db_f64(coeff)
    }
}
