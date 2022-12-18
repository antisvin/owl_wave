use anyhow::Error;
use midir::MidiOutputConnection;
use owl_midi::{OpenWareMidiSysexCommand, PatchParameterId, SysexConfiguration};
use std::collections::HashMap;
use wmidi::{Channel, ControlFunction, MidiMessage, U7};

use super::{
    parameter::OwlParameter,
    resources::{Resource, ResourceData},
    sysex::SysexData,
};

pub struct OwlCommandProcessor {
    pub firmware_version: Option<String>,
    pub parameters: HashMap<PatchParameterId, OwlParameter>,
    pub program_message: Option<String>,
    pub error_message: Option<String>,
    pub patch_name: Option<String>,
    pub patches: Vec<Option<Resource>>,
    pub resource_offset: usize,
    pub resources: Vec<Option<Resource>>,
    pub program_stats: Option<String>,
    pub settings: HashMap<SysexConfiguration, String>,
    pub log: String,
    pub resource_data: ResourceData,
}

impl OwlCommandProcessor {
    pub fn new() -> Self {
        OwlCommandProcessor {
            firmware_version: None,
            parameters: HashMap::new(),
            program_message: None,
            error_message: None,
            patch_name: None,
            patches: Vec::new(),
            resource_offset: 0,
            resources: Vec::new(),
            program_stats: None,
            settings: HashMap::new(),
            log: String::new(),
            resource_data: ResourceData::new(),
        }
    }
    pub fn request_settings(
        &mut self,
        connection: &mut MidiOutputConnection,
        command: OpenWareMidiSysexCommand,
    ) -> Result<(), Box<Error>> {
        self.log += format!("> {command:?}\n").as_str();
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
            self.patches.clear()
        } else if command == OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND {
            self.resource_offset = 0;
            self.resources.clear()
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
        self.log += format!("> {command:?}\n").as_str();
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
        self.log += format!("> {command:?} = {data:?}\n").as_str();
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
        self.log += format!("> MIDI {message:?}\n").as_str();
        if let MidiMessage::ProgramChange(_, _) = message {
            self.parameters.clear();
        }
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
    pub fn handle_sysex(&mut self, data: &[U7]) -> Result<(), Error> {
        // TODO: use different error trait
        if u8::from(data[0]) as u32 == owl_midi::MIDI_SYSEX_MANUFACTURER
            && u8::from(data[1]) as u32 == owl_midi::MIDI_SYSEX_OWL_DEVICE
        {
            if let Some(cmd) = data
                .get(2)
                .and_then(|&x| OpenWareMidiSysexCommand::try_from(u8::from(x) as isize).ok())
            {
                return self.handle_sysex_command(cmd, &data[3..]);
            }
            // TODO: handle unknown commands here
        }
        Ok(())
    }
    fn handle_sysex_command(
        &mut self,
        cmd: OpenWareMidiSysexCommand,
        data: &[U7],
    ) -> Result<(), Error> {
        let size = data.len();
        match cmd {
            OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION => {
                let firmware_version = String::from_utf8_lossy(U7::data_to_bytes(data));
                self.firmware_version = Some(firmware_version.to_string());
                self.log += format!("< SYSEX_FIRMWARE_VERSION = {firmware_version}\n").as_str();
            }
            OpenWareMidiSysexCommand::SYSEX_PARAMETER_NAME_COMMAND => {
                let param = PatchParameterId::try_from(u8::from(data[0]) as isize).unwrap();
                let parameter_name =
                    String::from_utf8_lossy(U7::data_to_bytes(&data[1..size - 1])).to_string();
                self.log += format!("< SYSEX_PARAMETER_NAME_COMMAND {param:?}: {parameter_name}\n")
                    .as_str();
                self.parameters
                    .insert(param, OwlParameter::new(parameter_name));
                //self.parameters.insert(k, v)
            }
            OpenWareMidiSysexCommand::SYSEX_PRESET_NAME_COMMAND => {
                let pos: usize = u8::from(data[0]).into();
                if pos >= self.patches.len() {
                    self.patches.resize_with(pos + 1, || None);
                }

                let mut end = size - 1;
                for (i, item) in data.iter().enumerate().take(size - 1).skip(1) {
                    if *item == U7::MIN {
                        end = i;
                        break;
                    }
                }
                let mut patch_size = 0;
                patch_size.decode(&data[end + 1..end + 6]).unwrap();
                let mut checksum = 0;
                checksum.decode(&data[end + 6..end + 11]).unwrap();
                self.patches[pos] = Some(Resource::new(
                    pos as u8,
                    String::from_utf8_lossy(U7::data_to_bytes(&data[1..end])).to_string(),
                    patch_size,
                    checksum,
                ));
                if !self.patches.is_empty() {
                    if let Some(patch) = &self.patches[0] {
                        self.patch_name = Some(patch.name.clone());
                    } else {
                        self.patch_name = None
                    }
                }
                self.log += format!(
                    "< SYSEX_PATCH_NAME_COMMAND {pos} = {}\n",
                    self.patches[pos].as_ref().unwrap().name
                )
                .as_str();
            }
            OpenWareMidiSysexCommand::SYSEX_RESOURCE_NAME_COMMAND => {
                let mut pos: usize = u8::from(data[0]).into();
                if self.resources.is_empty() {
                    self.resource_offset = pos;
                }
                pos -= self.resource_offset;
                if pos >= self.resources.len() {
                    self.resources.resize_with(pos + 1, || None);
                }

                let mut end = size - 1;
                for (i, item) in data.iter().enumerate().take(size - 1).skip(1) {
                    if *item == wmidi::U7::MIN {
                        end = i;
                        break;
                    }
                }
                let mut resource_size = 0;
                resource_size.decode(&data[end + 1..end + 6]).unwrap();
                let mut checksum = 0;
                checksum.decode(&data[end + 6..end + 11]).unwrap();
                self.resources[pos] = Some(Resource::new(
                    pos as u8,
                    String::from_utf8_lossy(U7::data_to_bytes(&data[1..end])).to_string(),
                    resource_size,
                    checksum,
                ));
                self.log += format!(
                    "< SYSEX_RESOURCE_NAME_COMMAND {pos} = {}\n",
                    self.resources[pos].as_ref().unwrap().name
                )
                .as_str();
            }
            OpenWareMidiSysexCommand::SYSEX_PROGRAM_MESSAGE => {
                let program_message = String::from_utf8_lossy(U7::data_to_bytes(&data[..size - 1]));
                self.program_message = Some(program_message.to_string());
                self.log += format!("< SYSEX_PROGRAM_MESSAGE = {program_message}\n").as_str();
            }
            OpenWareMidiSysexCommand::SYSEX_PROGRAM_ERROR => {
                let error_message = String::from_utf8_lossy(U7::data_to_bytes(&data[..size - 1]));
                self.error_message = Some(error_message.to_string());
                self.log += format!("< SYSEX_PROGRAM_ERROR = {error_message}\n").as_str();
            }
            OpenWareMidiSysexCommand::SYSEX_PROGRAM_STATS => {
                let stats = String::from_utf8_lossy(U7::data_to_bytes(&data[..size - 1]));
                self.program_stats = Some(stats.to_string());
                self.log += format!("< SYSEX_PROGRAM_STATS {stats}\n").as_str();
            }
            //cmd if OpenWareMidiSysexCommand::SYSEX_PARAMETER_NAME_COMMAND as u8 == cmd => {
            //}
            OpenWareMidiSysexCommand::SYSEX_CONFIGURATION_COMMAND => {
                let command_int = (u8::from(data[0]) as isize) << 8 | u8::from(data[1]) as isize;
                let command = SysexConfiguration::from(command_int);
                let value_str =
                    String::from_utf8_lossy(U7::data_to_bytes(&data[2..size])).to_string();
                println!("{command:?} = {value_str}");
                let result = i64::from_str_radix(value_str.as_str(), 16)
                    .map(|x| x.to_string())
                    .unwrap_or_else(|_err| {
                        self.log += format!("! Error parsing {value_str} as hex\n").as_str();
                        String::new()
                    });
                self.settings.insert(command, result);
                self.log += format!("< SYSEX_CONFIGURATION_COMMAND {command:?}\n").as_str();
            }
            OpenWareMidiSysexCommand::SYSEX_FIRMWARE_UPLOAD => {
                let mut idx = 0;
                if let Ok(_result) = idx.decode(data) {
                    if idx == 0 {
                        self.resource_data.reset();
                    }
                    self.log += format!("< SYSEX_FIRMWARE_UPLOAD #{idx}\n").as_str();
                    //idx.decode(&data[4..9]).unwrap();
                    //let decoded = SysexData::decode(&data[9..size - 1]).unwrap();
                    self.resource_data.process_data(&data[5..])?
                }
            }
            _ => {
                println!("Unhandled command: {cmd:?}")
            }
        }
        Ok(())
    }
    pub fn handle_midi_message(&mut self, midi_message: MidiMessage<'_>) {
        match midi_message {
            MidiMessage::ControlChange(_channel, function, value) => {
                let cc = u8::from(function);
                let value = u8::from(value);
                //let control = OpenWareMidiControl::try_from(cc as isize);
                if let Ok(pid) = PatchParameterId::try_from(cc as isize) {
                    self.log += format!("< PARAMETER {:?} = {}\n", pid.string_id(), value).as_str();
                    self.parameters
                        .entry(pid)
                        .and_modify(|p| p.midi_value = value)
                        .or_insert_with(|| {
                            let mut new_param = OwlParameter::new(pid.string_id().into());
                            new_param.midi_value = value;
                            new_param
                        });
                } else {
                    self.log += format!("< CC{cc} = {value}\n").as_str();
                }
            }
            _ => {
                self.log += format!("< MIDI {midi_message:?}\n").as_str();
            }
        }
    }
}
