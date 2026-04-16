use std::net::Ipv4Addr;

use clone_macro::clone;
use tracing::trace;

use crate::{
    app::{
        remote_print::{RemotePrint, add_printer, scan_for_printers},
        task::{PollResult, Task, TaskApp, thread::TaskThread},
    },
    ui::state::RemotePrintConnectStatus,
};

pub struct PrinterConnect {
    handle: TaskThread<()>,
}

pub struct PrinterScan {
    handle: TaskThread<()>,
}

impl PrinterConnect {
    pub fn new(remote_print: &RemotePrint, address: Ipv4Addr) -> Self {
        let services = remote_print.services.as_ref().unwrap().clone();
        let handle =
            TaskThread::spawn(clone!([{ remote_print.printers } as printers], move || {
                add_printer(services, printers, address).unwrap()
            }));

        Self { handle }
    }
}

impl PrinterScan {
    pub fn new(remote_print: &RemotePrint, broadcast: Ipv4Addr) -> Self {
        let services = remote_print.services.as_ref().unwrap().clone();
        let handle =
            TaskThread::spawn(clone!([{ remote_print.printers } as printers], move || {
                scan_for_printers(services, printers, broadcast).unwrap()
            }));

        Self { handle }
    }
}

impl Task for PrinterConnect {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Add Printer")
            .into_poll_result(|_| {
                let state = &mut app.state;
                state.remote_print_connecting = RemotePrintConnectStatus::None;
                state.working_address.clear();
                PollResult::complete()
            })
    }
}

impl Task for PrinterScan {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Scan for Printers")
            .into_poll_result(|_| {
                trace!("Finished scanning for printers");
                app.state.remote_print_connecting = RemotePrintConnectStatus::None;
                PollResult::complete()
            })
    }
}
