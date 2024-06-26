use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read},
    net::TcpListener,
    ops::Deref,
    sync::{atomic::Ordering, Arc},
    thread,
};

use afire::{
    internal::{event_loop::EventLoop, handle::handle, socket::Socket},
    trace, Method, Server, Status,
};
use parking_lot::RwLock;

pub struct HttpServer {
    inner: Arc<HttpServerInner>,
}

pub struct HttpServerInner {
    files: RwLock<HashMap<String, Arc<Vec<u8>>>>,
    listener: TcpListener,
}

struct ServerEventLoop;

struct ArcReader {
    inner: Arc<Vec<u8>>,
    pos: usize,
}

impl HttpServer {
    pub fn new(listener: TcpListener) -> Self {
        Self {
            inner: Arc::new(HttpServerInner {
                files: RwLock::new(HashMap::new()),
                listener,
            }),
        }
    }

    pub fn start_async(&self) {
        let mut server = Server::<Arc<HttpServerInner>>::new("0.0.0.0", 0)
            .event_loop(ServerEventLoop)
            .state(self.inner.clone());

        server.route(Method::HEAD, "/{file}", |ctx| {
            let file = ctx.param("file");

            let state = ctx.app();
            let files = state.files.read();
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
            let files = state.files.read();
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

    pub fn shutdown(&self) {
        self.inner.listener.set_nonblocking(true).unwrap();
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

impl EventLoop<Arc<HttpServerInner>> for ServerEventLoop {
    fn run(
        &self,
        server: Arc<Server<Arc<HttpServerInner>>>,
        _addr: std::net::SocketAddr,
    ) -> afire::error::Result<()> {
        let listener = server.app().listener.try_clone()?;
        for i in listener.incoming() {
            if !server.running.load(Ordering::Relaxed) {
                break;
            }

            match i {
                Ok(event) => {
                    let this_server = server.clone();
                    let event = Arc::new(Socket::new(event));
                    server.thread_pool.execute(|| handle(event, this_server));
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_err) => trace!(Level::Error, "Error accepting connection: {_err}"),
            };
        }
        Ok(())
    }
}

impl ArcReader {
    fn new(inner: Arc<Vec<u8>>) -> Self {
        Self { inner, pos: 0 }
    }
}

impl Deref for HttpServer {
    type Target = HttpServerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
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
