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

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::templates::filters;

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
}
