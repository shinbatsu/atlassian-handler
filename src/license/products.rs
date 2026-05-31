use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProductConfig {
    pub name: String,
}

pub fn get_product_config(product_key: &str) -> Option<ProductConfig> {
    let products: HashMap<&str, (&str, &str, bool)> = [
        ("crowd", "crowd", "Crowd", false),
        ("jira", "jira.product.jira-software", "JIRA Software (Software)", true),
        ("jira-software", "jira.product.jira-software", "JIRA Software", true),
        ("conf", "conf", "Confluence", false),
        ("confluence", "conf", "Confluence", false),
        ("bitbucket", "bitbucket", "Bitbucket", false),
        ("bamboo", "bamboo", "Bamboo", false),
        ("fisheye", "fisheye", "FishEye", false),
        ("crucible", "crucible", "Crucible", false),
        ("jsm", "jsm", "JIRA Service Management", false),
        ("jc", "jc", "JIRA Core", false),
        ("jsd", "jsd", "JIRA Service Desk", false),
        ("questions", "questions", "Questions plugin for Confluence", false),
        ("capture", "capture", "Capture plugin for JIRA", false),
        ("training", "training", "Training plugin for JIRA", false),
        ("portfolio", "portfolio", "Portfolio plugin for JIRA", false),
        ("tc", "tc", "Team Calendars plugin for Confluence", false),
    ]
    .iter().map(|(k, n, l, j)| (*k, (*n, *l, *j))).collect();

    products.get(product_key).map(|(name, _, _)| ProductConfig {
        name: name.to_string()
    })
}

pub fn get_products() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("crowd".to_string(), "Crowd".to_string());
    m.insert("jira".to_string(), "JIRA Software".to_string());
    m.insert("conf".to_string(), "Confluence".to_string());
    m.insert("bitbucket".to_string(), "Bitbucket".to_string());
    m.insert("bamboo".to_string(), "Bamboo".to_string());
    m.insert("fisheye".to_string(), "FishEye".to_string());
    m.insert("crucible".to_string(), "Crucible".to_string());
    m.insert("jsm".to_string(), "JIRA Service Management".to_string());
    m.insert("jc".to_string(), "JIRA Core".to_string());
    m.insert("jsd".to_string(), "JIRA Service Desk".to_string());
    m.insert("questions".to_string(), "Questions plugin for Confluence".to_string());
    m.insert("capture".to_string(), "Capture plugin for JIRA".to_string());
    m.insert("training".to_string(), "Training plugin for JIRA".to_string());
    m.insert("portfolio".to_string(), "Portfolio plugin for JIRA".to_string());
    m.insert("tc".to_string(), "Team Calendars plugin for Confluence".to_string());
    m.insert("confluence".to_string(), "Confluence (alias)".to_string());
    m.insert("jira-software".to_string(), "JIRA Software".to_string());
    m
}