use std::collections::HashMap;

use midir::MidiOutputConnection;
use owl_midi::{OpenWareMidiSysexCommand, PatchParameterId, SysexConfiguration};
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
    pub error_message: Option<String>,
    pub patch_name: Option<String>,
    pub patch_names: Vec<String>,
    pub resource_offset: usize,
    pub resource_names: Vec<String>,
    pub program_stats: Option<String>,
    pub settings: HashMap<SysexConfiguration, String>,
    pub log: String,
}

impl OwlCommandProcessor {
    pub fn new() -> Self {
        OwlCommandProcessor {
            firmware_version: None,
            parameters: Vec::new(),
            program_message: None,
            error_message: None,
            patch_name: None,
            patch_names: Vec::new(),
            resource_offset: 0,
            resource_names: Vec::new(),
            program_stats: None,
            settings: HashMap::new(),
            log: String::new(),
        }
    }
    pub fn request_settings(
        &mut self,
        connection: &mut MidiOutputConnection,
        command: OpenWareMidiSysexCommand,
    ) -> Result<(), Box<Error>> {
        self.log += format!("> {:?}\n", command).as_str();
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
        if command == OpenWareMidiSysexCommand::SYSEX_PRESET_NAME_COMMAND {
            self.patch_names.clear()
        } else if command == OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND {
            self.resource_offset = 0;
            self.resource_names.clear()
        }
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when forwarding message ..."));
        Ok(())
    }
    pub fn send_sysex_command(
        &mut self,
        connection: &mut MidiOutputConnection,
        command: OpenWareMidiSysexCommand,
    ) -> Result<(), Box<Error>> {
        self.log += format!("> {:?}\n", command).as_str();
        let data = [
            owl_midi::MIDI_SYSEX_MANUFACTURER as u8,
            owl_midi::MIDI_SYSEX_OMNI_DEVICE as u8,
            command as u8,
        ];
        let message = MidiMessage::SysEx(U7::try_from_bytes(&data).unwrap());
        let mut msg_data = [0u8; 5];
        message.copy_to_slice(&mut msg_data).unwrap();
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when sending SysEx ..."));
        Ok(())
    }
    pub fn send_sysex_string(
        &mut self,
        connection: &mut MidiOutputConnection,
        command: OpenWareMidiSysexCommand,
        string: &[u8],
    ) -> Result<(), Box<Error>> {
        let mut data = vec![
            owl_midi::MIDI_SYSEX_MANUFACTURER as u8,
            owl_midi::MIDI_SYSEX_OMNI_DEVICE as u8,
            command as u8,
        ];
        data.extend(string.iter());
        let message = MidiMessage::SysEx(U7::try_from_bytes(&data).unwrap());
        let mut msg_data = Vec::new();
        msg_data.resize(message.bytes_size(), 0);
        message.copy_to_slice(&mut msg_data).unwrap();
        self.log += format!("> {:?} = {:?}\n", command, data).as_str();
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when sending SysEx ..."));
        Ok(())
    }

    pub fn send_message(
        &mut self,
        connection: &mut MidiOutputConnection,
        message: MidiMessage<'_>,
    ) -> Result<(), Box<Error>> {
        self.log += format!("> MIDI {:?}\n", message).as_str();
        /*
        let message = MidiMessage::ControlChange(
            Channel::Ch1,
            ControlFunction::from(U7::try_from(cc).unwrap()),
            U7::try_from(value).unwrap(),
        );
        */
        let mut msg_data = [0u8; 3];
        message.copy_to_slice(&mut msg_data).unwrap();
        connection
            .send(&msg_data)
            .unwrap_or_else(|_| println!("Error when sending MIDI message ..."));
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
                                format!("< SYSEX_FIRMWARE_VERSION = {firmware_version}\n").as_str();
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
                            self.log += format!(
                                "< SYSEX_PATCH_NAME_COMMAND {pos} = {}\n",
                                self.patch_names[pos]
                            )
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
                            self.log += format!(
                                "< SYSEX_RESOURCE_NAME_COMMAND {pos} = {}\n",
                                self.resource_names[pos]
                            )
                            .as_str();
                        }
                        cmd if OpenWareMidiSysexCommand::SYSEX_PROGRAM_MESSAGE as u8 == cmd => {
                            let program_message = String::from_utf8_lossy(&data[4..size - 1]);
                            self.program_message = Some(program_message.to_string());
                            self.log +=
                                format!("< SYSEX_PROGRAM_MESSAGE = {program_message}").as_str();
                        }
                        cmd if OpenWareMidiSysexCommand::SYSEX_PROGRAM_STATS as u8 == cmd => {
                            let stats = String::from_utf8_lossy(&data[4..size - 1]);
                            self.program_stats = Some(stats.to_string());
                            self.log += format!("< SYSEX_PROGRAM_STATS {stats}").as_str();
                        }
                        //cmd if OpenWareMidiSysexCommand::SYSEX_PARAMETER_NAME_COMMAND as u8 == cmd => {
                        //}
                        cmd if OpenWareMidiSysexCommand::SYSEX_CONFIGURATION_COMMAND as u8
                            == cmd =>
                        {
                            let command_int = (data[4] as isize) << 8 | data[5] as isize;
                            let command = SysexConfiguration::from(command_int);
                            let value_str = String::from_utf8_lossy(&data[6..size - 1]).to_string();
                            let result = i64::from_str_radix(value_str.as_str(), 16)
                                .map(|x| x.to_string())
                                .unwrap_or_else(|_err| {
                                    self.log +=
                                        format!("! Error parsing {value_str} as hex\n").as_str();
                                    String::new()
                                });
                            self.settings.insert(command, result);
                            self.log +=
                                format!("< SYSEX_CONFIGURATION_COMMAND {:?}\n", command).as_str();
                        }
                        _ => {
                            println!("CMD={}", cmd as u8)
                        }
                    }
                    Ok(())
                }
                None => Ok(()),
            }
        } else {
            let msg = MidiMessage::try_from(data);
            if let Ok(MidiMessage::ControlChange(_channel, function, _value)) = msg {
                self.log += format!("CC = {:?}\n", function).as_str();
            }
            Ok(())
        }
    }
}
