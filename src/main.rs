use std::{path::PathBuf, str::FromStr, sync::Arc};

mod m3u8;
use clap::Parser;

mod builder;
use builder::M3U8Builder;

mod args;
use args::Args;

fn main() {
    let args = Args::parse();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut m3u8: M3U8Builder;

            if args.config.is_some() {
                m3u8 = M3U8Builder::new_with_config(args.config.unwrap());
            } else if args.url.is_some() {
                m3u8 = M3U8Builder::new_with_m3u8_url(args.url.unwrap());
            } else if args.concat {
                println!("concating");
                if let Some(output_dir) = args.output_dir {
                    m3u8::concat(output_dir);

                } else {
                    m3u8::concat(PathBuf::from_str("output").unwrap());
                }
                return;
            } else {
                println!("no actionable args found");
                return;
            }

            if let Some(headers) = args.headers {
                m3u8.load_headers(&headers);
            }

            if let Some(output_dir) = args.output_dir {
                m3u8.set_output_dir(output_dir);
            }

            let mut m3u8 = m3u8.build();

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
