use std::collections::HashMap;

use java_properties::PropertiesIter;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::AppError;

#[derive(Debug)]
pub struct Configuration {
    pub mongodb_connection_string: String,
    pub elasticsearch_url: String,
}

impl Configuration {
    pub async fn load() -> Result<Configuration, AppError> {
        log::info!("Loading configuration from file");

        let mut file = File::open("application.properties").await?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await?;

        let mut props_map = HashMap::new();
        PropertiesIter::new(buffer.as_slice()).read_into(|k, v| {
            props_map.insert(k, v);
        })?;

        Ok(Self {
            mongodb_connection_string: get_property(&mut props_map, "mongodb_connection_string")?,
            elasticsearch_url: get_property(&mut props_map, "elasticsearch_url")?,
        })
    }
}

fn get_property(props_map: &mut HashMap<String, String>, key: &str) -> Result<String, AppError> {
    props_map
        .remove(key)
        .ok_or_else(|| AppError::Initialization(format!("Missing property '{}'", key)))
}
