use crate::LuaGenerator;

impl LuaGenerator {
    /// Generate a parameter type comment
    pub(crate) fn parameter_type_comment(&self, source_type: Option<&str>) -> String {
        if self.debug_level_at_least(1) {
            source_type.map_or_else(String::new, |ty| format!(" --[[ {ty} ]]"))
        } else {
            String::new()
        }
    }

    /// Generate a variable type comment
    pub(crate) fn variable_type_comment(&self, source_type: Option<&str>) -> String {
        if self.debug_level_at_least(1) {
            source_type.map_or_else(String::new, |ty| format!(" --[[ {ty} ]]"))
        } else {
            String::new()
        }
    }

    /// Generate a function return command
    pub(crate) fn function_return_comment(&self, return_type: Option<&str>) -> String {
        if self.debug_level_at_least(1) {
            return_type.map_or_else(String::new, |ty| format!(" --[[ -> {ty} ]]"))
        } else {
            String::new()
        }
    }
}
