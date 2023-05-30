pub mod audio {
    use mpeg2ts::{
        es::StreamType,
        ts::{ReadTsPacket, TsPacketReader, TsPayload},
    };
    use std::{io::Read, fs::File};

    pub fn from_file_path<S: AsRef<str>>(path: S) -> Vec<u8> {
        let file = File::open(path.as_ref()).unwrap();
        extract_ts_audio(file)
    }

    pub fn from_file(file: File) -> Vec<u8> {
        extract_ts_audio(file)
    }

    pub fn from_raw(raw: &[u8]) -> Vec<u8> {
        extract_ts_audio(raw)
    }

    pub fn extract_ts_audio<R: Read>(raw: R) -> Vec<u8> {
        let mut reader = TsPacketReader::new(raw);

        let mut data: Vec<u8> = vec![];
        let mut audio_pid: u16 = 0;

        loop {
            match reader.read_ts_packet() {
                Ok(Some(packet)) => {
                    use TsPayload::*;

                    let pid = packet.header.pid.as_u16();
                    let is_audio_pid = pid == audio_pid;

                    if let Some(payload) = packet.payload {
                        match payload {
                            Pmt(pmt) => {
                                if let Some(el) = pmt
                                    .table
                                    .into_iter()
                                    .find(|el| el.stream_type == StreamType::AdtsAac)
                                {
                                    audio_pid = el.elementary_pid.as_u16();
                                }
                            }
                            Pes(pes) => {
                                if pes.header.stream_id.is_audio() && is_audio_pid {
                                    data.extend_from_slice(&pes.data);
                                }
                            }
                            Raw(bytes) => {
                                if is_audio_pid {
                                    data.extend_from_slice(&bytes);
                                }
                            }
                            _ => (),
                        }
                    }
                }
                Ok(None) => break,
                _ => (),
            }
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};
    use crate::audio::{from_raw, from_file, from_file_path};

    fn get_file() -> Result<File, std::io::Error> {
        File::open("tests/test.ts")
    }

    fn read_file_to_bytes() -> Result<Vec<u8>, std::io::Error> {
        let mut file = get_file()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn assert(source: &[u8], data: &[u8]) {
        assert!(!data.is_empty());
        assert!(data.len() < source.len());
    }

    #[test]
    fn test_from_raw() {
        if let Ok(bytes) = read_file_to_bytes() {
            let data = from_raw(bytes.as_slice());
            assert(&bytes, &data);
        }
    }

    #[test]
    fn test_from_file() {
        let file = get_file().unwrap();
        let bytes = read_file_to_bytes().unwrap();
        let data = from_file(file);
        assert(&bytes, &data);
    }

    #[test]
    fn test_from_file_path() {
        let data = from_file_path("tests/test.ts");
        let bytes = read_file_to_bytes().unwrap();
        assert(&bytes, &data);
    }
}
