pub static SSE_F32_WIDTH: usize = 4;
pub static SSE_F64_WIDTH: usize = 2;

pub static AVX_F32_WIDTH: usize = 8;
pub static AVX_F64_WIDTH: usize = 4;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_AVX2: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_AVX: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_SSE4_1: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_SSE4_2: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_SSE2: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_SSE: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_FMA: bool = false;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static mut HAS_SSE2_FMA: bool = false;

pub fn init() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // Safe because flags are treated like constants after init() is run once.
        //
        // For some reason rust-analyzer gives an `unresolved macro` error, even
        // when it builds just fine.
        unsafe {
            if is_x86_feature_detected!("avx2") {
                HAS_AVX2 = true;
            }
            if is_x86_feature_detected!("avx") {
                HAS_AVX = true;
            }
            if is_x86_feature_detected!("sse4.1") {
                HAS_SSE4_1 = true;
            }
            if is_x86_feature_detected!("sse4.2") {
                HAS_SSE4_2 = true;
            }
            if is_x86_feature_detected!("sse2") {
                HAS_SSE2 = true;
            }
            if is_x86_feature_detected!("sse") {
                HAS_SSE = true;
            }
            if is_x86_feature_detected!("fma") {
                HAS_FMA = true;
                HAS_SSE2_FMA = HAS_SSE2;
            }
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_avx2() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_AVX2 }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_avx() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_AVX }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_sse4_1() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_SSE4_1 }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_sse4_2() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_SSE4_2 }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_sse2() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_SSE2 }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_sse() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_SSE }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_fma() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_FMA }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
pub fn has_sse2_fma() -> bool {
    // safe because flags are treated like constants after init() is run once
    unsafe { HAS_SSE2_FMA }
}
