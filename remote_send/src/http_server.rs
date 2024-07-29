use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read},
    net::TcpListener,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use afire::{
    internal::{event_loop::EventLoop, handle::handle, socket::Socket},
    Content, Method, Server, Status,
};
use parking_lot::Mutex;
use serde::Serialize;
use serde_json::json;
use tracing::{error, trace};

use crate::{
    mqtt_server::{Mqtt, MqttInner},
    status::{self, Attributes},
};

pub struct HttpServer {
    inner: Arc<HttpServerInner>,
}

pub struct HttpServerInner {
    files: Mutex<HashMap<String, Arc<Vec<u8>>>>,
    listener: TcpListener,

    proxy_enabled: AtomicBool,
    mqtt_server: Arc<MqttInner>,
}

struct ServerEventLoop;

struct ArcReader {
    inner: Arc<Vec<u8>>,
    pos: usize,
}

impl HttpServer {
    pub fn new(listener: TcpListener, mqtt_server: &Mqtt) -> Self {
        Self {
            inner: Arc::new(HttpServerInner {
                files: Mutex::new(HashMap::new()),
                listener,

                proxy_enabled: AtomicBool::new(false),
                mqtt_server: mqtt_server.inner.clone(),
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
            let files = state.files.lock();
            let Some(_) = files.get(file) else {
                ctx.status(Status::NotFound).text("File not found").send()?;
                return Ok(());
            };

            ctx.send()?;
            Ok(())
        });

        server.route(Method::GET, "/{file}", |ctx| {
            let file_name = ctx.param("file");

            let state = ctx.app();
            let mut files = state.files.lock();
            let Some(file) = files.get(file_name) else {
                ctx.status(Status::NotFound).text("File not found").send()?;
                return Ok(());
            };

            trace!("Sending file `{file_name}`");
            ctx.stream(ArcReader::new(file.clone())).send()?;

            trace!("Removing file `{file_name}`");
            files.remove(file_name);

            Ok(())
        });

        server.route(Method::GET, "/status", |ctx| {
            let state = ctx.app();
            if !state.proxy_enabled.load(Ordering::Relaxed) {
                ctx.status(Status::Forbidden)
                    .text("Proxy is disabled")
                    .send()?;
                return Ok(());
            }

            trace!("Status requested by {}", ctx.req.address);

            #[derive(Serialize)]
            struct Printer<'a> {
                machine_id: &'a str,
                attributes: &'a Attributes,
                status: status::Status,
                last_update: i64,
            }

            let clients = state.mqtt_server.clients.read();
            let clients = clients
                .iter()
                .map(|(machine_id, printer)| Printer {
                    machine_id,
                    attributes: &printer.attributes,
                    status: printer.status.lock().clone(),
                    last_update: printer.last_update.load(Ordering::Relaxed),
                })
                .collect::<Vec<_>>();

            ctx.text(json!(clients)).content(Content::JSON).send()?;
            Ok(())
        });

        thread::spawn(|| {
            server.run().unwrap();
        });
    }

    pub fn shutdown(&self) {
        self.inner.listener.set_nonblocking(true).unwrap();
    }

    pub fn set_proxy_enabled(&self, enabled: bool) {
        self.inner.proxy_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn add_file(&self, name: &str, data: Arc<Vec<u8>>) {
        let mut files = self.files.lock();
        files.insert(name.to_owned(), data);
    }

    pub fn remove_file(&self, name: &str) {
        let mut files = self.files.lock();
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
                Err(_err) => error!("Error accepting connection: {_err}"),
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
