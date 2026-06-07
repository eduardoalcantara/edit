//! Amostragem periódica e leve do consumo total de memória do processo.

use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessesToUpdate, System};

const DEFAULT_INTERVAL: Duration = Duration::from_secs(2);

/// Monitor reutilizável do RSS/working set total do aplicativo.
pub struct MemoryMonitor {
    system: System,
    pid: Pid,
    cached_bytes: Option<u64>,
    last_update: Instant,
    interval: Duration,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        let pid = Pid::from_u32(std::process::id());
        let system = System::new();
        let mut monitor = Self {
            system,
            pid,
            cached_bytes: None,
            last_update: Instant::now() - DEFAULT_INTERVAL,
            interval: DEFAULT_INTERVAL,
        };
        monitor.refresh_if_due();
        monitor
    }

    /// Atualiza a amostra apenas se o intervalo tiver expirado.
    pub fn refresh_if_due(&mut self) -> bool {
        if self.last_update.elapsed() < self.interval {
            return false;
        }
        self.sample_now();
        true
    }

    /// Rótulo compacto para o rodapé (`Mem 128MB`), ou `None` se indisponível.
    pub fn display_label(&self) -> Option<String> {
        self.cached_bytes.map(format_memory_label)
    }

    pub fn cached_bytes(&self) -> Option<u64> {
        self.cached_bytes
    }

    #[cfg(test)]
    fn with_interval(interval: Duration) -> Self {
        let mut monitor = Self::new();
        monitor.interval = interval;
        monitor
    }

    pub(crate) fn set_cached_for_test(&mut self, bytes: Option<u64>) {
        self.cached_bytes = bytes;
        self.last_update = Instant::now();
    }

    fn sample_now(&mut self) {
        self.system.refresh_processes(ProcessesToUpdate::Some(&[self.pid]), false);
        self.cached_bytes = self.system.process(self.pid).map(|process| process.memory());
        self.last_update = Instant::now();
    }
}

/// Formato compacto alinhado ao rodapé: `Mem 128MB`.
pub fn format_memory_label(bytes: u64) -> String {
    let mb = bytes / (1024 * 1024);
    if mb > 0 {
        format!("Mem {mb}MB")
    } else {
        let kb = bytes.max(1024) / 1024;
        format!("Mem {kb}KB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_memory_label_uses_mb() {
        assert_eq!(format_memory_label(128 * 1024 * 1024), "Mem 128MB");
    }

    #[test]
    fn format_memory_label_small_values_use_kb() {
        assert_eq!(format_memory_label(512 * 1024), "Mem 512KB");
    }

    #[test]
    fn display_label_hidden_when_unavailable() {
        let mut monitor = MemoryMonitor::with_interval(Duration::from_secs(60));
        monitor.set_cached_for_test(None);
        assert_eq!(monitor.display_label(), None);
    }

    #[test]
    fn display_label_shows_cached_value() {
        let mut monitor = MemoryMonitor::with_interval(Duration::from_secs(60));
        monitor.set_cached_for_test(Some(64 * 1024 * 1024));
        assert_eq!(monitor.display_label(), Some("Mem 64MB".to_string()));
    }

    #[test]
    fn refresh_respects_interval() {
        let mut monitor = MemoryMonitor::with_interval(Duration::from_secs(60));
        monitor.set_cached_for_test(Some(1_000_000));
        assert!(!monitor.refresh_if_due());
        assert_eq!(monitor.cached_bytes(), Some(1_000_000));
    }
}
