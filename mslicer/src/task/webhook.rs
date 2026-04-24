use crate::{
    app::config::ContentType,
    task::{PollResult, Task, TaskApp, thread::TaskThread},
};

pub struct Webhook {
    handle: TaskThread<()>,
}

impl Webhook {
    pub fn new(url: &str, body: String, content_type: ContentType) -> Self {
        let request = ureq::post(url)
            .content_type(content_type.header())
            .header("Accept", content_type.header());

        Self {
            handle: TaskThread::spawn(move || {
                request.send(body).unwrap();
            }),
        }
    }
}

impl Task for Webhook {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Send Webhook")
            .into_poll_result(|_| PollResult::complete())
    }
}
