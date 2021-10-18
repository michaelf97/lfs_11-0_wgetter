use tokio;
use tokio::runtime::Runtime;
use md5;
use hex::FromHex;
use futures_util::StreamExt;
use std::io::Write;
use std::fs::File;
use std::time::Duration;

const packages: [(&str, &str, &str); 3] = [
    (
        "https://ftp.gnu.org/gnu/autoconf/autoconf-2.71.tar.xz",
        "12cfa1687ffa2606337efe1a64416106",
        "autoconf-2.71.tar.xz",
    ),
    (
        "https://download.savannah.gnu.org/releases/acl/acl-2.3.1.tar.xz",
        "95ce715fe09acca7c12d3306d0f076b2",
        "acl-2.3.1.tar.xz",
    ),
    (
        "https://download.savannah.gnu.org/releases/attr/attr-2.5.1.tar.gz",
        "ac1c5a7a084f0f83b8cace34211f64d8",
        "attr-2.5.1.tar.gz",
    ),
    ];

#[derive(Debug)]
struct Package {
    url: String,
    md5: String,
    client: reqwest::Client,
    path: String
}

impl Package {
    fn from(input: (&str, &str, &str)) -> Package {
        Package {
            url: String::from(input.0),
            md5: String::from(input.1),
            client: reqwest::Client::new(),
            path: String::from(input.2),
        }
    }

    async fn download(&self) {
        println!("Starting download for {}", self.path);
        let mut md5_context = md5::Context::new();
        let mut file: File = File::create(&self.path).unwrap();
        let mut response = self.client.get(&self.url).send().await.unwrap().bytes_stream();

        while let Some(chunk) = response.next().await {
            let chunk = chunk.unwrap();
            md5_context.consume(&chunk);
            file.write(&chunk).expect("Error writing value");
        }

        let correct_hash = <[u8; 16]>::from_hex(self.md5.as_str()).unwrap();
        let computed_hash = md5_context.compute().0;

        let result = computed_hash.iter().zip(correct_hash.iter()).all(|(a,b)| a == b);

        println!("File: {} - Passed: {}", self.path, result);
    }
}

fn main() {
    let packages: Vec<Package> = packages.iter().map(|p| Package::from(*p)).collect();
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async move {
        for p in packages {
            tokio::spawn(async move { p.download().await });
        }
    });
}
