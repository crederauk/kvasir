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
