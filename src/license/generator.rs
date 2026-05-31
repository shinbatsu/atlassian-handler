use chrono::{Datelike, DateTime, FixedOffset, Timelike, Utc};
use std::collections::HashMap;

use base64::Engine;

use super::product_type::LicenseType;
use super::edition::LicenseEdition;
use super::encoder;

fn java_hash_hashcode(code: u64) -> u64 {
    let h = code as u64;
    h ^ (h >> 16)
}

fn java_string_hashcode(s: &str) -> u64 {
    let mut h: i32 = 0;
    for c in s.chars() {
        h = (31i32).wrapping_mul(h).wrapping_add(c as i32);
    }
    h as u64 & 0xFFFFFFFF
}

fn java_hashmap_bucket(key: &str, capacity: u32) -> u32 {
    let h = java_string_hashcode(key);
    let supp = java_hash_hashcode(h);
    supp as u32 & (capacity - 1)
}

fn compute_output_order(keys: &[String]) -> Vec<String> {
    let _n = keys.len();
    
    let mut cap = 16u32;
    let threshold = 12;
    
    let mut bucket_map: HashMap<u32, Vec<usize>> = HashMap::new();
    
    for (i, key) in keys.iter().enumerate() {
        if i >= threshold && cap == 16 {
            cap = 32;
            bucket_map.clear();
            for (j, prev_key) in keys[..=i].iter().enumerate() {
                let rehash_cap = 32u32;
                let idx = java_hashmap_bucket(prev_key, rehash_cap);
                bucket_map.entry(idx).or_insert_with(Vec::new).push(j);
            }
        } else {
            let idx = java_hashmap_bucket(key, cap);
            bucket_map.entry(idx).or_insert_with(Vec::new).push(i);
        }
    }
    
    let mut result = Vec::new();
    for b in 0..cap {
        if let Some(indices) = bucket_map.get(&b) {
            for &i in indices {
                result.push(keys[i].clone());
            }
        }
    }
    result
}

pub struct LicenseData {
    pub contact_name: String,
    pub contact_email: String,
    pub server_id: String,
    pub organisation: String,
    pub data_center: bool,
    pub product_name: String,

    data: HashMap<String, String>,
    insertion_order: Vec<String>,
}

impl LicenseData {
    pub fn new(
        contact_name: String,
        contact_email: String,
        server_id: String,
        organisation: String,
        data_center: bool,
        product_name: String,
    ) -> Self {
        let mut ld = LicenseData {
            contact_name,
            contact_email,
            server_id,
            organisation,
            data_center,
            product_name,
            data: HashMap::new(),
            insertion_order: Vec::new(),
        };
        ld.init();
        ld
    }

    fn insert(&mut self, key: String, value: String) {
        self.data.insert(key.clone(), value);
        if !self.insertion_order.contains(&key) {
            self.insertion_order.push(key);
        }
    }

    fn init(&mut self) {
        let now = Utc::now();
        let expiry_millis: i64 = 3771590399000;
        let expiry_ts = expiry_millis / 1000;
        let expiry_date = chrono::DateTime::from_timestamp(expiry_ts, 0)
            .unwrap_or(now);

        let license_id = format!("L{}", now.timestamp_millis());
        let product = self.product_name.clone();

        let product_key = |prop: &str| -> String { format!("{}.{}", product, prop) };

        self.insert(product_key("active"), "true".to_string());
        self.insert("PurchaseDate".to_string(), now.format("%Y-%m-%d").to_string());
        self.insert("LicenseExpiryDate".to_string(), expiry_date.format("%Y-%m-%d").to_string());
        self.insert("MaintenanceExpiryDate".to_string(), expiry_date.format("%Y-%m-%d").to_string());
        self.insert(product_key("NumberOfUsers"), "-1".to_string());
        self.insert(product_key("Starter"), "false".to_string());
        self.insert("SEN".to_string(), format!("SEN-{}", license_id));
        self.insert("LicenseID".to_string(), format!("LIDSEN-{}", license_id));
        self.insert("CreationDate".to_string(), now.format("%Y-%m-%d").to_string());
        self.insert(product_key("LicenseTypeName"), LicenseType::Commercial.to_string());
        self.insert("Description".to_string(), "Unlimited license by https://zhile.io".to_string());
        self.insert("Evaluation".to_string(), "false".to_string());
        self.insert("ContactName".to_string(), self.contact_name.clone());
        self.insert("ContactEMail".to_string(), self.contact_email.clone());
        self.insert("ServerID".to_string(), self.server_id.clone());
        self.insert("Organisation".to_string(), self.organisation.clone());

        if product == "jira.product.jira-software" {
            self.insert("greenhopper.active".to_string(), "true".to_string());
            self.insert("jira.active".to_string(), "true".to_string());
            let users_count = self.data.get(&product_key("NumberOfUsers")).cloned();
            if let Some(ref users) = users_count {
                self.insert("jira.NumberOfUsers".to_string(), users.clone());
                self.insert("NumberOfUsers".to_string(), users.clone());
            }
            self.insert("greenhopper.LicenseEdition".to_string(), LicenseEdition::Unlimited.to_string());
            self.insert("jira.LicenseEdition".to_string(), LicenseEdition::Unlimited.to_string());
            self.insert("greenhopper.LicenseTypeName".to_string(), LicenseType::Commercial.to_string());
            self.insert("jira.LicenseTypeName".to_string(), LicenseType::Commercial.to_string());
            self.insert("LicenseTypeName".to_string(), LicenseType::Commercial.to_string());
            self.insert("greenhopper.enterprise".to_string(), "true".to_string());
        }

        if self.data_center {
            self.insert(product_key("DataCenter"), "true".to_string());
            self.insert("Subscription".to_string(), "true".to_string());
            if product == "jira.product.jira-software" {
                self.insert("jira.DataCenter".to_string(), "true".to_string());
            }
        }

        self.insert("licenseVersion".to_string(), "2".to_string());
        self.insert("keyVersion".to_string(), "1600708331".to_string());
    }

    fn generate_license_hash(&mut self) {
        if self.data.contains_key("licenseHash") {
            self.data.remove("licenseHash");
            if let Some(pos) = self.insertion_order.iter().position(|k| k == "licenseHash") {
                self.insertion_order.remove(pos);
            }
        }

        let mut sorted_keys: Vec<&String> = self.data.keys().collect();
        sorted_keys.sort();

        let mut sb = String::new();
        for key in sorted_keys {
            let value = self.data.get(key).unwrap();
            if value.is_empty() {
                continue;
            }
            sb.push_str(&escape(key, true));
            sb.push('=');
            sb.push_str(&escape(value, false));
            sb.push('\n');
        }

        let signature = encoder::sign_data(sb.as_bytes());
        let hash_b64 = base64::engine::general_purpose::STANDARD.encode(&signature);
        
        self.insert("licenseHash".to_string(), hash_b64);
    }

    fn format_java_date(date: DateTime<Utc>) -> String {
        let msk_offset = FixedOffset::east_opt(3 * 3600).unwrap();
        let local = date.with_timezone(&msk_offset);
        
        let weekday = match local.weekday() {
            chrono::Weekday::Mon => "Mon",
            chrono::Weekday::Tue => "Tue",
            chrono::Weekday::Wed => "Wed",
            chrono::Weekday::Thu => "Thu",
            chrono::Weekday::Fri => "Fri",
            chrono::Weekday::Sat => "Sat",
            chrono::Weekday::Sun => "Sun",
        };
        
        let month = match local.month() {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "???",
        };
        
        format!(
            "{} {} {:02} {:02}:{:02}:{:02} MSK {}",
            weekday,
            month,
            local.day(),
            local.hour(),
            local.minute(),
            local.second(),
            local.year()
        )
    }

    pub fn generate(&mut self) -> Result<String, String> {
        self.generate_license_hash();

        let mut sb = String::new();
        sb.push('#');
        sb.push_str(&Self::format_java_date(Utc::now()));

        let ordered_keys = compute_output_order(&self.insertion_order);
        for key in &ordered_keys {
            let value = self.data.get(key).unwrap();
            if value.is_empty() {
                continue;
            }
            sb.push('\n');
            sb.push_str(key);
            sb.push('=');
            sb.push_str(value);
        }

        encoder::encode_license(&sb)
    }
}

fn escape(str: &str, is_key: bool) -> String {
    let mut sb = String::new();
    for (i, c) in str.chars().enumerate() {
        match c {
            '\t' => sb.push_str("\\t"),
            '\n' => sb.push_str("\\n"),
            '\u{000c}' => sb.push_str("\\f"),
            '\r' => sb.push_str("\\r"),
            ' ' => {
                if i == 0 || is_key {
                    sb.push('\\');
                }
                sb.push(' ');
            }
            '\\' => sb.push_str("\\\\"),
            _ => {
                if c == '=' || c == ':' || c == '\t' || c == '\r' || c == '\n' || c == '\u{000c}' || c == '#' || c == '!' {
                    sb.push('\\');
                }
                sb.push(c);
            }
        }
    }
    sb
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_escape_plain_string() {
        assert_eq!(escape("hello", false), "hello", "Plain string should not be escaped");
        assert_eq!(escape("hello", true), "hello", "Plain string should not be escaped even if key");
    }

    #[test]
    fn test_escape_tab() {
        assert_eq!(escape("hello\tworld", false), "hello\\tworld", "Tab should be escaped");
    }

    #[test]
    fn test_escape_newline() {
        assert_eq!(escape("hello\nworld", false), "hello\\nworld", "Newline should be escaped");
    }

    #[test]
    fn test_escape_formfeed() {
        assert_eq!(escape("hello\u{000c}world", false), "hello\\fworld", "Form feed should be escaped");
    }

    #[test]
    fn test_escape_carriage_return() {
        assert_eq!(escape("hello\rworld", false), "hello\\rworld", "CR should be escaped");
    }

    #[test]
    fn test_escape_backslash() {
        assert_eq!(escape("hello\\world", false), "hello\\\\world", "Backslash should be escaped");
    }

    #[test]
    fn test_escape_leading_space() {
        assert_eq!(escape(" hello", false), "\\ hello", "Leading space should be escaped");
    }

    #[test]
    fn test_escape_trailing_space() {
        assert_eq!(escape("hello ", false), "hello ", "Trailing space should NOT be escaped (value)");
    }

    #[test]
    fn test_escape_key_any_space() {
        assert_eq!(escape("hello world", true), "hello\\ world", "Space in key should be escaped");
    }

    #[test]
    fn test_escape_value_mid_space() {
        assert_eq!(escape("hello world", false), "hello world", "Mid space in value should NOT be escaped");
    }

    #[test]
    fn test_escape_special_chars_in_value() {
        assert_eq!(escape("a=b", false), "a\\=b", "'=' in value should be escaped");
        assert_eq!(escape("a:b", false), "a\\:b", "':' in value should be escaped");
        assert_eq!(escape("a#b", false), "a\\#b", "'#' in value should be escaped");
        assert_eq!(escape("a!b", false), "a\\!b", "'!' in value should be escaped");
    }

    #[test]
    fn test_escape_special_chars_in_key() {
        assert_eq!(escape("a=b", true), "a\\=b", "'=' in key should be escaped");
        assert_eq!(escape("a:b", true), "a\\:b", "':' in key should be escaped");
    }

    #[test]
    fn test_escape_complex() {
        let input = "key with spaces";
        assert_eq!(escape(input, true), "key\\ with\\ spaces", "All spaces in key should be escaped");
    }

    #[test]
    fn test_escape_already_escaped() {
        assert_eq!(escape("a\\=b", false), "a\\\\\\=b", "Backslash before =: \\ becomes \\\\ and = becomes \\= independently");
    }


    #[test]
    fn test_license_data_fields_crowd() {
        let mut ld = LicenseData::new(
            "TestUser".to_string(),
            "test@example.com".to_string(),
            "ABCD-1234-EFGH-5678".to_string(),
            "TestOrg".to_string(),
            false,
            "crowd".to_string(),
        );

        ld.generate_license_hash();

        assert_eq!(ld.data.get("ContactName").unwrap(), "TestUser");
        assert_eq!(ld.data.get("ContactEMail").unwrap(), "test@example.com");
        assert_eq!(ld.data.get("ServerID").unwrap(), "ABCD-1234-EFGH-5678");
        assert_eq!(ld.data.get("Organisation").unwrap(), "TestOrg");
        assert_eq!(ld.data.get("Evaluation").unwrap(), "false");
        assert_eq!(ld.data.get("Description").unwrap(), "Unlimited license by https://zhile.io");
        assert_eq!(ld.data.get("licenseVersion").unwrap(), "2");
        assert_eq!(ld.data.get("keyVersion").unwrap(), "1600708331");

        assert_eq!(ld.data.get("crowd.active").unwrap(), "true");
        assert_eq!(ld.data.get("crowd.NumberOfUsers").unwrap(), "-1");
        assert_eq!(ld.data.get("crowd.Starter").unwrap(), "false");
        assert_eq!(ld.data.get("crowd.LicenseTypeName").unwrap(), "COMMERCIAL");

        assert!(ld.data.get("crowd.DataCenter").is_none(), "DataCenter should not be set for non-DC");
        assert!(ld.data.get("Subscription").is_none(), "Subscription should not be set for non-DC");

        assert!(ld.data.get("SEN").unwrap().starts_with("SEN-L"));
        assert!(ld.data.get("LicenseID").unwrap().starts_with("LIDSEN-L"));

        assert!(ld.data.get("PurchaseDate").unwrap().len() == 10, "PurchaseDate should be yyyy-MM-dd");
        let expiry = ld.data.get("LicenseExpiryDate").unwrap();
        assert_eq!(expiry, "2089-07-07", "LicenseExpiryDate should match Java reference value");
        assert!(ld.data.get("CreationDate").unwrap().len() == 10, "CreationDate should be yyyy-MM-dd");

        assert!(ld.data.contains_key("licenseHash"), "licenseHash must be present");
        assert!(!ld.data.get("licenseHash").unwrap().is_empty(), "licenseHash must not be empty");
    }

    #[test]
    fn test_license_data_data_center() {
        let mut ld = LicenseData::new(
            "DCUser".to_string(),
            "dc@example.com".to_string(),
            "DCID-1234".to_string(),
            "DC Corp".to_string(),
            true,
            "conf".to_string(),
        );

        ld.generate_license_hash();

        assert_eq!(ld.data.get("conf.DataCenter").unwrap(), "true", "DataCenter should be true");
        assert_eq!(ld.data.get("Subscription").unwrap(), "true", "Subscription should be true");
    }

    #[test]
    fn test_license_data_jira_software() {
        let mut ld = LicenseData::new(
            "JiraUser".to_string(),
            "jira@example.com".to_string(),
            "JIRA-5678".to_string(),
            "JiraCorp".to_string(),
            false,
            "jira.product.jira-software".to_string(),
        );

        ld.generate_license_hash();

        assert_eq!(ld.data.get("greenhopper.active").unwrap(), "true");
        assert_eq!(ld.data.get("jira.active").unwrap(), "true");
        assert_eq!(ld.data.get("jira.NumberOfUsers").unwrap(), "-1");
        assert_eq!(ld.data.get("greenhopper.LicenseEdition").unwrap(), "UNLIMITED");
        assert_eq!(ld.data.get("jira.LicenseEdition").unwrap(), "UNLIMITED");
        assert_eq!(ld.data.get("greenhopper.LicenseTypeName").unwrap(), "COMMERCIAL");
        assert_eq!(ld.data.get("jira.LicenseTypeName").unwrap(), "COMMERCIAL");
        assert_eq!(ld.data.get("greenhopper.enterprise").unwrap(), "true");

        assert_eq!(ld.data.get("NumberOfUsers").unwrap(), "-1",
            "Java JIRASoftware adds bare NumberOfUsers");
        assert_eq!(ld.data.get("LicenseTypeName").unwrap(), "COMMERCIAL",
            "Java JIRASoftware adds bare LicenseTypeName");
    }

    #[test]
    fn test_license_data_jira_software_data_center() {
        let mut ld = LicenseData::new(
            "JiraDC".to_string(),
            "jiradc@example.com".to_string(),
            "JDC-1234".to_string(),
            "JiraDC Inc".to_string(),
            true,
            "jira.product.jira-software".to_string(),
        );

        ld.generate_license_hash();

        assert_eq!(ld.data.get("jira.product.jira-software.DataCenter").unwrap(), "true",
            "Product DataCenter should be true");
        assert_eq!(ld.data.get("jira.DataCenter").unwrap(), "true",
            "Java JIRASoftware adds jira.DataCenter for DC");
        assert_eq!(ld.data.get("Subscription").unwrap(), "true", "Subscription should be true");
    }

    #[test]
    fn test_license_hash_is_dsa_signature_not_sha256() {
        let mut ld = LicenseData::new(
            "HashTest".to_string(),
            "hash@test.com".to_string(),
            "HASH-0001".to_string(),
            "HashCorp".to_string(),
            false,
            "crowd".to_string(),
        );

        ld.generate_license_hash();
        let hash_val = ld.data.get("licenseHash").unwrap();

        let decoded = base64::engine::general_purpose::STANDARD.decode(hash_val)
            .expect("licenseHash should be valid base64");

        assert_eq!(decoded[0], 0x30,
            "licenseHash should be a DER SEQUENCE (DSA signature), not a SHA-256 hash");

        assert!(decoded.len() >= 40, "DSA signature should be at least 40 bytes, got {}", decoded.len());
        assert!(decoded.len() <= 48, "DSA signature should be at most 48 bytes, got {}", decoded.len());
    }

    #[test]
    fn test_generate_license_hash_removes_previous() {
        let mut ld = LicenseData::new(
            "Test".to_string(),
            "test@test.com".to_string(),
            "SID-1234".to_string(),
            "Org".to_string(),
            false,
            "crowd".to_string(),
        );

        let count_before = ld.data.len();
        assert!(!ld.data.contains_key("licenseHash"), "licenseHash should not exist before generate");

        ld.generate_license_hash();
        let hash1 = ld.data.get("licenseHash").unwrap().clone();
        let count_after_first = ld.data.len();
        assert_eq!(count_after_first, count_before + 1, "licenseHash should be added");
        assert!(!hash1.is_empty(), "licenseHash should not be empty");

        ld.generate_license_hash();
        let hash2 = ld.data.get("licenseHash").unwrap().clone();
        let count_after_second = ld.data.len();
        
        assert_eq!(count_after_second, count_after_first, "Hash count should stay the same");
        assert!(!hash2.is_empty(), "Re-generated hash should not be empty");
    }

    #[test]
    fn test_generate_includes_hash_in_output() {
        let mut ld = LicenseData::new(
            "GenTest".to_string(),
            "gen@test.com".to_string(),
            "GEN-1234".to_string(),
            "GenCorp".to_string(),
            false,
            "crowd".to_string(),
        );

        ld.generate_license_hash();
        assert!(ld.data.contains_key("licenseHash"), "licenseHash must be in data before encoding");
        assert!(!ld.data.get("licenseHash").unwrap().is_empty(), "licenseHash must not be empty");

        let result = ld.generate().expect("Should generate license");

        assert!(result.contains("X02"), "License should contain X02 suffix");

        assert!(result.contains('\n'), "License should have line breaks");

        assert_eq!(ld.data.get("ContactName").unwrap(), "GenTest");
        assert_eq!(ld.data.get("ContactEMail").unwrap(), "gen@test.com");
        assert_eq!(ld.data.get("ServerID").unwrap(), "GEN-1234");
        assert_eq!(ld.data.get("Organisation").unwrap(), "GenCorp");
    }

    #[test]
    fn test_license_text_structure() {
        let mut ld = LicenseData::new(
            "StructTest".to_string(),
            "struct@test.com".to_string(),
            "STR-9999".to_string(),
            "StructCorp".to_string(),
            false,
            "crowd".to_string(),
        );

        ld.generate_license_hash();

        let ordered_keys = compute_output_order(&ld.insertion_order);

        let mut sb = String::new();
        sb.push('#');
        sb.push_str(&LicenseData::format_java_date(Utc::now()));
        for key in &ordered_keys {
            let value = ld.data.get(key).unwrap();
            if value.is_empty() {
                continue;
            }
            sb.push('\n');
            sb.push_str(key);
            sb.push('=');
            sb.push_str(value);
        }

        assert!(sb.starts_with('#'), "Must start with #");
        assert!(sb.contains("\n"), "Must contain at least one newline");

        for line in sb.lines().skip(1) {
            assert!(line.contains('='), "Each line (except first) should be key=value: {}", line);
        }

        assert!(sb.contains("\nlicenseVersion=2"), "Must contain licenseVersion=2");
        assert!(sb.contains("\nkeyVersion=1600708331"), "Must contain keyVersion=1600708331");
    }

    #[test]
    fn test_product_types_match_java() {
        let test_cases = vec![
            ("crowd", "crowd"),
            ("conf", "conf"),
            ("bitbucket", "bitbucket"),
            ("bamboo", "bamboo"),
            ("fisheye", "fisheye"),
            ("crucible", "crucible"),
            ("jsm", "jsm"),
            ("jc", "jc"),
            ("jsd", "jsd"),
            ("questions", "questions"),
            ("capture", "capture"),
            ("training", "training"),
            ("portfolio", "portfolio"),
            ("tc", "tc"),
        ];

        for (key, expected_name) in test_cases {
            let cfg = super::super::products::get_product_config(key);
            assert!(cfg.is_some(), "Product {} should have config", key);
            assert_eq!(cfg.unwrap().name, expected_name, "Product {} name mismatch", key);
        }
    }

    #[test]
    fn test_jira_software_product_name() {
        let cfg = super::super::products::get_product_config("jira")
            .or_else(|| super::super::products::get_product_config("jira-software"));
        assert!(cfg.is_some(), "JIRA should have a config");
        assert_eq!(cfg.unwrap().name, "jira.product.jira-software",
            "JIRA product name should match Java reference");
    }

    #[test]
    fn test_java_date_format() {
        let now = Utc::now();
        let formatted = LicenseData::format_java_date(now);
        
        let parts: Vec<&str> = formatted.split(' ').collect();
        assert_eq!(parts.len(), 6, "Java date format should have 6 parts: {}", formatted);
        
        assert!(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].contains(&parts[0]),
            "First part should be weekday abbreviation, got '{}'", parts[0]);
        
        assert!(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].contains(&parts[1]),
            "Second part should be month abbreviation, got '{}'", parts[1]);
        
        assert_eq!(parts[2].len(), 2, "Day should be 2 digits: {}", parts[2]);
        
        assert_eq!(parts[3].len(), 8, "Time should be HH:mm:ss: {}", parts[3]);
        assert_eq!(&parts[3][2..3], ":", "Time should have colon after hours");
        assert_eq!(&parts[3][5..6], ":", "Time should have colon after minutes");
        
        assert_eq!(parts[4], "MSK", "Timezone should be MSK (Moscow Standard Time)");
        
        assert_eq!(parts[5].len(), 4, "Year should be 4 digits: {}", parts[5]);
    }

    #[test]
    fn test_generate_starts_with_hash_and_date() {
        let mut ld = LicenseData::new(
            "DateTest".to_string(),
            "date@test.com".to_string(),
            "DATE-0001".to_string(),
            "DateCorp".to_string(),
            false,
            "crowd".to_string(),
        );
        
        ld.generate_license_hash();
        
        let mut sb = String::new();
        sb.push('#');
        sb.push_str(&LicenseData::format_java_date(Utc::now()));
        
        let first_line = sb.lines().next().unwrap();
        assert!(first_line.starts_with('#'), "First line must start with #");
        let date_part = &first_line[1..];
        assert!(date_part.contains(' '), "Date should contain spaces");
        assert!(date_part.contains(" MSK "), "Date should contain MSK timezone");
    }

    #[test]
    fn test_hash_input_uses_alphabetical_order() {
        let mut ld = LicenseData::new(
            "OrderTest".to_string(),
            "order@test.com".to_string(),
            "ORD-1234".to_string(),
            "OrderCorp".to_string(),
            false,
            "crowd".to_string(),
        );

        ld.generate_license_hash();
        
        let mut sorted_keys: Vec<&String> = ld.data.keys().collect();
        sorted_keys.sort();
        
        let mut hash_input = String::new();
        for key in sorted_keys {
            let value = ld.data.get(key).unwrap();
            if value.is_empty() {
                continue;
            }
            hash_input.push_str(&escape(key, true));
            hash_input.push('=');
            hash_input.push_str(&escape(value, false));
            hash_input.push('\n');
        }

        assert!(hash_input.contains("ContactEMail="));
        assert!(hash_input.contains("ContactName="));
        assert!(hash_input.contains("CreationDate="));
        assert!(hash_input.contains("Description="));
        assert!(hash_input.contains("keyVersion=1600708331"));
        assert!(hash_input.contains("licenseVersion=2"));
    }

    #[test]
    fn test_output_order_matches_java_hashmap() {
        let expected_order = vec![
            "CreationDate",
            "Evaluation",
            "Description",
            "Organisation",
            "crowd.LicenseTypeName",
            "crowd.Starter",
            "ContactEMail",
            "SEN",
            "LicenseID",
            "ContactName",
            "MaintenanceExpiryDate",
            "PurchaseDate",
            "crowd.NumberOfUsers",
            "crowd.active",
            "ServerID",
            "keyVersion",
            "LicenseExpiryDate",
            "licenseVersion",
            "licenseHash",
        ];

        let mut ld = LicenseData::new(
            "OrderCheck".to_string(),
            "order@check.com".to_string(),
            "ORD-5678".to_string(),
            "OrderCheck".to_string(),
            false,
            "crowd".to_string(),
        );

        ld.generate_license_hash();
        let ordered_keys = compute_output_order(&ld.insertion_order);

        let mut sorted_expected: Vec<&str> = expected_order.clone();
        sorted_expected.sort();
        let mut sorted_actual: Vec<&String> = ordered_keys.iter().collect();
        sorted_actual.sort();
        
        for (a, b) in sorted_expected.iter().zip(sorted_actual.iter()) {
            assert_eq!(*a, b.as_str(), "Element mismatch in sorted order");
        }

        for (i, key) in ordered_keys.iter().enumerate() {
            if i < expected_order.len() {
                assert_eq!(key.as_str(), expected_order[i], 
                    "Position {}: expected '{}', got '{}'", i, expected_order[i], key);
            }
        }
    }
}