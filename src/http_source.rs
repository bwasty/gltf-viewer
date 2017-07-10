use std::io::Read;

use futures::future;
use futures::BoxFuture;

use gltf::import::{Source, Error};

extern crate reqwest;

pub struct HttpSource {
    pub url: String
}

// TODO!: make clean/nice
impl Source for HttpSource {
    fn source_gltf(&self) -> BoxFuture<Box<[u8]>, Error> {
        fetch_data(self.url.clone())
    }

    fn source_external_data(&self, uri: &str) -> BoxFuture<Box<[u8]>, Error> {
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
