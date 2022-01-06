use midir::{Ignore, MidiInput, MidiInputPorts, MidiOutput, MidiOutputPorts};

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

pub struct MidiInputHandle {
    source: MidiInput,
    pub ports: MidiInputPorts,
    pub names: Vec<String>,
    pub selected_port: usize,
}

impl MidiInputHandle {
    pub fn new(name: &str) -> Self {
        let mut source = MidiInput::new(name).unwrap();
        source.ignore(Ignore::None);
        let ports = MidiInputPorts::new();
        let names = Vec::<String>::new();
        MidiInputHandle {
            source,
            ports,
            names,
            selected_port: 0,
        }
    }
    fn reset(&mut self) {
        self.ports.clear();
        self.names.clear();
    }
    pub fn reload(&mut self) -> &mut Self {
        self.reset();
        for p in self.source.ports().iter() {
            if let Ok(port_name) = self.source.port_name(p) {
                self.names.push(port_name);
                self.ports.push(p.clone());
            }
        }
        self
    }
    pub fn get_selected_port_mut(&mut self) -> &mut usize {
        &mut self.selected_port
    }
}

pub struct MidiOutputHandle {
    source: MidiOutput,
    pub ports: MidiOutputPorts,
    pub names: Vec<String>,
    pub selected_port: usize,
}

impl MidiOutputHandle {
    pub fn new(name: &str) -> Self {
        let source = MidiOutput::new(name).unwrap();
        let ports = MidiOutputPorts::new();
        let names = Vec::<String>::new();
        MidiOutputHandle {
            source,
            ports,
            names,
            selected_port: 0,
        }
    }
    fn reset(&mut self) -> &mut Self {
        self.ports.clear();
        self.names.clear();
        self
    }
    pub fn reload(&mut self) -> &mut Self {
        self.reset();
        for p in self.source.ports().iter() {
            if let Ok(port_name) = self.source.port_name(p) {
                self.names.push(port_name);
                self.ports.push(p.clone());
            }
        }
        self
    }
    pub fn get_selected_port_mut(&mut self) -> &mut usize {
        &mut self.selected_port
    }
}
