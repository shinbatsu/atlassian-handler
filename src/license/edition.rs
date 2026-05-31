use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LicenseEdition {
    Unlimited,
}

impl LicenseEdition {
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseEdition::Unlimited => "UNLIMITED",
        }
    }
}

impl fmt::Display for LicenseEdition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}