extern crate bytes;

use tokio::codec::Decoder;
use tokio::codec::Encoder;
use bytes::BytesMut;
use std::io::{Error, ErrorKind};

#[derive(Clone)]
pub struct NtsKeRecord {
    pub critical: bool,
    pub record_type: u16,
    pub contents: Vec<u8>,
}

pub fn serialize_record(rec: NtsKeRecord) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let our_type: u16;
    if rec.critical {
        our_type = 1 << 15 + rec.record_type;
    } else {
        our_type = rec.record_type;
    }
    out.extend(our_type.to_be_bytes().iter());
    let our_len = rec.contents.len() as u16;
    out.extend(our_len.to_be_bytes().iter());
    out.extend(rec.contents.iter());
    return out;
}

pub struct NtsKeCodec {}

impl Decoder for NtsKeCodec {
    type Item = NtsKeRecord;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<NtsKeRecord>, std::io::Error> {
        if buf.len() < 4 {
            return Ok(None);
        }
        let mut record :NtsKeRecord = NtsKeRecord{ critical: true, record_type: 0, contents: vec![]};
        let raw_recordtype: u16;
        let header_buf = buf.split_to(4);
        let head = &header_buf[..];
        let mut tmp: [u8;2] = [0, 0];
        tmp[0] = head[0];
        tmp[1] = head[1];
        raw_recordtype = u16::from_be_bytes(tmp);
        tmp[0] = head[2];
        tmp[1] = head[3];
        let record_len = u16::from_be_bytes(tmp);
        if raw_recordtype & (1<<15) != 0 {
            record.critical = true;
            record.record_type = raw_recordtype ^ (1<<15);
        } else {
            record.critical = false;
            record.record_type = raw_recordtype
        }
        record.contents = Vec::new();
        if buf.len() < (record_len as usize) {
            return Err(Error::new(ErrorKind::InvalidData, "invalid length"))
        }
        let data = buf.split_to(record_len as usize );
        record.contents.extend(data.iter());
        return Ok(Some(record));
    }
}

impl Encoder for NtsKeCodec {
    type Item = NtsKeRecord;
    type Error = std::io::Error;

    fn encode(&mut self, record: NtsKeRecord,  buf: &mut BytesMut)-> Result<(), std::io::Error> {
        let tmp = serialize_record(record);
        buf.extend_from_slice(&tmp[..]);
        Ok(())
    }
}
