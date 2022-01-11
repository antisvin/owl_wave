use std::error::Error;

use midir::{
    MidiInput, MidiInputConnection, MidiInputPorts, MidiOutput, MidiOutputConnection,
    MidiOutputPorts,
};
use wmidi::{Channel, ControlFunction, MidiMessage, U7};

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub enum MidiDeviceSelection {
    All,
    Owl,
}

impl MidiDeviceSelection {
    pub fn show_midi_device(&self, name: &str) -> bool {
        match self {
            MidiDeviceSelection::All => true,
            MidiDeviceSelection::Owl => name.starts_with("OWL-"),
        }
    }
}

pub struct MidiInputHandle<T: 'static> {
    pub connection: Option<MidiInputConnection<T>>,
    pub ports: MidiInputPorts,
    pub names: Vec<String>,
    pub selected_port: usize,
}

impl<T> MidiInputHandle<T> {
    pub fn new<F>(name: &str, selected_port: usize, callback: F, data: T) -> Self
    where
        T: Send + 'static,
        F: FnMut(u64, &[u8], &mut T) + Send + 'static,
    {
        let source = MidiInput::new(name).unwrap();
        let mut ports = MidiInputPorts::new();
        let mut names = Vec::<String>::new();
        for p in source.ports().iter() {
            if let Ok(port_name) = source.port_name(p) {
                names.push(port_name);
                ports.push(p.clone());
            }
        }
        let mut connection = None;
        if !ports.is_empty() {
            connection = source
                .connect(&ports[selected_port], name, callback, data)
                .ok();
        }

        MidiInputHandle {
            connection,
            ports,
            names,
            selected_port,
        }
    }
    pub fn get_selected_port_mut(&mut self) -> &mut usize {
        &mut self.selected_port
    }
}

pub struct MidiOutputHandle {
    connection: Option<MidiOutputConnection>,
    pub ports: MidiOutputPorts,
    pub names: Vec<String>,
    pub selected_port: usize,
}

impl MidiOutputHandle {
    pub fn new(name: &str, selected_port: usize) -> Self {
        let source = MidiOutput::new(name).unwrap();
        let mut ports = MidiOutputPorts::new();
        let mut names = Vec::<String>::new();
        for p in source.ports().iter() {
            if let Ok(port_name) = source.port_name(p) {
                names.push(port_name);
                ports.push(p.clone());
            }
        }
        let mut connection = None;
        if !ports.is_empty() {
            connection = source.connect(&ports[selected_port], name).ok();
        }

        MidiOutputHandle {
            connection,
            ports,
            names,
            selected_port,
        }
    }
    pub fn get_selected_port_mut(&mut self) -> &mut usize {
        &mut self.selected_port
    }
    pub fn request_owl_info(&mut self) -> Result<(), Box<dyn Error>> {
        let chan = Channel::from_index(0).unwrap();
        let message = MidiMessage::ControlChange(
            chan,
            ControlFunction(
                U7::try_from(owl_midi::OpenWareMidiControl::REQUEST_SETTINGS as u8).unwrap(),
            ),
            U7::try_from(owl_midi::OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION as u8).unwrap(),
        );
        let mut msg_data = [0u8; 3];
        message.copy_to_slice(&mut msg_data).unwrap();
        println!(
            "MSG={:x?} {:x?} {:x?}",
            msg_data[0], msg_data[1], msg_data[2]
        );
        if let Some(connection) = &mut self.connection {
            connection
                .send(&msg_data)
                .unwrap_or_else(|_| println!("Error when forwarding message ..."));
            println!("Message sent");
        }
        Ok(())
    }
}
