use std::time::{Duration, Instant};
use sysinfo::{Pid, System};
use tracing::{info, warn};

pub struct MemoryProfiler {
    system: System,
    pid: Pid,
    start_time: Instant,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get current PID");

        Self {
            system,
            pid,
            start_time: Instant::now(),
        }
    }

    pub fn log_memory(&mut self, checkpoint: &str) {
        self.system.refresh_process(self.pid);

        if let Some(process) = self.system.process(self.pid) {
            let memory_bytes = process.memory();
            let virtual_memory_bytes = process.virtual_memory();
            let elapsed = self.start_time.elapsed();

            info!(
                "MEMORY[{:.2}s] {}: RSS={:.1}MB VSZ={:.1}MB",
                elapsed.as_secs_f64(),
                checkpoint,
                memory_bytes as f64 / (1024.0 * 1024.0),
                virtual_memory_bytes as f64 / (1024.0 * 1024.0)
            );
        } else {
            warn!(
                "Failed to get process memory info for checkpoint: {}",
                checkpoint
            );
        }
    }

    pub async fn monitor_for_duration(&mut self, duration: Duration, interval: Duration) {
        let start = Instant::now();
        let mut next_log = start;

        while start.elapsed() < duration {
            if Instant::now() >= next_log {
                let elapsed = start.elapsed();
                self.log_memory(&format!("monitor_{}s", elapsed.as_secs()));
                next_log = Instant::now() + interval;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
