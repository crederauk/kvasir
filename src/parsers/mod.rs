/*
   Copyright 2021 Credera

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use super::errors::*;
use hocon::HoconLoader;
use log::{trace, warn};
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlparser::dialect::{
    Dialect, GenericDialect, HiveDialect, MsSqlDialect, MySqlDialect, PostgreSqlDialect,
    SQLiteDialect,
};
use sqlparser::parser::{Parser, ParserError};
use std::fs;
use std::path::{Path, PathBuf};

/// Return whether a path has one of the list of specified extensions.
fn has_extension(path: &Path, extensions: &[&str]) -> bool {
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
    fn can_parse(&self, path: &Path, contents: Result<&str>) -> bool;

    /// Parse a file and return a JSON result or an explanatory error.
    fn parse(&self, path: &Path, contents: Result<&str>) -> Result<Value>;
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
        Box::new(SqlParser {}),
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

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["json", "tfstate"])
    }

    fn parse(
        &self,
        path: &Path,
        #[allow(unused_variables)] contents: Result<&str>,
    ) -> Result<Value> {
        let contents = fs::read_to_string(path)?;
        Ok(serde_json::from_str(contents.as_str())?)
    }
}

/// File parser for YAML files.
pub struct YamlParser {}
impl FileParser for YamlParser {
    fn name(&self) -> &'static str {
        "yaml"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["yaml"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_yaml::from_str(contents?)?)
    }
}

/// File parser for Java Properties files.
pub struct PropertiesParser {}
impl FileParser for PropertiesParser {
    fn name(&self) -> &'static str {
        "java-properties"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["properties"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
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

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["yaml", "json"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        let api: OpenAPI = serde_json::from_str(contents?)?;
        Ok(serde_json::to_value(api)?)
    }
}

/// File parser for TOML files.
pub struct TomlParser {}
impl FileParser for TomlParser {
    fn name(&self) -> &'static str {
        "toml"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["toml"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        use toml::Value;
        Ok(serde_json::to_value(contents?.parse::<Value>()?)?)
    }
}

/// File parser for INI files.
pub struct IniParser {}
impl FileParser for IniParser {
    fn name(&self) -> &'static str {
        "ini"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["ini"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_json::to_value(serde_ini::from_str::<Value>(
            contents?,
        )?)?)
    }
}

/// File parser for XML files.
pub struct XmlParser {}
impl FileParser for XmlParser {
    fn name(&self) -> &'static str {
        "xml"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["xml"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_json::to_value(serde_xml_rs::from_str::<Value>(
            contents?,
        )?)?)
    }
}

/// File parser for HOCON files.
pub struct HoconParser {}
impl FileParser for HoconParser {
    fn name(&self) -> &'static str {
        "hocon"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["conf"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        Ok(serde_json::to_value(
            HoconLoader::new().load_str(contents?)?.resolve()?,
        )?)
    }
}

/// File parser for SQL files.
///
/// This parser will iterate through different SQL parsers until a file is successfully
/// parsed, returning an error if none of the parsers succeed.
pub struct SqlParser {}
impl FileParser for SqlParser {
    fn name(&self) -> &'static str {
        "sql"
    }

    fn can_parse(&self, path: &Path, #[allow(unused_variables)] contents: Result<&str>) -> bool {
        has_extension(path, &["sql"])
    }

    fn parse(
        &self,
        #[allow(unused_variables)] path: &Path,
        contents: Result<&str>,
    ) -> Result<Value> {
        let parsers: Vec<Box<dyn Dialect>> = vec![
            Box::new(GenericDialect {}),
            Box::new(PostgreSqlDialect {}),
            Box::new(MySqlDialect {}),
            Box::new(SQLiteDialect {}),
            Box::new(MsSqlDialect {}),
            Box::new(HiveDialect {}),
        ];

        let result = parsers
            .iter()
            .map(|dialect| {
                trace!("  parsing with sql parser {:?}", dialect);
                Parser::parse_sql(
                    dialect.as_ref(),
                    contents.as_ref().map_err(|_| {
                        ParserError::ParserError("Could not read file contents.".to_string())
                    })?,
                )
                .map_err(|e| {
                    warn!("  parsing error: {}", e.to_string());
                    e
                })
            })
            .find(|p| p.is_ok())
            .map(|f| match f {
                Ok(statements) => Ok(serde_json::to_value(&statements)),
                Err(e) => Err(e.to_string()),
            })
            .unwrap_or_else(|| bail!("Could not parse with any SQL parser dialects"));

        Ok(result??)
    }
}

// Protobuf Parser
// CSV Parser

#[cfg(test)]
mod tests {
    use crate::parsers;

    #[test]
    fn available_parsers() {
        assert_eq!(parsers::parsers().len(), crate::parsers::parsers().len())
    }
}
