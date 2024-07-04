pub struct M3u8Data {
    pub raw: String,
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

        M3u8Data { raw, links }
    }
}
