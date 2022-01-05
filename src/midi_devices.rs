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
