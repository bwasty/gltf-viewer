use std;
use std::io::Read;
use std::fmt;

use futures::future;
use futures::BoxFuture;

use gltf::import::{Source};

extern crate reqwest;

#[derive(Debug)]
pub struct HttpSource {
    url: reqwest::Url,
}

impl HttpSource {
    pub fn new(url: &str) -> HttpSource {
        HttpSource {
            url: reqwest::Url::parse(url)
                .expect("Failed to parse URL")
        }
    }
}

#[derive(Debug)]
pub enum Error {
    // TODO!: make/use proper error type
    HttpError,
}

impl Source for HttpSource {
    type Error = Error;
    fn source_gltf(&self) -> BoxFuture<Box<[u8]>, Self::Error> {
        fetch_data(self.url.to_string())
    }

    fn source_external_data(&self, uri: &str) -> BoxFuture<Box<[u8]>, Self::Error> {
        let mut new_url = self.url.clone();
        new_url.path_segments_mut()
            .expect("URL cannot be base")
            .pop().push(uri);
        fetch_data(new_url.to_string())
    }
}

fn fetch_data(url: String) -> BoxFuture<Box<[u8]>, Error> {
    // TODO!: use thread
    let future = future::lazy(move || {
        let mut resp = reqwest::get(&url).unwrap();
        assert!(resp.status().is_success());
        let mut data = vec![];
        let _ = resp.read_to_end(&mut data);
        Ok(data.into_boxed_slice())
    });
    Box::new(future)
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "HttpSource Error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        unimplemented!()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}
