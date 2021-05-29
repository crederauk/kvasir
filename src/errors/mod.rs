mod errors {
    error_chain! {
        foreign_links {
            Io(std::io::Error);
            JsonParse(serde_json::Error);
            YamlParse(serde_yaml::Error);
            TomlParse(toml::de::Error);
            IniParse(serde_ini::de::Error);
            XmlParse(serde_xml_rs::Error);
        }
    }
}
pub use errors::*;
