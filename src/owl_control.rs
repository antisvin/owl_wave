use midir::MidiOutputConnection;
use owl_midi::OpenWareMidiSysexCommand;
use wmidi::{Channel, ControlFunction, Error, MidiMessage, U7};

pub struct OwlCommandProcessor {
    pub current_command: Option<OpenWareMidiSysexCommand>,
    pub output: Option<String>,
}

impl OwlCommandProcessor {
    pub const fn new() -> Self {
        OwlCommandProcessor {
            current_command: None,
            output: None,
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
            Some(OpenWareMidiSysexCommand::SYSEX_FIRMWARE_VERSION) => {
                self.output = Some(String::from_utf8_lossy(&data[4..size - 1]).to_string());
                Ok(())
            }
            None => Ok(()),
            _ => Ok(()),
        }
    }
}
