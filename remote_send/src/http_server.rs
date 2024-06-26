use std::{
    collections::HashMap,
    io::{self, Read},
    sync::Arc,
    thread,
};

use afire::{Content, HeaderName, Method, Server, Status};
use parking_lot::RwLock;

type FileStore = Arc<RwLock<HashMap<String, Arc<Vec<u8>>>>>;

pub struct HttpServer {
    files: FileStore,
}

struct ArcReader {
    inner: Arc<Vec<u8>>,
    pos: usize,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn start_async(&self) {
        let mut server = Server::<FileStore>::new("0.0.0.0", 8080).state(self.files.clone());

        server.route(Method::HEAD, "/{file}", |ctx| {
            let file = ctx.param("file");

            let state = ctx.app();
            let files = state.read();
            let Some(_) = files.get(file) else {
                ctx.status(Status::NotFound).text("File not found").send()?;
                return Ok(());
            };

            ctx.send()?;
            Ok(())
        });

        server.route(Method::GET, "/{file}", |ctx| {
            let file = ctx.param("file");
            println!("Sending file `{file}`");

            let state = ctx.app();
            let files = state.read();
            let Some(file) = files.get(file) else {
                ctx.status(Status::NotFound).text("File not found").send()?;
                return Ok(());
            };

            ctx.stream(ArcReader::new(file.clone())).send()?;
            Ok(())
        });

        thread::spawn(|| {
            server.run().unwrap();
        });
    }

    pub fn add_file(&self, name: &str, data: Arc<Vec<u8>>) {
        let mut files = self.files.write();
        files.insert(name.to_owned(), data);
    }

    pub fn remove_file(&self, name: &str) {
        let mut files = self.files.write();
        files.remove(name);
    }
}

impl ArcReader {
    fn new(inner: Arc<Vec<u8>>) -> Self {
        Self { inner, pos: 0 }
    }
}

impl Read for ArcReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.inner.len();
        let remaining = len - self.pos;

        if remaining == 0 {
            return Ok(0);
        }

        let to_read = remaining.min(buf.len());
        buf[..to_read].copy_from_slice(&self.inner[self.pos..self.pos + to_read]);
        self.pos += to_read;

        Ok(to_read)
    }
}
