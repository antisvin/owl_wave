use wmidi::U7;

#[derive(PartialEq, Debug)]
pub struct CodingResult {
    pub bytes_read: usize,
    pub bytes_written: usize,
}

impl CodingResult {
    pub const fn new(bytes_read: usize, bytes_written: usize) -> Self {
        CodingResult {
            bytes_read,
            bytes_written,
        }
    }
}

pub trait SysexData {
    type Error;

    fn decode(&mut self, buf: &[U7]) -> Result<CodingResult, Self::Error>
/*
    where
        Self: std::marker::Sized
         */;

    fn encode(&self, buf: &mut [U7]) -> Result<CodingResult, Self::Error>;
}

impl SysexData for u32 {
    type Error = &'static str;

    fn decode(&mut self, data: &[U7]) -> Result<CodingResult, Self::Error> {
        if data.len() >= 5 {
            let msb = (u8::from(data[0])) as u32;
            let mut y = (u8::from(data[1]) as u32) << 24;
            y += (u8::from(data[2]) as u32) << 16;
            y += (u8::from(data[3]) as u32) << 8;
            y += u8::from(data[4]) as u32;
            if msb & 0x01 > 0 {
                y += 0x80000000
            };
            if msb & 0x02 > 0 {
                y += 0x800000
            };
            if msb & 0x04 > 0 {
                y += 0x8000
            };
            if msb & 0x08 > 0 {
                y += 0x80
            };
            *self = y;
            Ok(CodingResult::new(5, 4))
        } else {
            Err("Not enough data")
        }
    }

    fn encode(&self, buf: &mut [U7]) -> Result<CodingResult, Self::Error> {
        if buf.len() != 5 {
            Err("Invalid buffer size")
        } else {
            let mut first: u8 = 0;
            let value = *self;
            if value & 0x80000000 > 0 {
                first |= 1;
            }
            if value & 0x800000 > 0 {
                first |= 2;
            }
            if value & 0x8000 > 0 {
                first |= 4;
            }
            if value & 0x80 > 0 {
                first |= 8;
            }

            buf[0] = U7::from_u8_lossy(first);
            buf[1] = U7::from_u8_lossy((value >> 24) as u8);
            buf[2] = U7::from_u8_lossy((value >> 16) as u8);
            buf[3] = U7::from_u8_lossy((value >> 8) as u8);
            buf[4] = U7::from_u8_lossy(value as u8);
            Ok(CodingResult::new(4, 5))
        }
    }
}

impl SysexData for [u8] {
    //    impl SysexData for Vec<u8> {
    type Error = &'static str;

    fn decode(&mut self, data: &[U7]) -> Result<CodingResult, Self::Error> {
        let own_len = self.len();
        let len = data.len();
        let mut max_len = len * 7 / 8;
        if len % 8 > 0 {
            max_len += len % 8;
        }
        if own_len < max_len {
            Err("Buffer too small")
        } else {
            let mut bitmask = 0;
            if !data.is_empty() {
                bitmask = data[0].into();
            }
            let mut pos = 0;
            let mut cnt7 = 0;
            let mut last_cnt = 0;
            for (cnt, &c) in data.iter().enumerate() {
                last_cnt = cnt;
                if cnt7 == 7 {
                    cnt7 = 0;
                    pos += 7;
                    bitmask = u8::from(data[pos + 1]);
                } else if cnt > 0 {
                    let msb = ((bitmask >> cnt7) & 1) << 7;
                    self[pos + cnt7] = msb | u8::from(c);
                    cnt7 += 1;
                }
            }
            Ok(CodingResult::new(last_cnt + 1, pos + cnt7))
        }
    }

    fn encode(&self, buf: &mut [U7]) -> Result<CodingResult, Self::Error> {
        let mut pos = 0;
        let mut bitmask = 0;
        let mut cnt7 = 0;
        for (cnt, &char) in self.iter().enumerate() {
            if cnt % 7 == 0 {
                buf[pos] = U7::from_u8_lossy(bitmask);
                bitmask = 0;
                if cnt > 0 {
                    pos += 8;
                }
                cnt7 = 0;
            }
            buf[pos + cnt7 + 1] = U7::from_u8_lossy(char);
            let msb = (char >> 7) as u8;
            if msb > 0 {
                bitmask |= msb << cnt7;
                buf[pos] = U7::from_u8_lossy(bitmask);
            }
            cnt7 += 1;
        }
        Ok(CodingResult::new(self.len(), pos + cnt7 + 1))
    }
}

#[cfg(test)]
mod test {
    use crate::owl_control::sysex::*;

    #[test]
    fn test_decode_u32() {
        let data = U7::try_from_bytes(&[0, 0, 0, 0, 1]).unwrap();
        let mut result = 0;
        assert_eq!(result.decode(data), Ok(CodingResult::new(5, 4)));
        assert_eq!(result, 1);
        let data = U7::try_from_bytes(&[0, 0, 0, 1]).unwrap();
        assert_eq!(result.decode(data), Err("Not enough data"));
        let data = U7::try_from_bytes(&[4, 0, 0, 0, 0x2c]).unwrap();
        assert_eq!(result.decode(data), Ok(CodingResult::new(5, 4)));
        assert_eq!(result, 32812);
    }
    #[test]
    fn test_encode_u32() {
        let mut result = [U7::MIN; 5];
        let data = U7::try_from_bytes(&[0, 0, 0, 0, 1]).unwrap();
        assert_eq!(1u32.encode(&mut result), Ok(CodingResult::new(4, 5)));
        assert_eq!(data, result.as_slice());

        result = [U7::MIN; 5];
        let data = U7::try_from_bytes(&[0x04, 0x00, 0x00, 0x00, 0x2c]).unwrap();
        assert_eq!(32812u32.encode(&mut result), Ok(CodingResult::new(4, 5)));
        assert_eq!(data, result.as_slice());
    }
    #[test]
    fn test_decode_u8_slice() {
        let data = U7::try_from_bytes(&[0, 0]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(2, 1)));
        assert_eq!(result[..1], [0]);

        let data = U7::try_from_bytes(&[0, 1]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(2, 1)));
        assert_eq!(result[..1], [1]);

        let data = U7::try_from_bytes(&[0, 127]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(2, 1)));
        assert_eq!(result[..1], [127]);

        let data = U7::try_from_bytes(&[1, 0]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(2, 1)));
        assert_eq!(result[..1], [128]);

        let data = U7::try_from_bytes(&[1, 1]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(2, 1)));
        assert_eq!(result[..1], [129]);

        let data = U7::try_from_bytes(&[0, 0, 1, 2, 3, 4, 5, 6]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(8, 7)));
        assert_eq!(result[..7], [0, 1, 2, 3, 4, 5, 6]);

        let data = U7::try_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 0, 8]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(10, 8)));
        assert_eq!(result[..8], [1, 2, 3, 4, 5, 6, 7, 8]);

        let data = U7::try_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 1, 0]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(10, 8)));
        assert_eq!(result[..8], [1, 2, 3, 4, 5, 6, 7, 128]);

        let data = U7::try_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 3, 0, 1]).unwrap();
        let mut result = [0; 32];
        assert_eq!(result.decode(data), Ok(CodingResult::new(11, 9)));
        assert_eq!(result[..9], [1, 2, 3, 4, 5, 6, 7, 128, 129]);
    }
    #[test]
    fn test_encode_u8_slice() {
        let data = [0].as_slice();
        let mut buf = [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(1, 2));
        assert_eq!(
            U7::try_from_bytes(&[0, 0]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [1].as_slice();
        let mut buf = [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(1, 2));
        assert_eq!(
            U7::try_from_bytes(&[0, 1]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [127].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(1, 2));
        assert_eq!(
            U7::try_from_bytes(&[0, 127]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [128].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(1, 2));
        assert_eq!(
            U7::try_from_bytes(&[1, 0]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [129].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(1, 2));
        assert_eq!(
            U7::try_from_bytes(&[1, 1]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [0, 1, 2, 3, 4, 5, 6].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(7, 8));
        assert_eq!(
            U7::try_from_bytes(&[0, 0, 1, 2, 3, 4, 5, 6]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [1, 2, 3, 4, 5, 6, 7, 8].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(8, 10));
        assert_eq!(
            U7::try_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 0, 8]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [1, 2, 3, 4, 5, 6, 7, 128].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(8, 10));
        assert_eq!(
            U7::try_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 1, 0]).unwrap(),
            &buf[..result.bytes_written]
        );

        let data = [1, 2, 3, 4, 5, 6, 7, 128, 129].as_slice();
        let buf = &mut [U7::MIN; 32];
        let result = data.encode(buf.as_mut_slice()).unwrap();
        assert_eq!(result, CodingResult::new(9, 11));
        assert_eq!(
            U7::try_from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 3, 0, 1]).unwrap(),
            &buf[..result.bytes_written]
        );
    }
}
