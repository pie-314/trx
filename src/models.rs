/// Represents a package managed by the system
#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub repo: String,
    pub description: String,
    pub installed: bool,
}

impl Package {
    pub fn new(name: &str) -> Self {
        Package {
            name: name.to_string(),
            version: String::new(),
            arch: String::new(),
            repo: String::new(),
            description: String::new(),
            installed: false,
        }
    }
}