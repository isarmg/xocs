#[cfg(all(test, not(__ZEROCOPY_INTERNAL_USE_ONLY_DEV_MODE)))]
const _: () = {
    #[deprecated = "Development of zerocopy using cargo is not supported. Please use `cargo.sh` or `win-cargo.bat` instead."]
    #[allow(unused)]
    const WARNING: () = ();
    #[warn(deprecated)]
    WARNING
};
