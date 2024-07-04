use std::{
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

mod m3u8;
use m3u8::M3u8Data;

mod builder;
use builder::M3U8Builder;

struct M3U8 {
    index_url: String,
    base_url: String,
    client: reqwest::Client,
    data: Option<M3u8Data>,
    output_dir: PathBuf,
}

impl M3U8 {
    async fn get_index(&mut self) -> Vec<String> {
        let req = self.client.get(&self.index_url);
        let res = req.send().await.unwrap();
        let text = res.text().await.unwrap();

        let data = M3u8Data::new(text);

        let links = data.links.clone();
        self.data = Some(data);
        links
    }

    async fn get_url(&self, url: &String) {
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

    fn create_output_dir(&self) {
        if self.output_dir.exists() {
            return;
        }
        std::fs::create_dir(&self.output_dir).unwrap();
    }

    fn concat(&self) {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(&self.output_dir).unwrap() {
            let entry = entry.unwrap();

            let path = entry.path();
            files.push(path);
        }

        let mut files: Vec<_> = files
            .iter()
            .map(|file| {
                let num = file
                    .to_str()
                    .unwrap()
                    .split_once("-")
                    .unwrap()
                    .1
                    .split_once("-")
                    .unwrap()
                    .0;
                let num = num.parse::<u32>().expect("could not parse seg-number");
                (num, file)
            })
            .collect();

        files.sort_by(|a, b| a.cmp(b));

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

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut m3u8 = M3U8Builder::new().load_headers().build();

            m3u8.create_output_dir();
            let links = m3u8.get_index().await;

            dbg!(&links);

            let client = Arc::new(m3u8);

            let mut set = tokio::task::JoinSet::new();

            for link in links.into_iter() {
                let client_clone = client.clone();

                set.spawn(async move {
                    client_clone.get_url(&link).await;
                });
            }

            while let Some(_) = set.join_next().await {}

            client.concat();
        });
}
