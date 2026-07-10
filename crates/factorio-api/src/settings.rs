pub struct ModSettingValue {
    pub value: crate::LuaAny,
}

pub static UNIT_MOD_SETTING: ModSettingValue = ModSettingValue {
    value: crate::LuaAny,
};

pub struct SettingTable {
    pub startup: crate::LuaAny,
    pub global: crate::LuaAny,
    pub player_default: crate::LuaAny,
}

pub const settings: SettingTable = SettingTable {
    startup: crate::LuaAny,
    global: crate::LuaAny,
    player_default: crate::LuaAny,
};

pub struct LuaDataInterface;

impl LuaDataInterface {
    /// Register one or more prototype definitions. Translates to `data:extend({...})`.
    #[allow(unused_variables)]
    pub fn extend<T, I: IntoIterator<Item = T>>(&self, items: I) {}
}

/// The global `data` object used to register prototypes and settings.
pub static data: LuaDataInterface = LuaDataInterface;

pub struct BoolSetting {
    /// Internal mod-namespaced name (e.g. `"my-mod-enabled"`).
    pub name: &'static str,
    /// When the setting takes effect: `"startup"`, `"runtime-global"`, or `"runtime-per-user"`.
    pub setting_type: &'static str,
    /// The default value for this setting.
    pub default_value: bool,
}

pub struct IntSetting {
    /// Internal mod-namespaced name (e.g. `"my-mod-count"`).
    pub name: &'static str,
    /// When the setting takes effect: `"startup"`, `"runtime-global"`, or `"runtime-per-user"`.
    pub setting_type: &'static str,
    /// The default value for this setting.
    pub default_value: i64,
    /// Optional minimum allowed value.
    pub minimum_value: Option<i64>,
    /// Optional maximum allowed value.
    pub maximum_value: Option<i64>,
}

pub struct DoubleSetting {
    /// Internal mod-namespaced name.
    pub name: &'static str,
    /// When the setting takes effect: `"startup"`, `"runtime-global"`, or `"runtime-per-user"`.
    pub setting_type: &'static str,
    /// The default value for this setting.
    pub default_value: f64,
    /// Optional minimum allowed value.
    pub minimum_value: Option<f64>,
    /// Optional maximum allowed value.
    pub maximum_value: Option<f64>,
}

pub struct StringSetting {
    /// Internal mod-namespaced name.
    pub name: &'static str,
    /// When the setting takes effect: `"startup"`, `"runtime-global"`, or `"runtime-per-user"`.
    pub setting_type: &'static str,
    /// The default value for this setting.
    pub default_value: &'static str,
    /// If `true`, the value is not shown in-game (useful for internal state).
    pub hidden: bool,
}
