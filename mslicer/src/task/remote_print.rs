use std::net::Ipv4Addr;

use remote_print::manager::RemotePrintManager;
use tracing::trace;

use crate::{
    task::{PollResult, Task, TaskApp, thread::TaskThread},
    ui::state::RemotePrintConnectStatus,
};

pub struct PrinterConnect {
    handle: TaskThread<()>,
}

pub struct PrinterScan {
    handle: TaskThread<()>,
}

impl PrinterConnect {
    pub fn new(remote_print: &RemotePrintManager, address: Ipv4Addr) -> Self {
        let inner = remote_print.inner().unwrap();
        let handle = TaskThread::spawn(move || inner.add_printer(address).unwrap());

        Self { handle }
    }
}

impl PrinterScan {
    pub fn new(remote_print: &RemotePrintManager, broadcast: Ipv4Addr) -> Self {
        let inner = remote_print.inner().unwrap();
        let handle = TaskThread::spawn(move || inner.scan(broadcast).unwrap());

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
