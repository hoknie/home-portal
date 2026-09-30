#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionSettings {
    pub request_at_start: bool,
    pub automation: Vec<String>,
    pub folders: Vec<String>,
}

impl PermissionSettings {
    pub const TABLE: &'static str = "permissions";
    pub const REQUEST_AT_START: &'static str = "request_at_start";
    pub const AUTOMATION: &'static str = "automation";
    pub const FOLDERS: &'static str = "folders";
    pub const KNOWN_FOLDERS: [&'static str; 3] = ["Documents", "Desktop", "Downloads"];
    pub const DEFAULT_APPLICATION: &'static str = "System Events";
}

impl Default for PermissionSettings {
    fn default() -> PermissionSettings {
        PermissionSettings {
            request_at_start: true,
            automation: vec![Self::DEFAULT_APPLICATION.to_string()],
            folders: Self::KNOWN_FOLDERS.map(str::to_string).to_vec(),
        }
    }
}
