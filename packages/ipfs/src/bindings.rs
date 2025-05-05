#[allow(dead_code)]
pub mod wavs {
    #[allow(dead_code)]
    pub mod ipfs {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// Error type for operations that can fail
            pub type Error = _rt::String;
            /// IPFS URL type
            pub type IpfsUrl = _rt::String;
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod wavs {
        #[allow(dead_code)]
        pub mod ipfs {
            #[allow(dead_code, clippy::all)]
            pub mod ipfs {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type IpfsUrl = super::super::super::super::wavs::ipfs::types::IpfsUrl;
                pub type Error = super::super::super::super::wavs::ipfs::types::Error;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_upload_json_to_ipfs_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let result2 = T::upload_json_to_ipfs(
                        _rt::string_lift(bytes0),
                        _rt::string_lift(bytes1),
                    );
                    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result2 {
                        Ok(e) => {
                            *ptr3.add(0).cast::<u8>() = (0i32) as u8;
                            let vec4 = (e.into_bytes()).into_boxed_slice();
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            ::core::mem::forget(vec4);
                            *ptr3.add(8).cast::<usize>() = len4;
                            *ptr3.add(4).cast::<*mut u8>() = ptr4.cast_mut();
                        }
                        Err(e) => {
                            *ptr3.add(0).cast::<u8>() = (1i32) as u8;
                            let vec5 = (e.into_bytes()).into_boxed_slice();
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            ::core::mem::forget(vec5);
                            *ptr3.add(8).cast::<usize>() = len5;
                            *ptr3.add(4).cast::<*mut u8>() = ptr5.cast_mut();
                        }
                    };
                    ptr3
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_upload_json_to_ipfs<T: Guest>(
                    arg0: *mut u8,
                ) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                        _ => {
                            let l3 = *arg0.add(4).cast::<*mut u8>();
                            let l4 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l3, l4, 1);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_upload_image_to_ipfs_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                    arg4: *mut u8,
                    arg5: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let len2 = arg5;
                    let bytes2 = _rt::Vec::from_raw_parts(arg4.cast(), len2, len2);
                    let result3 = T::upload_image_to_ipfs(
                        _rt::Vec::from_raw_parts(arg0.cast(), len0, len0),
                        _rt::string_lift(bytes1),
                        _rt::string_lift(bytes2),
                    );
                    let ptr4 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result3 {
                        Ok(e) => {
                            *ptr4.add(0).cast::<u8>() = (0i32) as u8;
                            let vec5 = (e.into_bytes()).into_boxed_slice();
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            ::core::mem::forget(vec5);
                            *ptr4.add(8).cast::<usize>() = len5;
                            *ptr4.add(4).cast::<*mut u8>() = ptr5.cast_mut();
                        }
                        Err(e) => {
                            *ptr4.add(0).cast::<u8>() = (1i32) as u8;
                            let vec6 = (e.into_bytes()).into_boxed_slice();
                            let ptr6 = vec6.as_ptr().cast::<u8>();
                            let len6 = vec6.len();
                            ::core::mem::forget(vec6);
                            *ptr4.add(8).cast::<usize>() = len6;
                            *ptr4.add(4).cast::<*mut u8>() = ptr6.cast_mut();
                        }
                    };
                    ptr4
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_upload_image_to_ipfs<T: Guest>(
                    arg0: *mut u8,
                ) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                        _ => {
                            let l3 = *arg0.add(4).cast::<*mut u8>();
                            let l4 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l3, l4, 1);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_delete_file_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let result1 = T::delete_file(_rt::string_lift(bytes0));
                    let ptr2 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result1 {
                        Ok(e) => {
                            *ptr2.add(0).cast::<u8>() = (0i32) as u8;
                            *ptr2.add(4).cast::<u8>() = (match e {
                                true => 1,
                                false => 0,
                            }) as u8;
                        }
                        Err(e) => {
                            *ptr2.add(0).cast::<u8>() = (1i32) as u8;
                            let vec3 = (e.into_bytes()).into_boxed_slice();
                            let ptr3 = vec3.as_ptr().cast::<u8>();
                            let len3 = vec3.len();
                            ::core::mem::forget(vec3);
                            *ptr2.add(8).cast::<usize>() = len3;
                            *ptr2.add(4).cast::<*mut u8>() = ptr3.cast_mut();
                        }
                    };
                    ptr2
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_delete_file<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {}
                        _ => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_get_ipfs_url_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: i32,
                    arg3: *mut u8,
                    arg4: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let result2 = T::get_ipfs_url(
                        _rt::string_lift(bytes0),
                        match arg2 {
                            0 => None,
                            1 => {
                                let e = {
                                    let len1 = arg4;
                                    let bytes1 = _rt::Vec::from_raw_parts(
                                        arg3.cast(),
                                        len1,
                                        len1,
                                    );
                                    _rt::string_lift(bytes1)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        },
                    );
                    let ptr3 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec4 = (result2.into_bytes()).into_boxed_slice();
                    let ptr4 = vec4.as_ptr().cast::<u8>();
                    let len4 = vec4.len();
                    ::core::mem::forget(vec4);
                    *ptr3.add(4).cast::<usize>() = len4;
                    *ptr3.add(0).cast::<*mut u8>() = ptr4.cast_mut();
                    ptr3
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_get_ipfs_url<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l0, l1, 1);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_upload_nft_content_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                    arg4: *mut u8,
                    arg5: usize,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let len2 = arg5;
                    let bytes2 = _rt::Vec::from_raw_parts(arg4.cast(), len2, len2);
                    let result3 = T::upload_nft_content(
                        _rt::string_lift(bytes0),
                        _rt::Vec::from_raw_parts(arg2.cast(), len1, len1),
                        _rt::string_lift(bytes2),
                    );
                    let ptr4 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    match result3 {
                        Ok(e) => {
                            *ptr4.add(0).cast::<u8>() = (0i32) as u8;
                            let vec5 = (e.into_bytes()).into_boxed_slice();
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            ::core::mem::forget(vec5);
                            *ptr4.add(8).cast::<usize>() = len5;
                            *ptr4.add(4).cast::<*mut u8>() = ptr5.cast_mut();
                        }
                        Err(e) => {
                            *ptr4.add(0).cast::<u8>() = (1i32) as u8;
                            let vec6 = (e.into_bytes()).into_boxed_slice();
                            let ptr6 = vec6.as_ptr().cast::<u8>();
                            let len6 = vec6.len();
                            ::core::mem::forget(vec6);
                            *ptr4.add(8).cast::<usize>() = len6;
                            *ptr4.add(4).cast::<*mut u8>() = ptr6.cast_mut();
                        }
                    };
                    ptr4
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_upload_nft_content<T: Guest>(arg0: *mut u8) {
                    let l0 = i32::from(*arg0.add(0).cast::<u8>());
                    match l0 {
                        0 => {
                            let l1 = *arg0.add(4).cast::<*mut u8>();
                            let l2 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l1, l2, 1);
                        }
                        _ => {
                            let l3 = *arg0.add(4).cast::<*mut u8>();
                            let l4 = *arg0.add(8).cast::<usize>();
                            _rt::cabi_dealloc(l3, l4, 1);
                        }
                    }
                }
                pub trait Guest {
                    /// Upload JSON data to IPFS
                    fn upload_json_to_ipfs(
                        json_data: _rt::String,
                        ipfs_url: _rt::String,
                    ) -> Result<IpfsUrl, Error>;
                    /// Upload image to IPFS
                    fn upload_image_to_ipfs(
                        image_data: _rt::Vec<u8>,
                        filename: _rt::String,
                        ipfs_url: _rt::String,
                    ) -> Result<IpfsUrl, Error>;
                    /// Delete a file from the filesystem
                    fn delete_file(file_path: _rt::String) -> Result<bool, Error>;
                    /// Get IPFS URL from CID
                    fn get_ipfs_url(
                        cid: _rt::String,
                        filename: Option<_rt::String>,
                    ) -> IpfsUrl;
                    /// Upload NFT content (metadata and/or image) to IPFS
                    fn upload_nft_content(
                        content_type: _rt::String,
                        content: _rt::Vec<u8>,
                        ipfs_url: _rt::String,
                    ) -> Result<IpfsUrl, Error>;
                }
                #[doc(hidden)]
                macro_rules! __export_wavs_ipfs_ipfs_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "wavs:ipfs/ipfs#upload-json-to-ipfs"] unsafe extern "C" fn
                        export_upload_json_to_ipfs(arg0 : * mut u8, arg1 : usize, arg2 :
                        * mut u8, arg3 : usize,) -> * mut u8 { $($path_to_types)*::
                        _export_upload_json_to_ipfs_cabi::<$ty > (arg0, arg1, arg2, arg3)
                        } #[export_name = "cabi_post_wavs:ipfs/ipfs#upload-json-to-ipfs"]
                        unsafe extern "C" fn _post_return_upload_json_to_ipfs(arg0 : *
                        mut u8,) { $($path_to_types)*::
                        __post_return_upload_json_to_ipfs::<$ty > (arg0) } #[export_name
                        = "wavs:ipfs/ipfs#upload-image-to-ipfs"] unsafe extern "C" fn
                        export_upload_image_to_ipfs(arg0 : * mut u8, arg1 : usize, arg2 :
                        * mut u8, arg3 : usize, arg4 : * mut u8, arg5 : usize,) -> * mut
                        u8 { $($path_to_types)*:: _export_upload_image_to_ipfs_cabi::<$ty
                        > (arg0, arg1, arg2, arg3, arg4, arg5) } #[export_name =
                        "cabi_post_wavs:ipfs/ipfs#upload-image-to-ipfs"] unsafe extern
                        "C" fn _post_return_upload_image_to_ipfs(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_upload_image_to_ipfs::<$ty >
                        (arg0) } #[export_name = "wavs:ipfs/ipfs#delete-file"] unsafe
                        extern "C" fn export_delete_file(arg0 : * mut u8, arg1 : usize,)
                        -> * mut u8 { $($path_to_types)*:: _export_delete_file_cabi::<$ty
                        > (arg0, arg1) } #[export_name =
                        "cabi_post_wavs:ipfs/ipfs#delete-file"] unsafe extern "C" fn
                        _post_return_delete_file(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_delete_file::<$ty > (arg0) } #[export_name =
                        "wavs:ipfs/ipfs#get-ipfs-url"] unsafe extern "C" fn
                        export_get_ipfs_url(arg0 : * mut u8, arg1 : usize, arg2 : i32,
                        arg3 : * mut u8, arg4 : usize,) -> * mut u8 {
                        $($path_to_types)*:: _export_get_ipfs_url_cabi::<$ty > (arg0,
                        arg1, arg2, arg3, arg4) } #[export_name =
                        "cabi_post_wavs:ipfs/ipfs#get-ipfs-url"] unsafe extern "C" fn
                        _post_return_get_ipfs_url(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_get_ipfs_url::<$ty > (arg0) }
                        #[export_name = "wavs:ipfs/ipfs#upload-nft-content"] unsafe
                        extern "C" fn export_upload_nft_content(arg0 : * mut u8, arg1 :
                        usize, arg2 : * mut u8, arg3 : usize, arg4 : * mut u8, arg5 :
                        usize,) -> * mut u8 { $($path_to_types)*::
                        _export_upload_nft_content_cabi::<$ty > (arg0, arg1, arg2, arg3,
                        arg4, arg5) } #[export_name =
                        "cabi_post_wavs:ipfs/ipfs#upload-nft-content"] unsafe extern "C"
                        fn _post_return_upload_nft_content(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_upload_nft_content::<$ty >
                        (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_wavs_ipfs_ipfs_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 12]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 12],
                );
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    extern crate alloc as alloc_crate;
    pub use alloc_crate::alloc;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_ipfs_world_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::wavs::ipfs::ipfs::__export_wavs_ipfs_ipfs_cabi!($ty with_types_in
        $($path_to_types_root)*:: exports::wavs::ipfs::ipfs);
    };
}
#[doc(inline)]
pub(crate) use __export_ipfs_world_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:wavs:ipfs:ipfs-world:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 552] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xa7\x03\x01A\x02\x01\
A\x06\x01B\x04\x01s\x04\0\x05error\x03\0\0\x01s\x04\0\x08ipfs-url\x03\0\x02\x03\0\
\x0fwavs:ipfs/types\x05\0\x02\x03\0\0\x08ipfs-url\x02\x03\0\0\x05error\x01B\x12\x02\
\x03\x02\x01\x01\x04\0\x08ipfs-url\x03\0\0\x02\x03\x02\x01\x02\x04\0\x05error\x03\
\0\x02\x01j\x01\x01\x01\x03\x01@\x02\x09json-datas\x08ipfs-urls\0\x04\x04\0\x13u\
pload-json-to-ipfs\x01\x05\x01p}\x01@\x03\x0aimage-data\x06\x08filenames\x08ipfs\
-urls\0\x04\x04\0\x14upload-image-to-ipfs\x01\x07\x01j\x01\x7f\x01\x03\x01@\x01\x09\
file-paths\0\x08\x04\0\x0bdelete-file\x01\x09\x01ks\x01@\x02\x03cids\x08filename\
\x0a\0\x01\x04\0\x0cget-ipfs-url\x01\x0b\x01@\x03\x0ccontent-types\x07content\x06\
\x08ipfs-urls\0\x04\x04\0\x12upload-nft-content\x01\x0c\x04\0\x0ewavs:ipfs/ipfs\x05\
\x03\x04\0\x14wavs:ipfs/ipfs-world\x04\0\x0b\x10\x01\0\x0aipfs-world\x03\0\0\0G\x09\
producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.220.0\x10wit-bindgen-rus\
t\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
