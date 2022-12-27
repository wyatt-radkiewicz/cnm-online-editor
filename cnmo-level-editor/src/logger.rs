use std::sync::{Arc, Mutex};
use eframe::egui;

pub type Logs = Arc<Mutex<Vec<String>>>;

pub struct Logger {
    pub logs: Logs,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("[{}] {}", record.metadata().level(), record.args()));
    }

    fn flush(&self) {
        todo!()
    }
}

pub fn show_logs(logs: &Logs, ui: &mut egui::Ui) {
    let row_height = ui.text_style_height(&egui::TextStyle::Body);
    let logs = logs.lock().unwrap();
    egui::ScrollArea::new([true, true])
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show_rows(ui, row_height, logs.len(), |ui, row_range| {
        egui::Grid::new("info_grid")
            .num_columns(1)
            .min_col_width(ui.available_width())
            .striped(true)
            .show(ui, |ui| {
            for row in row_range {
                ui.label(logs[row].as_str());
                ui.end_row();
            }
        });
    });
}
