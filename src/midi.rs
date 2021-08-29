//! Module containing the MIDI-related code

use midir::{MidiOutput, MidiOutputConnection};

/// Sysex message start byte
pub const SYSEX_START_BYTE: u8 = 0xF0;

/// Sysex message end byte
pub const SYSEX_END_BYTE: u8 = 0xF7;

/// Container for connections and state
pub struct MidiConnector {
    /// Objects used for port scanning
    scan_output: Option<MidiOutput>,

    /// Vector of port names that are usable as outputs
    outputs_list: Vec<String>,

    /// Onput connection
    output: Option<MidiOutputConnection>,

    /// Name of the Output port
    output_name: String,
}

impl MidiConnector {
    /// Constructs a new instance
    pub fn new() -> Self {
        Self {
            scan_output: None,
            outputs_list: Vec::new(),
            output: None,
            output_name: String::new(),
        }
    }

    /// Scans the ports
    pub fn scan_ports(&mut self) {
        if self.scan_output.is_none() {
            match MidiOutput::new(&(env!("CARGO_PKG_NAME").to_owned() + " scan output")) {
                Ok(output) => {
                    self.scan_output = Some(output);
                }
                Err(error) => {
                    log::error!("MIDI scan output error: {}", error);
                }
            }
        }

        if self.scan_output.is_some() {
            let output = self.scan_output.as_ref().unwrap();
            let mut outputs_list = Vec::new();
            for port in output.ports().iter() {
                let port_name = output.port_name(port).unwrap();
                outputs_list.push(port_name);
            }
            self.outputs_list = outputs_list;
        }
    }

    /// Sends a message
    pub fn send(&mut self, message: &[u8]) {
        if let Some(conn) = self.output.as_mut() {
            conn.send(message).ok();
        }
    }

    /// Return a vector of outputs
    pub fn get_outputs(&self) -> &Vec<String> {
        &self.outputs_list
    }

    /// Select the output
    pub fn select_output(&mut self, output_name: String) {
        if self.output.is_some() {
            self.output = None;
            self.output_name = String::new();
        }

        if self.scan_output.is_none() {
            match MidiOutput::new(&(env!("CARGO_PKG_NAME").to_owned() + " scan output")) {
                Ok(output) => {
                    self.scan_output = Some(output);
                }
                Err(error) => {
                    log::error!("MIDI scan output error: {}", error);
                }
            }
        }

        let output = self.scan_output.as_ref().unwrap();

        for port in output.ports().iter() {
            let port_name = output.port_name(port).unwrap();
            if port_name == output_name {
                log::info!("MIDI output connected to port {}", port_name);
                self.output = Some(
                    self.scan_output
                        .take()
                        .unwrap()
                        .connect(port, "SysEx Drop Output")
                        .unwrap(),
                );
                self.output_name = port_name;
                break;
            }
        }
    }
}