#[derive(PartialEq, Eq)]
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
