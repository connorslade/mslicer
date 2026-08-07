use std::{
    collections::HashSet,
    io::ErrorKind,
    mem,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use clone_macro::clone;
use common::slice::format::RasterFormat;
use notify_rust::Notification;
use parking_lot::{Mutex, MutexGuard};
use tracing::{info, warn};

use crate::{
    app::config::Webhook,
    app_ref_type,
    task::Webhook as WebhookTask,
    ui::popup::{Popup, PopupIcon},
};

#[derive(Default)]
pub struct PrintCompletionState {
    sent: HashSet<String>,
    alert: bool,
    webhook: Webhook,

    pending_tasks: Vec<WebhookTask>,
}
