use std::{
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

pub struct M3u8Data {
    pub links: Vec<String>,
}

impl M3u8Data {
    pub fn new(raw: String) -> Self {
        let mut links: Vec<String> = Vec::new();

        for line in raw.lines() {
            if line.starts_with("#") {
                continue;
            }
            links.push(line.to_owned());
        }

        M3u8Data { links }
    }
}

pub struct M3U8 {
    pub master_url: String,
    pub index_url: String,

    pub base_url: String,

    pub client: reqwest::Client,
    pub data: Option<M3u8Data>,
    pub output_dir: PathBuf,
}

impl M3U8 {
    async fn get_master(&mut self) {
        println!("getting master playlist...\n{}", self.master_url);

        let req = self.client.get(&self.master_url);
        let res = req.send().await.unwrap();
        let text = res.text().await.unwrap();

        for line in text.lines() {
            if line.starts_with("#") || line.is_empty() {
                continue;
            }

            println!(
                "setting index url to {}\nsetting base_url to {}",
                self.index_url, self.base_url
            );
            self.index_url = line.to_owned();
            self.base_url = self.master_url.rsplit_once("/").unwrap().0.to_owned();
            self.base_url.push_str("/");

            break;
        }
    }

    pub async fn get_index(&mut self) -> Vec<String> {
        if !self.master_url.is_empty() {
            self.get_master().await;
        }

        let url = self.index_url.clone();
        println!("getting m3mu index ...\n{}", url);

        let req = self.client.get(url);
        let res = req.send().await.unwrap();
        let text = res.text().await.unwrap();

        let data = M3u8Data::new(text);

        let links = data.links.clone();
        self.data = Some(data);
        links
    }

    pub async fn get_url(&self, url: &String) {
        let mut full_url = self.base_url.clone();
        full_url.push_str(&url);

        let req = self.client.get(full_url);

        let res = req.send().await.unwrap();
        let data = res.bytes().await.unwrap();
        self.write_to_disk(data, url);
    }

    fn write_to_disk(&self, data: bytes::Bytes, name: &String) {
        let filename = name.split_once(".ts").unwrap().0;

        let mut path = self.output_dir.clone();
        path.push(filename);
        let path = path.with_extension("ts");

        println!("writing {:?}", path);

        let file = std::fs::File::create(path).unwrap();
        let mut buf = BufWriter::new(file);

        buf.write_all(&data).unwrap();
    }

    pub fn create_output_dir(&self) {
        if self.output_dir.exists() {
            return;
        }
        std::fs::create_dir(&self.output_dir).unwrap();
    }

    pub fn concat(&self) {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(&self.output_dir).unwrap() {
            let entry = entry.unwrap();

            let path = entry.path();
            files.push(path);
        }

        let pattern = regex::Regex::new(r"seg-(\d+)-|_(\d+).ts").unwrap();

        let mut files: Vec<_> = files
            .iter()
            .map(|file| {
                let file_name = file.to_str().unwrap();

                let m = pattern.captures(file_name).expect("could not find seg-num");

                let num = m.get(1).or(m.get(2)).unwrap().as_str();

                let num = match num.parse::<u32>() {
                    Ok(n) => n,
                    Err(_) => {
                        println!("could not parse .ts number:  {}", num);
                        panic!();
                    }
                };

                (num, file)

            })
            .collect();

        files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut concat = self.output_dir.clone();
        concat.push("concat");
        let concat = concat.with_extension("ts");

        let concat = std::fs::File::create(concat).expect("Could not create concat file");
        let mut buf_writer = BufWriter::new(concat);

        for (_, path) in files {
            let file = std::fs::File::open(path).expect("Could not open file");
            let mut buf_reader = BufReader::new(file);
            std::io::copy(&mut buf_reader, &mut buf_writer).expect("failed to copy to concat");

            std::fs::remove_file(path).unwrap();
        }
    }
}
