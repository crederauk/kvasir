pub mod filters {
    //! Custom filters provided to Tera templates.

    use serde_json::to_value;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::path::Path;
    use tera::Error;

    /// Register custom tera filters
    pub fn register_filters(tera: &mut tera::Tera) {
        tera.register_filter("jsonPath", json_path);
        tera.register_filter("filename", filename);
        tera.register_filter("extension", extension);
        tera.register_filter("directory", directory);
        tera.register_filter("parsed_by", parsed_by);
        tera.register_filter("file", file);
    }

    /// Return a JSON value by applying the provided JSON path to the provided ihput value.
    ///
    /// The `params` map must contain a key "path" with a value of the JSON path.
    pub fn json_path(value: &Value, params: &HashMap<String, Value>) -> tera::Result<Value> {
        let json = to_value(
            jsonpath_lib::select(
                &value,
                params
                    .get("path")
                    .ok_or("No path parameter.")?
                    .as_str()
                    .ok_or("Empty or non-string path parameter.")?,
            )
            .map_err(|e| Error::msg(e.to_string()))?,
        )?;

        Ok(json)
    }

    /// Filter files for those parsed by the specified parser, provided in the `parser` argument.
    ///
    /// The hashmap must contain a key with the value "parser".
    pub fn parsed_by(value: &Value, params: &HashMap<String, Value>) -> tera::Result<Value> {
        let parser_name = params
            .get("parser")
            .ok_or("No parser parameter.")?
            .as_str()
            .ok_or("Empty or non-string parser parameter.")?;

        let json = to_value(
            jsonpath_lib::select(
                &value,
                format!("$.[?(@.parser=='{}')]", parser_name).as_str(),
            )
            .map_err(|e| Error::msg(e.to_string()))?,
        )?;

        Ok(json)
    }

    /// Filter files for the file with the specified path.
    ///
    /// The hashmap must contain a key with the value "path".
    pub fn file(value: &Value, params: &HashMap<String, Value>) -> tera::Result<Value> {
        let path = params
            .get("path")
            .ok_or("No path parameter.")?
            .as_str()
            .ok_or("Empty or non-string path parameter.")?;

        let json = to_value(
            jsonpath_lib::select(&value, format!("$.[?(@.path=='{}')]", path).as_str())
                .map_err(|e| Error::msg(e.to_string()))?,
        )?;

        Ok(json)
    }

    /// Return the filename of a path.
    pub fn filename(
        value: &Value,
        #[allow(unused_variables)] params: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        Ok(to_value(
            Path::new(&value.as_str().ok_or("Path must be a string")?)
                .file_name()
                .unwrap_or_default()
                .to_str(),
        )?)
    }

    /// Return the directory of a path.
    pub fn directory(
        value: &Value,
        #[allow(unused_variables)] params: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        Ok(to_value(
            Path::new(&value.as_str().ok_or("Path must be a string")?)
                .parent()
                .map(|p| p.display().to_string()),
        )?)
    }

    /// Return the filename extension of a path.
    pub fn extension(
        value: &Value,
        #[allow(unused_variables)] params: &HashMap<String, Value>,
    ) -> tera::Result<Value> {
        Ok(to_value(
            Path::new(&value.as_str().ok_or("Path must be a string")?)
                .extension()
                .unwrap_or_default()
                .to_str(),
        )?)
    }
}

pub mod functions {

    use itertools::Itertools;
    use log::error;
    use serde_json::to_value;
    use serde_json::Value;
    use std::collections::HashMap;

    pub fn register_functions(tera: &mut tera::Tera) {
        tera.register_function("glob", glob);
    }

    /// Return the filename extension of a path.
    pub fn glob(args: &HashMap<String, Value>) -> tera::Result<Value> {
        let paths = glob::glob(
            args.get("glob")
                .ok_or("No glob parameter.")?
                .as_str()
                .ok_or("Empty or non-string glob parameter.")?,
        )
        .map_err(|e| e.to_string())
        .map(|paths| {
            paths
                .map(|p| match p {
                    Ok(pb) => Ok(pb.display().to_string()),
                    Err(e) => {
                        error!("Could not list file: {}.", e.to_string());
                        Err(e.to_string())
                    }
                })
                .filter(|r| r.is_ok())
                .map(|p| p.unwrap())
                .collect_vec()
        })?;

        Ok(to_value(paths)?)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::templates::filters;
    use crate::templates::functions;
    use itertools::Itertools;
    use serde_json::json;

    #[test]
    fn extension() {
        let map: HashMap<String, serde_json::Value> = HashMap::new();
        assert_eq!(
            filters::extension(&serde_json::to_value("/path/file.txt").unwrap(), &map).unwrap(),
            serde_json::to_value("txt").unwrap()
        );

        assert_eq!(
            filters::extension(&serde_json::to_value("/path/file").unwrap(), &map).unwrap(),
            serde_json::to_value("").unwrap()
        )
    }

    #[test]
    fn filename() {
        let map: HashMap<String, serde_json::Value> = HashMap::new();
        assert_eq!(
            filters::filename(&serde_json::to_value("/path/file.txt").unwrap(), &map).unwrap(),
            serde_json::to_value("file.txt").unwrap()
        );

        assert_eq!(
            filters::filename(&serde_json::to_value("/path/file").unwrap(), &map).unwrap(),
            serde_json::to_value("file").unwrap()
        )
    }

    #[test]
    fn glob() {
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        map.insert(
            "glob".to_string(),
            serde_json::to_value("test/resources/*.*").unwrap(),
        );

        assert_eq!(
            functions::glob(&map).unwrap().as_array().unwrap().len(),
            glob::glob("test/resources/*.*")
                .unwrap()
                .collect_vec()
                .len()
        );
    }

    #[test]
    fn directory() {
        let map: HashMap<String, serde_json::Value> = HashMap::new();
        assert_eq!(
            filters::directory(&serde_json::to_value("/path/file.txt").unwrap(), &map).unwrap(),
            serde_json::to_value("/path").unwrap()
        );
    }

    #[test]
    fn json_path() {
        let data = serde_json::json!({
            "a": {
                "b": {
                    "c": [1, 2, 3]
                }
            }
        });

        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        map.insert("path".to_string(), serde_json::to_value("$.a.b.c").unwrap());

        assert_eq!(filters::json_path(&data, &map).unwrap(), json!([[1, 2, 3]]));
    }
}
