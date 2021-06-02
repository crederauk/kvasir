use super::errors::*;
use hocon::HoconLoader;
use java_properties;
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use serde_ini;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Return whether a path has one of the list of specified extensions.
fn has_extension(path: &PathBuf, extensions: &[&str]) -> bool {
    match path.extension() {
        Some(ext) => ext.to_str().map_or(false, |e| extensions.contains(&e)),
        None => false,
    }
}

/// Interface to common functionality for file parsers.
pub trait FileParser {
    /// Return the name of the parser.
    fn name(&self) -> &'static str;

    /// Return whether the specified path is able to be parsed by this parser.
    ///
    /// This check is not intended to be expensive. Whilst the contents of the
    /// file are available for use if required, use the path alone wherever
    /// possible to minimise IO.
    fn can_parse(&self, path: &PathBuf, contents: Result<&str>) -> bool;

    /// Parse a file and return a JSON result or an explanatory error.
    fn parse(&self, path: &PathBuf, contents: Result<&str>) -> Result<Value>;
}

/// Return a list of available file parser instances.
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
/// A successful file parsing result.
pub struct ParseSuccess {
    pub path: PathBuf,
    pub parser: String,
    pub contents: Value,
}

#[derive(Debug)]
/// A failed file parsing result.
pub struct ParseFailure {
    pub path: PathBuf,
    pub parser: String,
    pub error: Error, // Can't implement Serialize/Deserialize
}

/// File parser for JSON files.
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

/// File parser for YAML files.
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

/// File parser for Java Properties files.
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

/// File parser for OpenAPI files.
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

/// File parser for TOML files.
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

/// File parser for INI files.
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

/// File parser for XML files.
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

/// File parser for HOCON files.
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
// CSV Parser

#[cfg(test)]
mod tests {
    use crate::parsers;

    #[test]
    fn available_parsers() {
        assert_eq!(parsers::parsers().len(), 8)
    }
}
