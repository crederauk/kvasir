pub mod filters {

    use serde_json::Value;
    use std::collections::HashMap;
    use tera::Error;

    pub fn json_path(value: &Value, params: &HashMap<String, Value>) -> tera::Result<Value> {
        let json = serde_json::to_value(
            jsonpath_lib::select(
                &value,
                params
                    .get("path")
                    .ok_or("No path parameter.")?
                    .as_str()
                    .ok_or("Empty path parameter.")?,
            )
            .map_err(|e| Error::msg(e.to_string()))?,
        )?;

        Ok(json)
    }
}
