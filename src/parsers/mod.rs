use super::errors::*;
use hocon::HoconLoader;
use java_properties;
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use serde_ini;
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

pub trait FileParser {
    fn name(&self) -> &'static str;
    fn can_parse(&self, path: &PathBuf, contents: Result<&str>) -> bool;
    fn parse(&self, path: &PathBuf, contents: Result<&str>) -> Result<Value>;
}

pub fn parsers() -> Vec<Box<dyn FileParser>> {
    vec![
        Box::new(JsonParser {}),
        Box::new(YamlParser {}),
        Box::new(PropertiesParser {}),
        Box::new(OpenAPIParser {}),
        Box::new(TomlParser {}),
        Box::new(IniParser {}),
        Box::new(XmlParser {}),
        Box::new(HoconParser {}),
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
    pub error: Error, // Can't implement Serialize/Deserialize
}

pub struct JsonParser {}
impl FileParser for JsonParser {
    fn name(&self) -> &'static str {
        "json"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["json", "tfstate"])
    }

    fn parse(
        &self,
        path: &PathBuf,
        #[allow(unused_variables)] contents: Result<&str>,
    ) -> Result<Value> {
        let contents = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents.as_str())?)
    }
}

pub struct YamlParser {}
impl FileParser for YamlParser {
    fn name(&self) -> &'static str {
        "yaml"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["yaml"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_yaml::from_str(&contents?)?)
    }
}

pub struct PropertiesParser {}
impl FileParser for PropertiesParser {
    fn name(&self) -> &'static str {
        "java-properties"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["properties"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        match java_properties::read(contents?.as_bytes()) {
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

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["yaml", "json"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        let api: OpenAPI = serde_json::from_str(&contents?)?;
        Ok(serde_json::to_value(api)?)
    }
}

pub struct TomlParser {}
impl FileParser for TomlParser {
    fn name(&self) -> &'static str {
        "toml"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["toml"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        use toml::Value;
        Ok(serde_json::to_value(&contents?.parse::<Value>()?)?)
    }
}
pub struct IniParser {}
impl FileParser for IniParser {
    fn name(&self) -> &'static str {
        "ini"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["ini"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_json::to_value(serde_ini::from_str::<Value>(
            &contents?,
        )?)?)
    }
}

pub struct XmlParser {}
impl FileParser for XmlParser {
    fn name(&self) -> &'static str {
        "xml"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["xml"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_json::to_value(serde_xml_rs::from_str::<Value>(
            &contents?,
        )?)?)
    }
}

pub struct HoconParser {}
impl FileParser for HoconParser {
    fn name(&self) -> &'static str {
        "hocon"
    }

    fn can_parse(&self, path: &PathBuf, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["conf"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &PathBuf,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_json::to_value(
            HoconLoader::new().load_str(&contents?)?.resolve()?,
        )?)
    }
}

// GRPC Parser

#[cfg(test)]
mod tests {
    use crate::parsers;

    #[test]
    fn available_parsers() {
        assert_eq!(parsers::parsers().len(), 8)
    }
}
