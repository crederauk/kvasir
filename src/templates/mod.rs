pub mod filters {
    //! Custom filters provided to Tera templates.

    use serde_json::Value;
    use std::collections::HashMap;
    use tera::Error;

    /// Return a JSON value by applying the provided JSON path to the provided ihput value.
    ///
    /// The `params` map must contain a key "path" with a value of the JSON path.
    pub fn json_path(value: &Value, params: &HashMap<String, Value>) -> tera::Result<Value> {
        let json = serde_json::to_value(
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
}
