use midir::MidiOutputConnection;
use owl_midi::OpenWareMidiSysexCommand;
use wmidi::{Channel, ControlFunction, Error, MidiMessage, U7};

pub struct OwlCommandProcessor {
    pub current_command: Option<OpenWareMidiSysexCommand>,
    pub firmware_version: Option<String>,
    pub preset_name: Option<String>,
    pub preset_names: Vec<String>,
    pub resource_offset: usize,
    pub resource_names: Vec<String>,
}

impl OwlCommandProcessor {
    pub const fn new() -> Self {
        OwlCommandProcessor {
            firmware_version: None,
            preset_name: None,
            preset_names: Vec::new(),
            resource_offset: 0,
            resource_names: Vec::new(),
            current_command: None,
        }
    }
    pub fn request_settings(
        &mut self,
        connection: &mut MidiOutputConnection,
        command: OpenWareMidiSysexCommand,
    ) -> Result<(), Box<Error>> {
        let chan = Channel::from_index(0).unwrap();
        let message = MidiMessage::ControlChange(
            chan,
            ControlFunction(
                U7::try_from(owl_midi::OpenWareMidiControl::REQUEST_SETTINGS as u8).unwrap(),
            ),
            U7::try_from(command as u8).unwrap(),
        );
        let mut msg_data = [0u8; 3];
        message.copy_to_slice(&mut msg_data).unwrap();
        println!(
            "MSG={:x?} {:x?} {:x?}",
            msg_data[0], msg_data[1], msg_data[2]
        );
        if command == OpenWareMidiSysexCommand::SYSEX_PRESET_NAME_COMMAND {
            self.preset_names.clear()
        } else if command == OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND {
            self.resource_offset = 0;
            self.resource_names.clear()
        }
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when forwarding message ..."));
        println!("Message sent");
        self.current_command = Some(command);
        Ok(())
    }
    pub fn send_sysex_command(
        &mut self,
        connection: &mut MidiOutputConnection,
        command: OpenWareMidiSysexCommand,
    ) -> Result<(), Box<Error>> {
        let data = [
            owl_midi::MIDI_SYSEX_MANUFACTURER as u8,
            owl_midi::MIDI_SYSEX_OMNI_DEVICE as u8,
            command as u8,
        ];
        let message = MidiMessage::SysEx(U7::try_from_bytes(&data).unwrap());
        let mut msg_data = [0u8; 5];
        message.copy_to_slice(&mut msg_data).unwrap();
        println!(
            "MSG={:x?} {:x?} {:x?} {:x?} {:x?}",
            msg_data[0], msg_data[1], msg_data[2], msg_data[3], msg_data[4]
        );
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when forwarding message ..."));
        println!("Message sent");
        self.current_command = Some(command);
        Ok(())
    }
    pub fn handle_response(&mut self, data: &[u8], size: usize) -> Result<(), Error> {
        // TODO: use different error trait
        match self.current_command {
            Some(cmd) => {
                match cmd {
                    OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION => {
                        println!("FW");
                        self.firmware_version =
                            Some(String::from_utf8_lossy(&data[4..size - 1]).to_string());
                    }
                    OpenWareMidiSysexCommand::SYSEX_PRESET_NAME_COMMAND => {
                        println!("NAME");
                        let pos = data[4] as usize;
                        if pos >= self.preset_names.len() {
                            self.preset_names.resize(pos + 1, String::new());
                        }

                        let mut end = size - 1;
                        for (i, item) in data.iter().enumerate().take(size - 1).skip(6) {
                            if *item == 0 {
                                end = i;
                                break;
                            }
                        }
                        self.preset_names[pos] = String::from_utf8_lossy(&data[5..end]).to_string();
                        println!("P {} = {}", pos, self.preset_names[pos]);
                    }
                    OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND => {
                        println!("RES");
                        let mut pos = data[4] as usize;
                        if self.resource_names.is_empty() {
                            self.resource_offset = pos;
                        }
                        pos -= self.resource_offset;
                        if pos >= self.resource_names.len() {
                            self.resource_names.resize(pos + 1, String::new());
                        }

                        let mut end = size - 1;
                        for (i, item) in data.iter().enumerate().take(size - 1).skip(6) {
                            if *item == 0 {
                                end = i;
                                break;
                            }
                        }
                        self.resource_names[pos] =
                            String::from_utf8_lossy(&data[5..end]).to_string();
                        println!("R {} = {}", pos, self.resource_names[pos]);
                    }
                    _ => {
                        println!("CMD={}", cmd as u8)
                    }
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}
