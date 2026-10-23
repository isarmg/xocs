
    use super::ctypes;
    static_assertions::assert_eq_size!(ctypes::c_char, libc::c_char);
    static_assertions::assert_type_eq_all!(ctypes::c_schar, libc::c_schar);
    static_assertions::assert_type_eq_all!(ctypes::c_uchar, libc::c_uchar);
    static_assertions::assert_type_eq_all!(ctypes::c_short, libc::c_short);
    static_assertions::assert_type_eq_all!(ctypes::c_ushort, libc::c_ushort);
    static_assertions::assert_type_eq_all!(ctypes::c_int, libc::c_int);
    static_assertions::assert_type_eq_all!(ctypes::c_uint, libc::c_uint);
    static_assertions::assert_type_eq_all!(ctypes::c_long, libc::c_long);
    static_assertions::assert_type_eq_all!(ctypes::c_ulong, libc::c_ulong);
    static_assertions::assert_type_eq_all!(ctypes::c_longlong, libc::c_longlong);
    static_assertions::assert_type_eq_all!(ctypes::c_ulonglong, libc::c_ulonglong);
    static_assertions::assert_type_eq_all!(ctypes::c_float, libc::c_float);
    static_assertions::assert_type_eq_all!(ctypes::c_double, libc::c_double);
