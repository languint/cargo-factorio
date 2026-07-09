use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RuntimeApi {
    pub application_version: String,
    pub api_version: u32,
    pub classes: Vec<Class>,
    pub events: Vec<Event>,
    pub defines: Vec<Define>,
    pub global_objects: Vec<GlobalObject>,
    #[serde(default)]
    pub global_functions: Vec<Method>,
    #[serde(default)]
    pub concepts: Vec<Concept>,
}

#[derive(Debug, Deserialize)]
pub struct Class {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub r#abstract: bool,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub methods: Vec<Method>,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub data: Vec<Parameter>,
}

#[derive(Debug, Deserialize)]
pub struct Concept {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub type_name: ApiType,
}

#[derive(Debug, Deserialize)]
pub struct Define {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub values: Vec<DefineValue>,
    #[serde(default)]
    pub subkeys: Vec<Define>,
}

#[derive(Debug, Deserialize)]
pub struct DefineValue {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct GlobalObject {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub type_name: ApiType,
}

#[derive(Debug, Deserialize)]
pub struct Method {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub return_values: Vec<Parameter>,
    #[serde(default)]
    pub format: MethodFormat,
}

#[derive(Debug, Default, Deserialize)]
pub struct MethodFormat {
    #[serde(default)]
    pub takes_table: bool,
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub read_type: Option<ApiType>,
    #[serde(default)]
    pub write_type: Option<ApiType>,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Deserialize)]
pub struct Parameter {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub type_name: ApiType,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct ApiType(pub serde_json::Value);

impl ApiType {
    pub fn as_simple_name(&self) -> Option<&str> {
        self.0.as_str()
    }

    pub fn complex_type(&self) -> Option<&str> {
        self.0.get("complex_type").and_then(|value| value.as_str())
    }

    pub fn child_type(&self, key: &str) -> Option<ApiType> {
        self.0.get(key).cloned().map(ApiType)
    }

    pub fn options(&self) -> Vec<ApiType> {
        self.0
            .get("options")
            .and_then(|value| value.as_array())
            .map(|values| values.iter().cloned().map(ApiType).collect())
            .unwrap_or_default()
    }
}
