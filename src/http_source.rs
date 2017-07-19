use std;
use std::boxed::Box;
use std::io::{self, Read, Write};
use std::fmt;

use futures::Future;

extern crate futures_cpupool;
use self::futures_cpupool::CpuPool;

use gltf::import::{Source};

extern crate reqwest;

pub struct HttpSource {
    url: reqwest::Url,
    cpu_pool: CpuPool,
}

impl fmt::Debug for HttpSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HttpSource {{ url: {} }}", self.url)
    }
}

impl HttpSource {
    pub fn new(url: &str) -> HttpSource {
        // Use 8 threads - like max parallel requests per domain in some browsers
        // Seems to be a sweet spot (tested with VC.gltf)
        let pool = CpuPool::new(8);

        HttpSource {
            url: reqwest::Url::parse(url)
                .expect("Failed to parse URL"),
            cpu_pool: pool
        }
    }
}

#[derive(Debug)]
pub enum Error {
    HttpError(String),
}

impl HttpSource {
    fn fetch_data(&self, url: String) -> Box<Future<Item=Box<[u8]>, Error=Error>> {
        let future = self.cpu_pool.spawn_fn(move || {
            let mut resp = reqwest::get(&url)
                .expect(&format!("Network problem on GET {}", url));
            if !resp.status().is_success() {
                return Err(Error::HttpError(format!("{}: {}", resp.status(), url)));
            }
            let mut data = vec![];
            let _ = resp.read_to_end(&mut data);
            print!(".");
            let _ = io::stdout().flush();
            Ok(data.into_boxed_slice())
        });
        Box::new(future)
    }
}

impl Source for HttpSource {
    type Error = Error;
    fn source_gltf(&self) -> Box<Future<Item=Box<[u8]>, Error=Self::Error>> {
        println!("Downloading");
        let _ = io::stdout().flush();
        self.fetch_data(self.url.to_string())
    }

    fn source_external_data(&self, uri: &str) -> Box<Future<Item=Box<[u8]>, Error=Self::Error>> {
        let mut new_url = self.url.clone();
        new_url.path_segments_mut()
            .expect("URL cannot be base")
            .pop().push(uri);
        self.fetch_data(new_url.to_string())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::HttpError(ref status) => status
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}
