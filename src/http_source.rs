use std;
use std::io::Read;
use std::fmt;

use futures::future;
use futures::BoxFuture;

use gltf::import::{Source};

extern crate reqwest;

#[derive(Debug)]
pub struct HttpSource {
    pub url: String
}

#[derive(Debug)]
pub enum Error {
    // TODO!: make proper error type
    HttpError,
}

// // TODO!: make clean/nice
impl Source for HttpSource {
    type Error = Error;
    fn source_gltf(&self) -> BoxFuture<Box<[u8]>, Self::Error> {
        fetch_data(self.url.clone())
    }

    fn source_external_data(&self, uri: &str) -> BoxFuture<Box<[u8]>, Self::Error> {
        let url = reqwest::Url::parse(&self.url).unwrap();
        let mut segments = url.path_segments().unwrap().collect::<Vec<_>>();
        let len = segments.len();
        segments[len - 1] = uri.into();
        let new_path = segments.join("/");
        let mut new_url = url.clone();
        new_url.set_path(&new_path);
        fetch_data(new_url.as_str().into())
    }
}

fn fetch_data(url: String) -> BoxFuture<Box<[u8]>, Error> {
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
