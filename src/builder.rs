use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
};

use crate::M3U8;

#[derive(Debug, Deserialize)]
pub struct M3U8Builder {

    #[serde(default)]
    master_url: String,

    #[serde(default)]
    index_url: String,

    #[serde(default)]
    base_url: String,

    output_dir: String,

    #[serde(skip)]
    header_map: HeaderMap,
}

impl M3U8Builder {
    pub fn new() -> M3U8Builder {
        let file = match std::fs::File::open("config.toml") {
            Ok(f) => f,
            Err(_) => {
                return M3U8Builder {
                    master_url: String::new(),
                    index_url: String::new(),
                    base_url: String::new(),

                    output_dir: "m3mu".to_owned(),
                    header_map: HeaderMap::new(),
                }
            }
        };

        let mut buffer = BufReader::new(file);
        let mut result = String::new();
        buffer.read_to_string(&mut result).unwrap();

        let config: M3U8Builder = toml::from_str(&result).expect("Could not parse config.toml");

        dbg!(&config);
        config
    }

    fn create_header_map(load_header_map: HashMap<String, String>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        for (key, value) in load_header_map {
            let header_name = reqwest::header::HeaderName::from_str(&key).unwrap();
            headers.insert(header_name, HeaderValue::from_str(&value).unwrap());
        }

        headers
    }

    pub fn load_headers(mut self) -> Self {
        let file = match std::fs::File::open("headers.json") {
            Ok(f) => f,
            Err(_) => return self,
        };

        let buffer = BufReader::new(file);

        let header_map: HashMap<String, String> =
            serde_json::from_reader(buffer).expect("could not find headers.json file");

        let header_map = M3U8Builder::create_header_map(header_map);
        self.header_map = header_map;
        self
    }

    pub fn build(self) -> M3U8 {
        let client = reqwest::ClientBuilder::new()
            .default_headers(self.header_map)
            .build()
            .unwrap();

        let mut base_url = self.base_url;

        if !base_url.ends_with("/") {
            base_url.push_str("/");
        }

        M3U8 {
            master_url: self.master_url,
            index_url: self.index_url,
            base_url: base_url,
            client,
            data: None,
            output_dir: PathBuf::from_str(&self.output_dir).unwrap(),
        }
    }
}
