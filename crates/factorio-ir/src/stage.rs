#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Settings,
    Data,
    Control,
    Shared,
}

impl Stage {
    /// Infer a stage from a dotted module name (e.g. `"control.on_built_entity"`).
    #[must_use]
    pub fn from_module_name(module_name: &str) -> Option<Self> {
        if module_name == "settings" || module_name.starts_with("settings.") {
            return Some(Self::Settings);
        }
        if module_name == "data" || module_name.starts_with("data.") {
            return Some(Self::Data);
        }
        if module_name == "control" || module_name.starts_with("control.") {
            return Some(Self::Control);
        }
        if module_name == "shared" || module_name.starts_with("shared.") {
            return Some(Self::Shared);
        }
        None
    }

    /// The canonical root module name for this stage.
    #[must_use]
    pub const fn default_module_name(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Data => "data",
            Self::Control => "control",
            Self::Shared => "shared",
        }
    }

    /// The Factorio entry-point file name for this stage, if any.
    ///
    /// `Shared` modules have no entry file - they are required by other stages.
    #[must_use]
    pub const fn entry_file_name(self) -> Option<&'static str> {
        match self {
            Self::Settings => Some("settings.lua"),
            Self::Data => Some("data.lua"),
            Self::Control => Some("control.lua"),
            Self::Shared => None,
        }
    }
}
