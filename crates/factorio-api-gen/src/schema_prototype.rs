//! Schema for Factorio `prototype-api.json` (stage = `"prototype"`, api_version 6).

use serde::Deserialize;

use crate::schema::ApiType;

#[derive(Debug, Deserialize)]
pub struct PrototypeApi {
    pub application_version: String,
    pub api_version: u32,
    #[serde(default)]
    pub stage: String,
    pub prototypes: Vec<PrototypeDef>,
    #[serde(default)]
    pub types: Vec<PrototypeTypeDef>,
    #[serde(default)]
    pub defines: Vec<crate::schema::Define>,
}

#[derive(Debug, Deserialize)]
pub struct PrototypeDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub r#abstract: bool,
    /// Factorio `type = "..."` discriminant (absent on abstract prototypes).
    #[serde(default)]
    pub typename: Option<String>,
    #[serde(default)]
    pub properties: Vec<PrototypeProperty>,
}

#[derive(Debug, Deserialize)]
pub struct PrototypeProperty {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_name: ApiType,
}

#[derive(Debug, Deserialize)]
pub struct PrototypeTypeDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub r#abstract: bool,
    #[serde(default)]
    pub inline: bool,
    #[serde(rename = "type")]
    pub type_name: ApiType,
    #[serde(default)]
    pub properties: Option<Vec<PrototypeProperty>>,
}
