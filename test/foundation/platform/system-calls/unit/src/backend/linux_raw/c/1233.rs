#[cfg(feature = "fs")]
#[cfg(test)]
pub(crate) use linux_raw_sys::general::{
    __kernel_fsid_t as fsid_t, stat, statfs64, statx, statx_timestamp,
};
