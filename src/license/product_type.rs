use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LicenseType {
    Commercial,
}

impl LicenseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseType::Commercial => "COMMERCIAL",
        }
    }
}

impl fmt::Display for LicenseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}