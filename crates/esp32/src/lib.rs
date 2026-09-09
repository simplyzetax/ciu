pub mod attachment;
mod espnow;
pub mod network;
pub mod pairing;
mod saved_state;
pub mod wifi;
pub mod wire;

const APP_STACK_SIZE: usize = 10 * 1024;
const PAIRING_STACK_SIZE: usize = 8 * 1024;

/// Runs a firmware application on a task with enough stack for ESP-IDF.
pub fn run_app(name: &'static str, run: fn() -> anyhow::Result<()>) {
    esp_idf_svc::sys::link_patches();

    std::thread::Builder::new()
        .name(name.into())
        .stack_size(APP_STACK_SIZE)
        .spawn(run)
        .expect("failed to start application task")
        .join()
        .expect("application task panicked")
        .expect("application initialization failed");
}

/// Starts the long-running physical pairing task.
pub fn spawn_pairing_task(
    run: impl FnOnce() + Send + 'static,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("pairing".into())
        .stack_size(PAIRING_STACK_SIZE)
        .spawn(run)
}
