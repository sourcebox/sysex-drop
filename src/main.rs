//! Cross-platform utility for dumping sysex files via drag-and-drop GUI

#![windows_subsystem = "windows"]

mod midi;

use eframe::{egui, epi};
use simple_logger::SimpleLogger;
use std::io::{BufRead, BufReader, Seek};
use std::sync::{Arc, Mutex};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

////////////////////////////////////////////////////////////////////////////////

/// Starts the application
fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let app = App::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 300.0)),
        resizable: false,
        drag_and_drop_support: true,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(Box::new(app), native_options);
}

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    /// Selected file path
    #[cfg_attr(feature = "persistence", serde(skip))]
    file_path: Option<std::path::PathBuf>,

    /// File size in bytes
    #[cfg_attr(feature = "persistence", serde(skip))]
    file_size: u64,

    /// No of packets in file
    #[cfg_attr(feature = "persistence", serde(skip))]
    file_packet_count: usize,

    /// Selected MIDI device
    selected_device: Option<String>,

    /// Interval in ms between packets
    packet_interval: u64,

    /// Transfer state
    #[cfg_attr(feature = "persistence", serde(skip))]
    transfer_state: TransferState,

    /// Transfer progress
    #[cfg_attr(feature = "persistence", serde(skip))]
    transfer_progress: f32,

    /// MIDI handler
    #[cfg_attr(feature = "persistence", serde(skip))]
    midi: Arc<Mutex<midi::MidiConnector>>,

    /// Last error message
    #[cfg_attr(feature = "persistence", serde(skip))]
    error_message: Option<String>,

    /// Message channel
    #[cfg_attr(feature = "persistence", serde(skip))]
    message_channel: (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ),

    /// MPSC sender to cancel the transmit thread
    #[cfg_attr(feature = "persistence", serde(skip))]
    transmit_thread_sender: Option<std::sync::mpsc::Sender<bool>>,
}

////////////////////////////////////////////////////////////////////////////////

/// Messages for application actions
#[derive(Debug, Clone)]
pub enum Message {
    /// Force rescanning of devices
    RescanDevices,

    /// Select a device by name
    SelectDevice(String),

    /// Start the transfer
    StartTransfer,

    /// Packet transferred
    PacketTransferred(usize),

    /// Transfer finished successfully
    TransferFinished,

    /// Transfer cancelled
    TransferCancelled,

    /// Error
    Error(String),
}

////////////////////////////////////////////////////////////////////////////////

/// Transfer states
#[derive(PartialEq)]
pub enum TransferState {
    Idle,
    Running,
    Finished,
    Cancelled,
}

////////////////////////////////////////////////////////////////////////////////

impl Default for App {
    fn default() -> Self {
        Self {
            file_path: None,
            file_size: 0,
            file_packet_count: 0,
            selected_device: None,
            packet_interval: 20,
            transfer_state: TransferState::Idle,
            transfer_progress: 0.0,
            midi: Arc::new(Mutex::new(midi::MidiConnector::new())),
            error_message: None,
            message_channel: std::sync::mpsc::channel(),
            transmit_thread_sender: None,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "SysEx Drop"
    }

    /// Called by the frame work to save state before shutdown
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        log::debug!("Saving persistent data.");
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called once on startup
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        #[cfg(feature = "persistence")]
        if let Some(storage) = storage {
            log::debug!("Loading persistent data.");
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.message_channel.0.send(Message::RescanDevices).ok();

        if let Some(device) = &self.selected_device {
            self.message_channel
                .0
                .send(Message::SelectDevice(device.to_owned()))
                .ok();
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        // Continuous run mode is required for message processing
        ctx.request_repaint();

        while let Ok(message) = self.message_channel.1.try_recv() {
            self.process_message(&message);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            ui.scope(|ui| {
                ui.set_enabled(self.transfer_state != TransferState::Running);

                device_selection(
                    ui,
                    self.midi.lock().unwrap().get_outputs(),
                    self.selected_device.to_owned(),
                    &self.message_channel.0,
                );

                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(60.0);

                    ui.centered_and_justified(|ui| {
                        if !ctx.input().raw.hovered_files.is_empty()
                            && self.transfer_state != TransferState::Running
                        {
                            // Files hovered
                            egui::Frame::group(ui.style())
                                .stroke(egui::Stroke::new(1.0, egui::Color32::YELLOW))
                                .show(ui, |ui| {
                                    ui.label("Drop file to open");
                                });
                        } else if self.file_path.is_some() {
                            let basename = self.file_path.as_ref().unwrap().file_name().unwrap();
                            egui::Grid::new("file_info").show(ui, |ui| {
                                ui.label("File:");
                                ui.label(basename.to_str().unwrap_or("Invalid filename"))
                                    .on_hover_text(
                                        self.file_path
                                            .as_ref()
                                            .unwrap()
                                            .to_str()
                                            .unwrap_or("Invalid filename"),
                                    );
                                ui.end_row();
                                ui.label("Size:");
                                ui.label(format!("{}", self.file_size))
                                    .on_hover_text("File size in bytes");
                                ui.end_row();
                                ui.label("Packets:");
                                ui.label(format!("{}", self.file_packet_count))
                                    .on_hover_text("Total number of packets in file");
                                ui.end_row();
                            });
                        } else {
                            ui.label("Drop a sysex file here!");
                        }
                    });

                    // Files dropped
                    if !ctx.input().raw.dropped_files.is_empty()
                        && self.transfer_state != TransferState::Running
                    {
                        for file in &ctx.input().raw.dropped_files {
                            if let Some(path) = &file.path {
                                self.transfer_progress = 0.0;
                                self.transfer_state = TransferState::Idle;
                                match self.process_file(path) {
                                    Ok(()) => self.error_message = None,
                                    Err(error) => {
                                        self.error_message = Some(error.to_string());
                                    }
                                }
                                break;
                            }
                        }
                    }
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.label("Delay between packets:");
                    ui.add(
                        egui::DragValue::new(&mut self.packet_interval)
                            .clamp_range(std::ops::RangeInclusive::new(1, 500))
                            .speed(1.0),
                    );
                    ui.label("ms");
                });
            });

            ui.add_space(20.0);

            ui.scope(|ui| {
                ui.set_enabled(self.file_path.is_some() && self.error_message.is_none());

                ui.horizontal(|ui| {
                    ui.add(
                        egui::ProgressBar::new(self.transfer_progress)
                            .show_percentage()
                            .desired_width(ui.available_width() - 100.0)
                            .animate(self.transfer_state == TransferState::Running),
                    );
                    ui.centered_and_justified(|ui| {
                        if self.transfer_state != TransferState::Running {
                            if ui
                                .button("Start")
                                .on_hover_text("Send file to the device")
                                .clicked()
                            {
                                self.message_channel.0.send(Message::StartTransfer).ok();
                            };
                        } else if ui
                            .button("Cancel")
                            .on_hover_text("Cancel file transfer")
                            .clicked()
                        {
                            self.transmit_thread_sender
                                .as_ref()
                                .unwrap()
                                .send(true)
                                .ok();
                        };
                    });
                });
            });

            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                if let Some(error_message) = &self.error_message {
                    ui.add(
                        egui::Label::new(format!("Error: {}", error_message))
                            .text_color(egui::Color32::RED),
                    );
                } else if self.file_path.is_none() {
                    ui.add(egui::Label::new("No file selected."));
                } else {
                    match self.transfer_state {
                        TransferState::Idle => {
                            ui.add(
                                egui::Label::new("Press start to send the file.")
                                    .text_color(egui::Color32::YELLOW),
                            );
                        }
                        TransferState::Running => {
                            ui.add(egui::Label::new("Transfer in progress."));
                        }
                        TransferState::Finished => {
                            ui.add(
                                egui::Label::new("Transfer finished.")
                                    .text_color(egui::Color32::GREEN),
                            );
                        }
                        TransferState::Cancelled => {
                            ui.add(
                                egui::Label::new("Transfer cancelled.")
                                    .text_color(egui::Color32::RED),
                            );
                        }
                    }
                }
            });
        });

        // Bottom panel with app version
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("v{}", &env!("CARGO_PKG_VERSION")));
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl App {
    /// Process a message
    fn process_message(&mut self, message: &Message) {
        match message {
            Message::RescanDevices => {
                self.midi.lock().unwrap().scan_ports();
            }
            Message::SelectDevice(name) => {
                log::debug!("Device {} selected.", name);
                self.midi.lock().unwrap().select_output(name.to_string());
                self.selected_device = Some(name.to_owned())
            }
            Message::StartTransfer => {
                self.transfer_state = TransferState::Running;
                let file_path = self.file_path.as_ref().unwrap().clone();
                let midi = self.midi.clone();
                let packet_interval = self.packet_interval;
                let message_sender = self.message_channel.0.clone();
                let message_sender_result = self.message_channel.0.clone();
                let (sender, receiver): (
                    std::sync::mpsc::Sender<bool>,
                    std::sync::mpsc::Receiver<bool>,
                ) = std::sync::mpsc::channel();
                self.transmit_thread_sender = Some(sender);
                std::thread::spawn(move || {
                    let result = send_sysex(
                        file_path,
                        midi,
                        std::time::Duration::from_millis(packet_interval),
                        message_sender,
                        receiver,
                    );
                    match result {
                        Ok(finished) => {
                            if finished {
                                message_sender_result.send(Message::TransferFinished).ok();
                            } else {
                                message_sender_result.send(Message::TransferCancelled).ok();
                            }
                        }
                        Err(error) => {
                            message_sender_result
                                .send(Message::Error(format!("{}", error)))
                                .ok();
                        }
                    }
                });
            }
            Message::PacketTransferred(packet_count) => {
                self.transfer_progress = (*packet_count as f32) / (self.file_packet_count as f32)
            }
            Message::TransferFinished => {
                self.transfer_state = TransferState::Finished;
            }
            Message::TransferCancelled => {
                self.transfer_state = TransferState::Cancelled;
            }
            Message::Error(error) => self.error_message = Some(error.to_string()),
        }
    }

    /// Process file
    fn process_file(&mut self, path: &std::path::Path) -> Result<()> {
        // Reset file info initially
        self.file_path = None;
        self.file_size = 0;
        self.file_packet_count = 0;

        let mut file = std::fs::File::open(path.to_path_buf())?;
        let file_size = file.seek(std::io::SeekFrom::End(0))?;

        file.seek(std::io::SeekFrom::Start(0))?;

        let mut buf_reader = BufReader::new(file);
        let mut packet_count = 0;

        loop {
            let mut data = Vec::new();
            let data_length = buf_reader.read_until(midi::SYSEX_END_BYTE, &mut data)?;
            if data_length == 0 {
                // End of file
                break;
            }
            if data[0] != midi::SYSEX_START_BYTE {
                return Err(Box::new(Error::NoStartByte));
            }
            if data[data_length - 1] != midi::SYSEX_END_BYTE {
                return Err(Box::new(Error::NoEndByte));
            }
            packet_count += 1;
        }

        if packet_count == 0 {
            return Err(Box::new(Error::NoPackets));
        }

        // File is valid, so set the info fields
        self.file_path = Some(path.to_path_buf());
        self.file_size = file_size;
        self.file_packet_count = packet_count;

        Ok(())
    }
}

/// Show combobox with devices
pub fn device_selection(
    ui: &mut egui::Ui,
    devices: &[String],
    selected_device: Option<String>,
    message_sender: &std::sync::mpsc::Sender<Message>,
) {
    let mut device_list = Vec::new();
    let mut device_index = 0;

    if !devices.is_empty() {
        for (index, device) in devices.iter().enumerate() {
            device_list.push((&device).to_string());

            if selected_device.is_some() && selected_device.as_ref().unwrap() == device {
                device_index = index;
            }
        }
    }

    let device_count = device_list.len();

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label("Device:");
        });

        ui.scope(|ui| {
            ui.set_enabled(!device_list.is_empty());

            let combo_box = egui::ComboBox::from_id_source("device_list")
                .width(ui.available_width() - 100.0)
                .show_index(ui, &mut device_index, device_list.len(), |i| {
                    if device_count > 0 {
                        device_list[i].clone()
                    } else {
                        String::from("No devices found")
                    }
                });

            if combo_box.changed() && !devices.is_empty() {
                for (index, device) in devices.iter().enumerate() {
                    let d = devices.iter().find(|&x| x == device);
                    if d.is_some() && index == device_index {
                        message_sender
                            .send(Message::SelectDevice(device.to_string()))
                            .ok();
                    }
                }
            };
        });

        ui.centered_and_justified(|ui| {
            if ui
                .button("Rescan")
                .on_hover_text("Search for new MIDI devices")
                .clicked()
            {
                message_sender.send(Message::RescanDevices).ok();
            };
        });
    });
}

fn send_sysex(
    file_path: std::path::PathBuf,
    midi: Arc<Mutex<midi::MidiConnector>>,
    packet_interval: std::time::Duration,
    message_sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<bool>,
) -> Result<bool> {
    let file = std::fs::File::open(file_path)?;

    let mut buf_reader = BufReader::new(file);
    let mut packet_count = 0;

    loop {
        let mut data = Vec::new();
        let data_length = buf_reader.read_until(midi::SYSEX_END_BYTE, &mut data)?;
        if data_length == 0 {
            // End of file
            break;
        }
        packet_count += 1;
        message_sender.send(Message::PacketTransferred(packet_count))?;

        midi.lock().unwrap().send(&data);

        std::thread::sleep(packet_interval);

        if receiver.try_recv().is_ok() {
            return Ok(false);
        }
    }

    Ok(true)
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
    /// Sysex start byte not found
    NoStartByte,

    /// Sysex end byte not found
    NoEndByte,

    /// File does not contain any packets
    NoPackets,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoStartByte =>
                    format!("Start byte 0x{:02X} not found.", midi::SYSEX_START_BYTE),
                Self::NoEndByte => format!("End byte 0x{:02X} not found.", midi::SYSEX_END_BYTE),
                Self::NoPackets => "No sysex packets found.".to_string(),
            }
        )
    }
}
