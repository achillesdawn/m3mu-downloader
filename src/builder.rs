use reqwest::header::{HeaderMap, HeaderValue};
use std::{collections::HashMap, io::BufReader, path::PathBuf, str::FromStr};

use crate::m3u8::{M3u8Data, M3U8};

pub struct M3U8Builder {
    master_url: String,
    index_url: String,
    base_url: String,
    output_dir: PathBuf,
    data: Option<M3u8Data>,
    full_url: bool,
    header_map: HeaderMap,
}

impl M3U8Builder {
    pub fn new_with_m3u8_url(url: String) -> M3U8Builder {
        return M3U8Builder {
            master_url: String::new(),
            index_url: url,

            base_url: String::new(),

            output_dir: PathBuf::from_str("output").unwrap(),
            header_map: HeaderMap::new(),
            data: None,
            full_url: false,
        };
    }

    fn create_header_map(load_header_map: HashMap<String, String>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        for (key, value) in load_header_map {
            let header_name = reqwest::header::HeaderName::from_str(&key).unwrap();
            headers.insert(header_name, HeaderValue::from_str(&value).unwrap());
        }

        headers
    }

    pub fn load_headers(&mut self, path: &PathBuf) {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let buffer = BufReader::new(file);

        let header_map: HashMap<String, String> =
            serde_json::from_reader(buffer).expect("could not find headers.json file");

        let header_map = M3U8Builder::create_header_map(header_map);
        self.header_map = header_map;
    }

    pub fn set_output_dir(&mut self, output_dir: PathBuf) {
        self.output_dir = output_dir;
    }

    pub fn with_data(data: String) -> M3U8Builder {
        return M3U8Builder {
            master_url: String::new(),
            index_url: String::new(),

            base_url: String::new(),

            output_dir: PathBuf::from_str("output").unwrap(),
            header_map: HeaderMap::new(),
            data: Some(M3u8Data::new(data)),
            full_url: false,
        };
    }

    pub fn set_full_url(&mut self) {
        self.full_url = true;
    }

    pub fn build(mut self) -> M3U8 {
        let client = reqwest::ClientBuilder::new()
            .default_headers(self.header_map)
            .build()
            .unwrap();

        if !self.full_url && self.index_url != "" {
            self.base_url = self.index_url.clone();
            let (base, _) = self.base_url.rsplit_once("/").unwrap();
            self.base_url = base.to_string();
            self.base_url.push('/');
        }

        M3U8 {
            master_url: self.master_url,
            index_url: self.index_url,
            base_url: self.base_url,
            client,
            data: self.data,
            output_dir: self.output_dir,
            full_url: self.full_url,
        }
    }
}
