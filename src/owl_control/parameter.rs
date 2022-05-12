#[derive(Debug, PartialEq)]
pub struct OwlParameter {
    pub name: String,
    pub value: f32,
    pub midi_value: u8,
    pub prev_midi_value: u8,
}

impl OwlParameter {
    pub fn new(name: String) -> Self {
        OwlParameter {
            name,
            value: 0.0,
            midi_value: 0,
            prev_midi_value: 0,
        }
    }
    // Returns true if CC should be sent
    pub fn sync(&mut self) -> bool {
        if self.midi_value != self.prev_midi_value {
            // MIDI value changed
            self.prev_midi_value = self.midi_value;
            self.value = self.midi_value as f32 / 127.0;
        } else {
            // UI value changed
            let expected_value = (self.value * 127.0) as u8;
            if self.midi_value != expected_value {
                self.midi_value = expected_value;
                self.prev_midi_value = expected_value;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::OwlParameter;

    #[test]
    pub fn test_sync() {
        let mut param = OwlParameter::new("foo".to_string());
        assert_eq!(param, OwlParameter::new("foo".to_string()));
        assert!(!param.sync());
        assert_eq!(param, OwlParameter::new("foo".to_string()));
        param.midi_value = 1;
        assert_ne!(param, OwlParameter::new("foo".to_string()));
        assert_eq!(
            param,
            OwlParameter {
                name: "foo".to_string(),
                value: 0.0,
                midi_value: 1,
                prev_midi_value: 0
            }
        );
        assert!(!param.sync());
        assert_eq!(
            param,
            OwlParameter {
                name: "foo".to_string(),
                value: 1.0 / 127.0,
                midi_value: 1,
                prev_midi_value: 1
            }
        );
        param.value = 0.5;
        assert!(param.sync());
        assert_eq!(
            param,
            OwlParameter {
                name: "foo".to_string(),
                value: 0.5,
                midi_value: 63,
                prev_midi_value: 63
            }
        );
        assert!(!param.sync());
        assert_eq!(
            param,
            OwlParameter {
                name: "foo".to_string(),
                value: 0.5,
                midi_value: 63,
                prev_midi_value: 63
            }
        );
    }
}
