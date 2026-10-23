#[cfg(feature = "fs")]
#[cfg(target_arch = "x86")]
#[cfg(test)]
pub(crate) use linux_raw_sys::general::stat64;
