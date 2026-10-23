cfg_if::cfg_if! {
    if #[cfg(any(sha2_backend = "soft", sha2_512_backend = "soft"))] {
        mod soft;
        use soft::compress;
    } else if #[cfg(sha2_512_backend = "x86-avx2")] {
        mod x86_avx2;

        #[cfg(not(target_feature = "avx2"))]
        compile_error!("x86-avx2 backend requires avx2 target features");

        fn compress(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
            // SAFETY: the required target features are checked above.
            unsafe { x86_avx2::compress(state, blocks) }
        }
    } else if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
        mod wasm32_simd128;
        use wasm32_simd128::compress;
    } else {
        mod soft;

        #[cfg(target_arch = "x86_64")]
        mod x86_avx2;
        #[cfg(target_arch = "x86_64")]
        cpufeatures::new!(hw_features, "avx2");

        fn compress(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
            #[cfg(target_arch = "x86_64")]
            if hw_features::get() {
                // SAFETY: the required target features are available.
                return unsafe { x86_avx2::compress(state, blocks) };
            }

            soft::compress(state, blocks);
        }
    }
}

/// Raw SHA-512 compression function.
///
/// This is a low-level "hazmat" API which provides direct access to the core
/// functionality of SHA-512.
pub fn compress512(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
    compress(state, blocks)
}
