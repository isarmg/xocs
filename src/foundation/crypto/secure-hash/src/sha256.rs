cfg_if::cfg_if! {
    if #[cfg(any(sha2_backend = "soft", sha2_256_backend = "soft"))] {
        mod soft;
        use soft::compress;
    } else if #[cfg(sha2_256_backend = "x86-sha")] {
        mod x86_sha;

        #[cfg(not(all(target_feature = "sha", target_feature = "sse2", target_feature = "ssse3", target_feature = "sse4.1")))]
        compile_error!("x86-sha backend requires sha,sse2,ssse3,sse4.1 target features");

        fn compress(state: &mut [u32; 8], blocks: &[[u8; 64]]) {
            // SAFETY: the required target features are checked above.
            unsafe { x86_sha::compress(state, blocks) }
        }
    } else if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
        mod wasm32_simd128;
        use wasm32_simd128::compress;
    } else {
        mod soft;

        #[cfg(target_arch = "x86_64")]
        mod x86_sha;
        #[cfg(target_arch = "x86_64")]
        cpufeatures::new!(hw_features, "sha", "sse2", "ssse3", "sse4.1");

        fn compress(state: &mut [u32; 8], blocks: &[[u8; 64]]) {
            #[cfg(target_arch = "x86_64")]
            if hw_features::get() {
                // SAFETY: the required target features are available.
                return unsafe { x86_sha::compress(state, blocks) };
            }

            soft::compress(state, blocks);
        }
    }
}

/// Raw SHA-256 compression function.
///
/// This is a low-level "hazmat" API which provides direct access to the core
/// functionality of SHA-256.
pub fn compress256(state: &mut [u32; 8], blocks: &[[u8; 64]]) {
    compress(state, blocks)
}
