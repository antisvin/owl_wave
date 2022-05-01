use midir::MidiOutputConnection;
use owl_midi::{OpenWareMidiSysexCommand, PatchParameterId};
use wmidi::{Channel, ControlFunction, Error, MidiMessage, U7};

pub struct OwlParameter {
    pub id: PatchParameterId,
    pub name: String,
    pub value: f32,
}

pub struct OwlCommandProcessor {
    pub firmware_version: Option<String>,
    pub parameters: Vec<Option<OwlParameter>>,
    pub program_message: Option<String>,
    pub patch_name: Option<String>,
    pub patch_names: Vec<String>,
    pub resource_offset: usize,
    pub resource_names: Vec<String>,
    pub program_stats: Option<String>,
    pub log: String,
}

impl OwlCommandProcessor {
    pub const fn new() -> Self {
        OwlCommandProcessor {
            firmware_version: None,
            parameters: Vec::new(),
            program_message: None,
            patch_name: None,
            patch_names: Vec::new(),
            resource_offset: 0,
            resource_names: Vec::new(),
            program_stats: None,
            log: String::new(),
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
            self.patch_names.clear()
        } else if command == OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND {
            self.resource_offset = 0;
            self.resource_names.clear()
        }
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when forwarding message ..."));
        println!("Message sent");
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
        Ok(())
    }
    pub fn handle_response(&mut self, data: &[u8], size: usize) -> Result<(), Error> {
        // TODO: use different error trait
        if data[1] == owl_midi::MIDI_SYSEX_MANUFACTURER as u8
            && data[2] == owl_midi::MIDI_SYSEX_OWL_DEVICE as u8
        {
            println!("MATCHING...");
            match data.get(3) {
                Some(&cmd) => {
                    match cmd {
                        cmd if OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION as u8 == cmd => {
                            let firmware_version = String::from_utf8_lossy(&data[4..size - 1]);
                            self.firmware_version = Some(firmware_version.to_string());
                            self.log +=
                                format!("< FIRMWARE_VERSION = {firmware_version}\n").as_str();
                        }
                        cmd if OpenWareMidiSysexCommand::SYSEX_PRESET_NAME_COMMAND as u8 == cmd => {
                            let pos = data[4] as usize;
                            if pos >= self.patch_names.len() {
                                self.patch_names.resize(pos + 1, String::new());
                            }

                            let mut end = size - 1;
                            for (i, item) in data.iter().enumerate().take(size - 1).skip(6) {
                                if *item == 0 {
                                    end = i;
                                    break;
                                }
                            }
                            self.patch_names[pos] =
                                String::from_utf8_lossy(&data[5..end]).to_string();
                            if !self.patch_names.is_empty() {
                                self.patch_name = Some(self.patch_names[0].clone());
                            }
                            self.log +=
                                format!("< PATCH_NAME {} = {}\n", pos, self.patch_names[pos])
                                    .as_str();
                        }
                        cmd if OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND as u8
                            == cmd =>
                        {
                            let mut pos = data[4] as usize;
                            if self.resource_names.is_empty() {
                                self.resource_offset = pos;
                            }
                            println!("{pos}");
                            println!("{}", self.resource_offset);
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
                            self.log +=
                                format!("< RESOURCE_NAME {} = {}\n", pos, self.resource_names[pos])
                                    .as_str();
                        }
                        cmd if OpenWareMidiSysexCommand::SYSEX_PROGRAM_MESSAGE as u8 == cmd => {
                            let program_message = String::from_utf8_lossy(&data[4..size - 1]);
                            self.program_message = Some(program_message.to_string());
                            self.log += format!("< PROGRAM_MESSAGE = {program_message}").as_str();
                        }
                        cmd if OpenWareMidiSysexCommand::SYSEX_PROGRAM_STATS as u8 == cmd => {
                            let stats = String::from_utf8_lossy(&data[4..size - 1]);
                            self.program_stats = Some(stats.to_string());
                            self.log += format!("< PROGRAM_STATS {stats}").as_str();
                        }
                        //cmd if OpenWareMidiSysexCommand::SYSEX_PARAMETER_NAME_COMMAND as u8 == cmd => {
                        //}
                        _ => {
                            println!("CMD={}", cmd as u8)
                        }
                    }
                    Ok(())
                }
                None => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}
