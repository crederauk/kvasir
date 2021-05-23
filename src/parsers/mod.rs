use java_properties;
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

// Error chaining and configuration
// #![recursion_limit = "1024"]

fn has_extension(path: &PathBuf, extensions: &[&str]) -> bool {
    match path.extension() {
        Some(ext) => ext.to_str().map_or(false, |e| extensions.contains(&e)),
        None => false,
    }
}

mod errors {
    error_chain! {
        foreign_links {
            Io(std::io::Error);
            JsonParse(serde_json::Error);
            YamlParse(serde_yaml::Error);
            //PropertiesParse(java_properties::PropertiesError);
        }
    }
}
pub use errors::*;

pub trait FileParser {
    fn name(&self) -> &'static str;
    fn can_parse(&self, path: &PathBuf) -> bool;
    fn parse(&self, path: &PathBuf) -> Result<Value>;
}

pub fn parsers() -> Vec<Box<dyn FileParser>> {
    vec![
        Box::new(JsonParser {}),
        Box::new(YamlParser {}),
        Box::new(PropertiesParser {}),
        Box::new(OpenAPIParser {}),
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParseSuccess {
    pub path: PathBuf,
    pub parser: String,
    pub contents: Value,
}

#[derive(Debug)]
pub struct ParseFailure {
    pub path: PathBuf,
    pub parser: String,
    pub error: Error,
}

pub struct JsonParser {}

impl FileParser for JsonParser {
    fn name(&self) -> &'static str {
        "json"
    }

    fn can_parse(&self, path: &PathBuf) -> bool {
        has_extension(path, &["json", "tfstate"])
    }

    fn parse(&self, path: &PathBuf) -> Result<Value> {
        let contents = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents.as_str())?)
    }
}

pub struct YamlParser {}
impl FileParser for YamlParser {
    fn name(&self) -> &'static str {
        "yaml"
    }

    fn can_parse(&self, path: &PathBuf) -> bool {
        has_extension(path, &["yaml"])
    }

    fn parse(&self, path: &PathBuf) -> Result<Value> {
        let contents = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&contents.as_str())?)
    }
}

pub struct PropertiesParser {}
impl FileParser for PropertiesParser {
    fn name(&self) -> &'static str {
        "java-properties"
    }

    fn can_parse(&self, path: &PathBuf) -> bool {
        has_extension(path, &["properties"])
    }

    fn parse(&self, path: &PathBuf) -> Result<Value> {
        let contents = fs::read_to_string(path)?;
        match java_properties::read(contents.as_bytes()) {
            Ok(props) => Ok(serde_json::to_value(props)?),
            Err(error) => Err(error.to_string().into()),
        }
    }
}

pub struct OpenAPIParser {}
impl FileParser for OpenAPIParser {
    fn name(&self) -> &'static str {
        "openapi-v3"
    }

    fn can_parse(&self, path: &PathBuf) -> bool {
        has_extension(path, &["yaml", "json"])
    }

    fn parse(&self, path: &PathBuf) -> Result<Value> {
        let contents = fs::read_to_string(path)?;
        let api: OpenAPI = serde_json::from_str(&contents)?;
        Ok(serde_json::to_value(api)?)
    }
}
