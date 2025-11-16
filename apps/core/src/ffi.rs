//! FFI interface for Ruby bindings
//!
//! This module provides C-compatible FFI bindings for the durable-filestor library
//! to be used by Ruby via FFI or C extension.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use durable_filestor_shared::{KeyTranslator, KeyTranslation, HashFunction};

/// Opaque handle to a KeyTranslator instance
#[repr(C)]
pub struct DfsKeyTranslator {
    translator: Box<KeyTranslator>,
}

/// Hash function enum for FFI
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum DfsHashFunction {
    Md5 = 0,
    Sha1 = 1,
    Sha256 = 2,
    Sha512 = 3,
    Blake2b = 4,
}

impl From<DfsHashFunction> for HashFunction {
    fn from(f: DfsHashFunction) -> Self {
        match f {
            DfsHashFunction::Md5 => HashFunction::Md5,
            DfsHashFunction::Sha1 => HashFunction::Sha1,
            DfsHashFunction::Sha256 => HashFunction::Sha256,
            DfsHashFunction::Sha512 => HashFunction::Sha512,
            DfsHashFunction::Blake2b => HashFunction::Blake2b,
        }
    }
}

/// Configuration structure for FFI
#[repr(C)]
pub struct DfsKeyTranslationConfig {
    pub downcase: bool,
    pub upcase: bool,
    pub hash_function: i32, // -1 for none, otherwise DfsHashFunction
    pub hash_prefix_length: i32, // -1 for none
    pub partition: bool,
    pub partition_length: usize,
    pub partition_depth: usize,
    pub partition_separator: *const c_char,
    pub prefix: *const c_char, // NULL for none
    pub suffix: *const c_char, // NULL for none
    pub timestamp_suffix: bool,
    pub normalize_unicode: bool,
    pub strip_whitespace: bool,
    pub replace_spaces: *const c_char, // NULL for none
    pub max_key_length: i32, // -1 for none
}

/// Create a new KeyTranslator instance
///
/// # Safety
/// - `config` must be a valid pointer to a DfsKeyTranslationConfig
/// - All string pointers in config must be valid UTF-8 or NULL
/// - Returns NULL on error
#[no_mangle]
pub unsafe extern "C" fn dfs_key_translator_new(config: *const DfsKeyTranslationConfig) -> *mut DfsKeyTranslator {
    if config.is_null() {
        return ptr::null_mut();
    }

    let config_ref = &*config;

    // Convert FFI config to Rust config
    let mut key_translation = KeyTranslation::default();

    key_translation.downcase = config_ref.downcase;
    key_translation.upcase = config_ref.upcase;

    if config_ref.hash_function >= 0 {
        let hash_func: DfsHashFunction = match config_ref.hash_function {
            0 => DfsHashFunction::Md5,
            1 => DfsHashFunction::Sha1,
            2 => DfsHashFunction::Sha256,
            3 => DfsHashFunction::Sha512,
            4 => DfsHashFunction::Blake2b,
            _ => return ptr::null_mut(),
        };
        key_translation.hash_function = Some(hash_func.into());
    }

    if config_ref.hash_prefix_length >= 0 {
        key_translation.hash_prefix_length = Some(config_ref.hash_prefix_length as usize);
    }

    key_translation.partition = config_ref.partition;
    key_translation.partition_length = config_ref.partition_length;
    key_translation.partition_depth = config_ref.partition_depth;

    if !config_ref.partition_separator.is_null() {
        if let Ok(s) = CStr::from_ptr(config_ref.partition_separator).to_str() {
            key_translation.partition_separator = s.to_string();
        }
    }

    if !config_ref.prefix.is_null() {
        if let Ok(s) = CStr::from_ptr(config_ref.prefix).to_str() {
            key_translation.prefix = Some(s.to_string());
        }
    }

    if !config_ref.suffix.is_null() {
        if let Ok(s) = CStr::from_ptr(config_ref.suffix).to_str() {
            key_translation.suffix = Some(s.to_string());
        }
    }

    key_translation.timestamp_suffix = config_ref.timestamp_suffix;
    key_translation.normalize_unicode = config_ref.normalize_unicode;
    key_translation.strip_whitespace = config_ref.strip_whitespace;

    if !config_ref.replace_spaces.is_null() {
        if let Ok(s) = CStr::from_ptr(config_ref.replace_spaces).to_str() {
            key_translation.replace_spaces = Some(s.to_string());
        }
    }

    if config_ref.max_key_length >= 0 {
        key_translation.max_key_length = Some(config_ref.max_key_length as usize);
    }

    let translator = Box::new(KeyTranslator::new(key_translation));
    Box::into_raw(Box::new(DfsKeyTranslator { translator }))
}

/// Translate a key using the KeyTranslator
///
/// # Safety
/// - `translator` must be a valid pointer from dfs_key_translator_new
/// - `key` must be a valid UTF-8 C string
/// - The returned string must be freed with dfs_string_free
/// - Returns NULL on error
#[no_mangle]
pub unsafe extern "C" fn dfs_key_translator_translate(
    translator: *const DfsKeyTranslator,
    key: *const c_char,
) -> *mut c_char {
    if translator.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    let translator_ref = &*translator;

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = translator_ref.translator.translate(key_str);

    match CString::new(result) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a KeyTranslator instance
///
/// # Safety
/// - `translator` must be a valid pointer from dfs_key_translator_new
/// - Must only be called once per translator
#[no_mangle]
pub unsafe extern "C" fn dfs_key_translator_free(translator: *mut DfsKeyTranslator) {
    if !translator.is_null() {
        let _ = Box::from_raw(translator);
    }
}

/// Free a string returned by FFI functions
///
/// # Safety
/// - `s` must be a valid pointer from an FFI function that returns *mut c_char
/// - Must only be called once per string
#[no_mangle]
pub unsafe extern "C" fn dfs_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Get version information
///
/// # Safety
/// - The returned string must be freed with dfs_string_free
#[no_mangle]
pub unsafe extern "C" fn dfs_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    match CString::new(version) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_basic_translation() {
        unsafe {
            let config = DfsKeyTranslationConfig {
                downcase: true,
                upcase: false,
                hash_function: -1,
                hash_prefix_length: -1,
                partition: false,
                partition_length: 0,
                partition_depth: 0,
                partition_separator: ptr::null(),
                prefix: ptr::null(),
                suffix: ptr::null(),
                timestamp_suffix: false,
                normalize_unicode: false,
                strip_whitespace: false,
                replace_spaces: ptr::null(),
                max_key_length: -1,
            };

            let translator = dfs_key_translator_new(&config);
            assert!(!translator.is_null());

            let key = CString::new("HELLO").unwrap();
            let result = dfs_key_translator_translate(translator, key.as_ptr());
            assert!(!result.is_null());

            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(result_str, "hello");

            dfs_string_free(result);
            dfs_key_translator_free(translator);
        }
    }

    #[test]
    fn test_ffi_version() {
        unsafe {
            let version = dfs_version();
            assert!(!version.is_null());

            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert!(!version_str.is_empty());

            dfs_string_free(version);
        }
    }
}
