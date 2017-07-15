use std;
use std::io::Read;
use std::fmt;

use futures::BoxFuture;

extern crate futures_cpupool;
use self::futures_cpupool::CpuPool;

extern crate reqwest;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::io::{self, Write};
use futures::{Future, Stream};
use self::hyper::Client;
use self::tokio_core::reactor::Core;

use gltf::import::{Source};

pub struct HttpSource {
    url: reqwest::Url,
    cpu_pool: CpuPool,
    client: Client<hyper::client::HttpConnector>,
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

        let mut core = Core::new().unwrap(); // TODO?
        let client = Client::new(&core.handle());

        HttpSource {
            url: reqwest::Url::parse(url)
                .expect("Failed to parse URL"),
            cpu_pool: pool,
            client: client,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    HttpError(String),
}

impl HttpSource {
    fn fetch_data(&self, url: String) -> BoxFuture<Box<[u8]>, Error> {
        let future = self.cpu_pool.spawn_fn(move || {
            let mut resp = reqwest::get(&url).unwrap(); // TODO!: generate error
            if !resp.status().is_success() {
                return Err(Error::HttpError(format!("{}: {}", resp.status(), url)));
            }
            let mut data = vec![];
            let _ = resp.read_to_end(&mut data);
            Ok(data.into_boxed_slice())
        });
        Box::new(future)
    }
}

impl Source for HttpSource {
//   ^^^^^^ `std::rc::Rc<std::cell::RefCell<http_source::hyper::client::pool::PoolInner<tokio_proto::util::client_proxy::ClientProxy<tokio_proto::streaming::message::Message<http_source::hyper::http::MessageHead<http_source::hyper::http::RequestLine>, http_source::hyper::Body>, tokio_proto::streaming::message::Message<http_source::hyper::http::MessageHead<http_source::hyper::http::RawStatus>, tokio_proto::streaming::body::Body<http_source::hyper::Chunk, http_source::hyper::Error>>, http_source::hyper::Error>>>>` cannot be shared between threads safely

    type Error = Error;
    fn source_gltf(&self) -> BoxFuture<Box<[u8]>, Self::Error> {
        self.fetch_data(self.url.to_string())
    }

    fn source_external_data(&self, uri: &str) -> BoxFuture<Box<[u8]>, Self::Error> {
        let mut new_url = self.url.clone();
        new_url.path_segments_mut()
            .expect("URL cannot be base")
            .pop().push(uri);
        self.fetch_data(new_url.to_string())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::HttpError(ref status) => status
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None // TODO?
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}
