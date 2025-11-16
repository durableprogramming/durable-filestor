//! Key translator implementation
//!
//! This module provides functionality to translate keys according to
//! the provided configuration (prefix, suffix, case transformation, etc.)

use crate::{KeyTranslation, HashFunction};
use unicode_normalization::UnicodeNormalization;

/// Translates keys according to the provided configuration
pub struct KeyTranslator {
    config: KeyTranslation,
}

impl KeyTranslator {
    /// Create a new translator with the given configuration
    pub fn new(config: KeyTranslation) -> Self {
        Self { config }
    }

    /// Translate a key according to the configuration
    pub fn translate(&self, key: &str) -> String {
        let mut result = key.to_string();

        // Normalization options
        if self.config.normalize_unicode {
            result = result.nfc().collect();
        }

        if self.config.strip_whitespace {
            result = result.trim().to_string();
        }

        if let Some(replace) = &self.config.replace_spaces {
            result = result.replace(' ', replace);
        }

        // Case transformation
        if self.config.downcase {
            result = result.to_lowercase();
        } else if self.config.upcase {
            result = result.to_uppercase();
        }

        // Hashing - only if not already a hash
        if let Some(hash_func) = &self.config.hash_function {
            if !self.is_hash(&result, hash_func) {
                let hash = self.compute_hash(&result, hash_func);
                let prefix_len = self.config.hash_prefix_length.unwrap_or(hash.len());
                result = hash.chars().take(prefix_len).collect();
            }
        }

        // Partitioning - only if not already partitioned
        if self.config.partition {
            if !self.is_partitioned(&result) {
                result = self.apply_partitioning(&result);
            }
        }

        // Prefix and suffix - only if not already present
        if let Some(prefix) = &self.config.prefix {
            if !result.starts_with(prefix) {
                result = format!("{}{}", prefix, result);
            }
        }

        if let Some(suffix) = &self.config.suffix {
            if !result.ends_with(suffix) {
                result = format!("{}{}", result, suffix);
            }
        }

        if self.config.timestamp_suffix {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            result = format!("{}.{}", result, timestamp);
        }

        // Max length
        if let Some(max_len) = self.config.max_key_length {
            if result.len() > max_len {
                result = result.chars().take(max_len).collect();
            }
        }

        result
    }

    fn compute_hash(&self, input: &str, func: &HashFunction) -> String {
        match func {
            HashFunction::Md5 => {
                use md5;
                format!("{:x}", md5::compute(input))
            }
            HashFunction::Sha1 => {
                use sha1::{Sha1, Digest};
                let mut hasher = Sha1::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashFunction::Sha256 => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashFunction::Sha512 => {
                use sha2::{Sha512, Digest};
                let mut hasher = Sha512::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
            HashFunction::Blake2b => {
                use blake2::{Blake2b512, Digest};
                let mut hasher = Blake2b512::new();
                hasher.update(input);
                format!("{:x}", hasher.finalize())
            }
        }
    }

    fn apply_partitioning(&self, key: &str) -> String {
        let length = if self.config.partition_length == 0 { 4 } else { self.config.partition_length };
        let depth = if self.config.partition_depth == 0 { 2 } else { self.config.partition_depth };
        let separator = if self.config.partition_separator.is_empty() { "/" } else { &self.config.partition_separator };

        let mut result = String::new();
        let chars: Vec<char> = key.chars().collect();

        for level in 0..depth {
            if level > 0 {
                result.push_str(separator);
            }
            let start = level * length;
            let end = (start + length).min(chars.len());
            for &ch in &chars[start..end] {
                result.push(ch);
            }
            if end - start < length {
                break;
            }
        }

        if chars.len() > depth * length {
            result.push_str(separator);
            for &ch in &chars[depth * length..] {
                result.push(ch);
            }
        }

        result
    }

    fn is_hash(&self, input: &str, func: &HashFunction) -> bool {
        let expected_len = match func {
            HashFunction::Md5 => 32,
            HashFunction::Sha1 => 40,
            HashFunction::Sha256 => 64,
            HashFunction::Sha512 => 128,
            HashFunction::Blake2b => 128,
        };

        // If we have a prefix length configured, check that instead
        let check_len = self.config.hash_prefix_length.unwrap_or(expected_len);
        
        // Remove prefix and suffix to get the core part to check
        let mut check_input = input.to_string();
        
        // Remove prefix if configured
        if let Some(prefix) = &self.config.prefix {
            if let Some(without_prefix) = check_input.strip_prefix(prefix) {
                check_input = without_prefix.to_string();
            }
        }
        
        // Remove suffix if configured
        if let Some(suffix) = &self.config.suffix {
            if let Some(without_suffix) = check_input.strip_suffix(suffix) {
                check_input = without_suffix.to_string();
            }
        }

        // Check if it's partitioned and extract the original hash
        if self.config.partition {
            let separator = if self.config.partition_separator.is_empty() { "/" } else { &self.config.partition_separator };
            
            // Reconstruct the hash from partition
            let parts: Vec<&str> = check_input.split(separator).collect();
            if parts.len() >= 2 {
                check_input = parts.join("");
            }
        }
        
        check_input.len() == check_len && check_input.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn is_partitioned(&self, input: &str) -> bool {
        let length = if self.config.partition_length == 0 { 4 } else { self.config.partition_length };
        let depth = if self.config.partition_depth == 0 { 2 } else { self.config.partition_depth };
        let separator = if self.config.partition_separator.is_empty() { "/" } else { &self.config.partition_separator };

        // Remove prefix and suffix to get the core part to check
        let mut check_input = input;
        
        // Remove prefix if configured
        if let Some(prefix) = &self.config.prefix {
            if let Some(without_prefix) = check_input.strip_prefix(prefix) {
                check_input = without_prefix;
            }
        }
        
        // Remove suffix if configured
        if let Some(suffix) = &self.config.suffix {
            if let Some(without_suffix) = check_input.strip_suffix(suffix) {
                check_input = without_suffix;
            }
        }

        // Check if the remaining input matches the expected partition pattern
        let parts: Vec<&str> = check_input.split(separator).collect();
        
        if parts.len() < depth + 1 {
            return false;
        }

        // Check if the first `depth` parts have the correct length
        for i in 0..depth {
            if parts[i].len() != length {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downcase() {
        let config = KeyTranslation {
            downcase: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        assert_eq!(translator.translate("My File.txt"), "my file.txt");
    }

    #[test]
    fn test_prefix_suffix() {
        let config = KeyTranslation {
            prefix: Some("files/".to_string()),
            suffix: Some(".dat".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        assert_eq!(translator.translate("document"), "files/document.dat");
    }

    #[test]
    fn test_partitioning() {
        let config = KeyTranslation {
            partition: true,
            partition_length: 2,
            partition_depth: 2,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        assert_eq!(translator.translate("abcdefg"), "ab/cd/efg");
    }

    #[test]
    fn test_idempotent_downcase() {
        let config = KeyTranslation {
            downcase: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("My File.txt");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Downcase should be idempotent");
    }

    #[test]
    fn test_idempotent_upcase() {
        let config = KeyTranslation {
            upcase: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("My File.txt");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Upcase should be idempotent");
    }

    #[test]
    fn test_idempotent_unicode_normalization() {
        let config = KeyTranslation {
            normalize_unicode: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("café");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Unicode normalization should be idempotent");
    }

    #[test]
    fn test_idempotent_strip_whitespace() {
        let config = KeyTranslation {
            strip_whitespace: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("  hello world  ");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Strip whitespace should be idempotent");
    }

    #[test]
    fn test_idempotent_replace_spaces() {
        let config = KeyTranslation {
            replace_spaces: Some("_".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello world");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Replace spaces should be idempotent");
    }

    #[test]
    fn test_idempotent_max_length() {
        let config = KeyTranslation {
            max_key_length: Some(5),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello world");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Max length should be idempotent");
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_idempotent_hash_md5() {
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Md5),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello world");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "MD5 hash should be idempotent");
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_idempotent_hash_sha256() {
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Sha256),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello world");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "SHA256 hash should be idempotent");
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_idempotent_hash_with_prefix_length() {
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Sha256),
            hash_prefix_length: Some(16),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello world");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Hash with prefix length should be idempotent");
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_idempotent_prefix() {
        let config = KeyTranslation {
            prefix: Some("files/".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("document.txt");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Prefix should be idempotent");
    }

    #[test]
    fn test_idempotent_suffix() {
        let config = KeyTranslation {
            suffix: Some(".dat".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("document");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Suffix should be idempotent");
    }

    #[test]
    fn test_idempotent_partitioning() {
        let config = KeyTranslation {
            partition: true,
            partition_length: 2,
            partition_depth: 2,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("abcdefg");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Partitioning should be idempotent");
    }

    #[test]
    fn test_non_idempotent_timestamp_suffix() {
        let config = KeyTranslation {
            timestamp_suffix: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result1 = translator.translate("document");
        std::thread::sleep(std::time::Duration::from_millis(1001));
        let result2 = translator.translate("document");
        assert_ne!(result1, result2, "Timestamp suffix should NOT be idempotent by design");
    }

    #[test]
    fn test_idempotent_complex_transformation() {
        let config = KeyTranslation {
            downcase: true,
            strip_whitespace: true,
            replace_spaces: Some("_".to_string()),
            hash_function: Some(HashFunction::Sha256),
            hash_prefix_length: Some(16),
            prefix: Some("cache/".to_string()),
            suffix: Some(".tmp".to_string()),
            partition: true,
            partition_length: 2,
            partition_depth: 2,
            max_key_length: Some(30),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("  Hello World  ");
        let result2 = translator.translate(&result);
        assert_eq!(result, result2, "Complex transformation should be idempotent");
    }

    #[test]
    fn test_hash_detection_md5() {
        let input = "5d41402abc4b2a76b9719d911017c592"; // MD5 hash
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Md5),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate(input);
        assert_eq!(result, input, "MD5 hash should remain unchanged");
    }

    #[test]
    fn test_hash_detection_sha256() {
        let input = "2cf24dba4f21d4288cfc4c8e62bf5c3a4e7a4e3f8b89a4e7b8a5b4c8f0e9d8c7"; // SHA256 hash
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Sha256),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate(input);
        assert_eq!(result, input, "SHA256 hash should remain unchanged");
    }

    #[test]
    fn test_partition_detection() {
        let input = "ab/cd/efghij"; // Already partitioned
        let config = KeyTranslation {
            partition: true,
            partition_length: 2,
            partition_depth: 2,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate(input);
        assert_eq!(result, input, "Already partitioned path should remain unchanged");
    }

    #[test]
    fn test_prefix_detection() {
        let input = "cache/document.txt"; // Already has prefix
        let config = KeyTranslation {
            prefix: Some("cache/".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate(input);
        assert_eq!(result, input, "Key with existing prefix should remain unchanged");
    }

    #[test]
    fn test_suffix_detection() {
        let input = "document.dat"; // Already has suffix
        let config = KeyTranslation {
            suffix: Some(".dat".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate(input);
        assert_eq!(result, input, "Key with existing suffix should remain unchanged");
    }

    // Edge case tests for comprehensive coverage

    #[test]
    fn test_empty_string() {
        let config = KeyTranslation {
            downcase: true,
            prefix: Some("files/".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("");
        assert_eq!(result, "files/", "Empty string should get prefix");
    }

    #[test]
    fn test_unicode_edge_cases() {
        let config = KeyTranslation {
            normalize_unicode: true,
            downcase: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        // Test various unicode forms
        let result1 = translator.translate("café"); // NFC
        let result2 = translator.translate("café"); // NFD - should normalize to same
        assert_eq!(result1, result2, "Unicode normalization should handle different forms");

        // Test combining characters
        let result = translator.translate("e\u{0301}"); // e + combining acute
        assert_eq!(result, "é", "Combining characters should be normalized");
    }

    #[test]
    fn test_very_long_key() {
        let long_key = "a".repeat(10000);
        let config = KeyTranslation {
            max_key_length: Some(100),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate(&long_key);
        assert_eq!(result.len(), 100, "Very long key should be truncated");
        assert!(result.chars().all(|c| c == 'a'), "Truncated key should contain correct characters");
    }

    #[test]
    fn test_whitespace_edge_cases() {
        let config = KeyTranslation {
            strip_whitespace: true,
            replace_spaces: Some("_".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        // Test various whitespace characters - strip_whitespace removes leading/trailing, replace_spaces replaces internal spaces
        let result = translator.translate(" \t\n hello \r\n world \t ");
        assert_eq!(result, "hello_\r\n_world", "Leading/trailing whitespace removed, internal spaces replaced");

        // Test only whitespace
        let result = translator.translate("   \t\t  ");
        assert_eq!(result, "", "Only whitespace should become empty");
    }

    #[test]
    fn test_special_characters() {
        let config = KeyTranslation {
            replace_spaces: Some("_".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        let result = translator.translate("file with spaces & special chars: @#$%^&*()");
        assert_eq!(result, "file_with_spaces_&_special_chars:_@#$%^&*()", "Special characters should be preserved except spaces");
    }

    #[test]
    fn test_conflicting_transformations() {
        // Test downcase and upcase both enabled (should prioritize downcase)
        let config = KeyTranslation {
            downcase: true,
            upcase: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("MiXeD CaSe");
        assert_eq!(result, "mixed case", "Downcase should take precedence over upcase");
    }

    #[test]
    fn test_partitioning_edge_cases() {
        let config = KeyTranslation {
            partition: true,
            partition_length: 3,
            partition_depth: 2,
            partition_separator: "-".to_string(),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        // Short key
        let result = translator.translate("ab");
        assert_eq!(result, "ab", "Short key should not be partitioned");

        // Key shorter than partition segments
        let result = translator.translate("abcdefg");
        assert_eq!(result, "abc-def-g", "Key should be partitioned correctly");

        // Key with existing partitioning
        let result = translator.translate("abc-def-xyz");
        assert_eq!(result, "abc-def-xyz", "Already partitioned key should remain unchanged");
    }

    #[test]
    fn test_hash_edge_cases() {
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Md5),
            hash_prefix_length: Some(8),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        // Empty string hash
        let result = translator.translate("");
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()), "Hash should contain only hex digits");
        assert_eq!(result.len(), 8, "Hash prefix should be correct length");

        // Very long input
        let long_input = "x".repeat(1000000);
        let result = translator.translate(&long_input);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()), "Long input hash should be valid");
        assert_eq!(result.len(), 8, "Long input hash prefix should be correct length");
    }

    #[test]
    fn test_timestamp_suffix_uniqueness() {
        let config = KeyTranslation {
            timestamp_suffix: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        let result1 = translator.translate("test");
        std::thread::sleep(std::time::Duration::from_secs(2)); // Wait longer to ensure different seconds
        let result2 = translator.translate("test");

        assert_ne!(result1, result2, "Timestamp suffixes should be unique");
        assert!(result1.starts_with("test."), "Should start with original key");
        assert!(result2.starts_with("test."), "Should start with original key");
    }

    #[test]
    fn test_complex_transformation_order() {
        // Test that transformations happen in the correct order
        let config = KeyTranslation {
            strip_whitespace: true,
            replace_spaces: Some("_".to_string()),
            downcase: true,
            prefix: Some("files/".to_string()),
            suffix: Some(".txt".to_string()),
            hash_function: Some(HashFunction::Md5),
            hash_prefix_length: Some(8),
            max_key_length: Some(20), // Increased to accommodate suffix
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        let result = translator.translate("  Hello World  ");
        // Order: strip_whitespace -> replace_spaces -> downcase -> hash -> prefix -> suffix -> max_length
        // "hello_world" -> hash -> "files/" + hash_prefix + ".txt" -> truncate to 20 chars
        assert!(result.starts_with("files/"), "Should have prefix");
        assert!(result.contains(".txt"), "Should contain suffix before truncation");
        assert_eq!(result.len(), 18, "Should be truncated to max length");
    }

    #[test]
    fn test_invalid_configurations() {
        // Test partitioning with invalid parameters
        let config = KeyTranslation {
            partition: true,
            partition_length: 0, // Invalid - should use fallback 4
            partition_depth: 0,  // Invalid - should use fallback 2
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("abcdefg");
        // Should work with fallback values (length=4, depth=2)
        assert_eq!(result, "abcd/efg", "Invalid partition config should use fallback values");
    }

    #[test]
    fn test_boundary_conditions() {
        // Test max_key_length of 0
        let config = KeyTranslation {
            max_key_length: Some(0),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello");
        assert_eq!(result, "", "Max length 0 should result in empty string");

        // Test hash_prefix_length of 0
        let config = KeyTranslation {
            hash_function: Some(HashFunction::Md5),
            hash_prefix_length: Some(0),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello");
        assert_eq!(result, "", "Hash prefix length 0 should result in empty string");
    }

    #[test]
    fn test_all_hash_functions() {
        let long_input = "a".repeat(1000);
        let inputs = vec![
            ("", "empty"),
            ("hello", "simple"),
            ("hello world with spaces", "complex"),
            (long_input.as_str(), "long"),
        ];

        for hash_func in &[HashFunction::Md5, HashFunction::Sha1, HashFunction::Sha256, HashFunction::Sha512, HashFunction::Blake2b] {
            let config = KeyTranslation {
                hash_function: Some(hash_func.clone()),
                ..Default::default()
            };
            let translator = KeyTranslator::new(config);

            for (input, desc) in &inputs {
                let result = translator.translate(input);
                assert!(result.chars().all(|c| c.is_ascii_hexdigit()),
                       "Hash function {:?} with {} input should produce hex digits", hash_func, desc);

                // Check expected lengths
                let expected_len = match hash_func {
                    HashFunction::Md5 => 32,
                    HashFunction::Sha1 => 40,
                    HashFunction::Sha256 => 64,
                    HashFunction::Sha512 => 128,
                    HashFunction::Blake2b => 128,
                };
                assert_eq!(result.len(), expected_len,
                          "Hash function {:?} with {} input should have correct length", hash_func, desc);
            }
        }
    }

    #[test]
    fn test_unicode_normalization_comprehensive() {
        let config = KeyTranslation {
            normalize_unicode: true,
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        // Test various normalization scenarios
        let test_cases = vec![
            ("café", "café"), // Already NFC
            ("café", "café"), // NFD form
            ("ñ", "ñ"),       // Single character
            ("русский", "русский"), // Cyrillic
            ("العربية", "العربية"), // Arabic
        ];

        for (input, expected) in test_cases {
            let result = translator.translate(input);
            assert_eq!(result, expected, "Unicode normalization should work for: {}", input);
        }
    }

    #[test]
    fn test_replace_spaces_edge_cases() {
        let config = KeyTranslation {
            replace_spaces: Some("".to_string()), // Replace with empty string
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("hello world test");
        assert_eq!(result, "helloworldtest", "Spaces should be removed when replaced with empty string");

        let config = KeyTranslation {
            replace_spaces: Some("___".to_string()), // Replace with multiple characters
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("a b c");
        assert_eq!(result, "a___b___c", "Spaces should be replaced with multiple characters");
    }

    #[test]
    fn test_prefix_suffix_with_special_characters() {
        let config = KeyTranslation {
            prefix: Some("path/with/slashes/".to_string()),
            suffix: Some(".extension.with.dots".to_string()),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);
        let result = translator.translate("file");
        assert_eq!(result, "path/with/slashes/file.extension.with.dots", "Prefix and suffix with special characters should work");
    }

    #[test]
    fn test_max_length_with_unicode() {
        let config = KeyTranslation {
            max_key_length: Some(5),
            ..Default::default()
        };
        let translator = KeyTranslator::new(config);

        let result = translator.translate("café"); // 4 chars
        assert_eq!(result, "café", "Unicode string shorter than limit should be unchanged");

        let result = translator.translate("caféau"); // 6 chars
        assert_eq!(result.chars().count(), 5, "Unicode string should be truncated by character count, not byte count");
    }
}