mod errors {
    //! Error chain providing a wrapper around several error types.
    error_chain! {
        foreign_links {
            Io(std::io::Error);
            JsonParse(serde_json::Error);
            YamlParse(serde_yaml::Error);
            TomlParse(toml::de::Error);
            IniParse(serde_ini::de::Error);
            XmlParse(serde_xml_rs::Error);
            HoconParse(hocon::Error);
        }
    }
}
pub use errors::*;
