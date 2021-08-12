// Basic low-quality (but fast) linear resampler.

/// Basic low-quality (but fast) linear resampler.
///
/// This function allocates memory and is *not* realtime safe. It is intended for
/// resampling audio clips to be sent to the rt thread.
pub fn linear_resample_non_rt_mono(
    src: &[f32],
    // The ratio between the dst samplerate / src samplerate.
    resample_ratio: f64,
) -> Vec<f32> {
    let dst_len = ((src.len() - 1) as f64 * resample_ratio).ceil() as usize;

    let mut dst = Vec::<f32>::with_capacity(dst_len);

    let ratio_inv = 1.0 / resample_ratio;

    // TODO: SIMD optimizations.

    for i in 0..dst_len {
        let src_pos = i as f64 * ratio_inv;
        let src_i = src_pos.floor() as usize;
        let fract = src_pos.fract();

        let smp_before = src[src_i];
        let smp_after = src[src_i + 1];

        dst.push(smp_before + ((smp_after - smp_before) * fract as f32));
    }

    dst
}

/// Basic low-quality (but fast) linear resampler.
///
/// This function allocates memory and is *not* realtime safe. It is intended for
/// resampling audio clips to be sent to the rt thread.
pub fn linear_resample_non_rt_stereo(
    src_l: &[f32],
    src_r: &[f32],
    // The ratio between the dst samplerate / src samplerate.
    resample_ratio: f64,
) -> (Vec<f32>, Vec<f32>) {
    // Make sure we are given valid slices.
    let len = src_l.len().min(src_r.len());
    let src_l = &src_l[0..len];
    let src_r = &src_r[0..len];

    let dst_len = ((src_l.len() - 1) as f64 * resample_ratio).ceil() as usize;

    let mut dst_l = Vec::<f32>::with_capacity(dst_len);
    let mut dst_r = Vec::<f32>::with_capacity(dst_len);

    let ratio_inv = 1.0 / resample_ratio;

    // TODO: SIMD optimizations.

    for i in 0..dst_len {
        let src_pos = i as f64 * ratio_inv;
        let src_i = src_pos.floor() as usize;
        let fract = src_pos.fract();

        let smp_before_l = src_l[src_i];
        let smp_before_r = src_r[src_i];

        let smp_after_l = src_l[src_i + 1];
        let smp_after_r = src_r[src_i + 1];

        dst_l.push(smp_before_l + ((smp_after_l - smp_before_l) * fract as f32));
        dst_r.push(smp_before_r + ((smp_after_r - smp_before_r) * fract as f32));
    }

    (dst_l, dst_r)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Quick test to make sure there are no off-by-one errors.
    #[test]
    fn test_linear_resample_non_rt() {
        let mut src = Vec::<f32>::with_capacity(1000);
        let mut src_r = Vec::<f32>::with_capacity(1000);
        src.resize(1000, 0.0);
        src_r.resize(1000, 0.0);

        let _dst = linear_resample_non_rt_mono(src.as_slice(), 44100.0 / 48000.0);
        let _dst = linear_resample_non_rt_mono(src.as_slice(), 48000.0 / 44100.0);
        let _dst = linear_resample_non_rt_mono(src.as_slice(), 1.0 / 2.0);
        let _dst = linear_resample_non_rt_mono(src.as_slice(), 2.0);
    }
}
