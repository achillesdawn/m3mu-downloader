use std::{path::PathBuf, str::FromStr, sync::Arc};

mod m3u8;
use clap::Parser;

mod builder;
use builder::M3U8Builder;

mod args;
use args::Args;

fn parse_args() -> Option<m3u8::M3U8> {
    let args = Args::parse();
    let mut m3u8: M3U8Builder;

    if let Some(url) = args.url {
        m3u8 = M3U8Builder::new_with_m3u8_url(url);
    } else if let Some(file) = args.file {
        let content = std::fs::read_to_string(file).unwrap();
        m3u8 = M3U8Builder::with_data(content).set_full_url();
    } else if args.concat {
        println!("concating");
        if let Some(output_dir) = args.output_dir {
            m3u8::concat(output_dir);
        } else {
            m3u8::concat(PathBuf::from_str("output").unwrap());
        }
        return None;
    } else {
        println!("no actionable args found");
        return None;
    }

    if let Some(headers) = args.headers {
        m3u8.load_headers(&headers);
    }

    if let Some(output_dir) = args.output_dir {
        m3u8.set_output_dir(output_dir);
    }

    Some(m3u8.build())
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let Some(mut m3u8) = parse_args() else { return };

            m3u8.create_output_dir();

            let links: Vec<String>;
            if m3u8.data.is_none() {
                links = m3u8.get_index().await;
            } else {
                links = m3u8.data.as_ref().unwrap().links.clone()
            }

            let client = Arc::new(m3u8);

            let mut set = tokio::task::JoinSet::new();

            for link in links.into_iter() {
                let client_clone = client.clone();

                set.spawn(async move {
                    client_clone.get_url(link).await;
                });
            }

            while let Some(_) = set.join_next().await {}

            client.concat();
        });
}
