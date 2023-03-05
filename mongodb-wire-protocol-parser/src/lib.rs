use bson::Document;
use nom::{
    bytes::complete::{take, take_while, take_while_m_n},
    combinator::{map_res, peek},
    number::complete::{le_u32, le_u8},
    IResult,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct MsgHeader {
    pub message_length: u32,
    pub request_id: u32,
    pub response_to: u32,
    pub op_code: u32,
}

impl MsgHeader {
    pub fn from_slice(input: &[u8]) -> Result<MsgHeader, Box<dyn std::error::Error>> {
        if input.len() < 16 {
            return Err("not enough bytes to parse MsgHeader".into());
        }

        Ok(MsgHeader {
            message_length: u32::from_le_bytes([input[0], input[1], input[2], input[3]]),
            request_id: u32::from_le_bytes([input[4], input[5], input[6], input[7]]),
            response_to: u32::from_le_bytes([input[8], input[9], input[10], input[11]]),
            op_code: u32::from_le_bytes([input[12], input[13], input[14], input[15]]),
        })
    }
}

#[derive(Clone, Debug)]
pub enum OpCode {
    OpMsg(OpMsg),
    OpQuery(OpQuery),
}

impl OpCode {
    pub fn command(&self) -> String {
        match self {
            OpCode::OpMsg(op_msg) => op_msg.command(),
            OpCode::OpQuery(op_query) => op_query.command(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpQuery {
    pub header: MsgHeader,
    pub flags: u32,
    pub full_collection_name: String,
    pub number_to_skip: u32,
    pub number_to_return: u32,
    pub query: Document,
    pub return_field_selector: Option<Document>,
}

impl OpQuery {
    pub fn as_documents(&self) -> Vec<Document> {
        vec![self.query.clone()]
    }

    pub fn command(&self) -> String {
        self.query.keys().next().unwrap().to_string()
    }
}

#[derive(Clone, Debug)]
pub struct OpMsg {
    pub header: MsgHeader,
    pub flags: u32,
    pub sections: Vec<Section>,
    pub checksum: Option<u32>,
}

impl OpMsg {
    pub fn as_documents(&self) -> Vec<Document> {
        self.sections
            .iter()
            .filter_map(|section| match section {
                Section::Body(body) => Some(body.payload.clone()),
                Section::DocumentSequence(_) => todo!(),
            })
            .collect()
    }

    pub fn command(&self) -> String {
        match self.clone().sections.first() {
            Some(section) => match section {
                Section::Body(body) => {
                    let payload = body.payload.clone();
                    payload.keys().next().unwrap().to_string()
                }
                Section::DocumentSequence(_) => todo!(),
            },
            None => "ismaster".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Section {
    Body(BodySection),
    DocumentSequence(DocumentSequenceSection),
}

impl Section {
    pub fn new(kind: u8, data: &[u8]) -> Vec<Section> {
        match kind {
            0 => vec![Section::Body(BodySection {
                payload: bson::from_slice(data).unwrap(),
            })],
            1 => vec![Section::DocumentSequence(DocumentSequenceSection {
                identifier: String::from_utf8(data[0..4].to_vec()).unwrap(),
                documents: data[4..].chunks(4).map(|chunk| chunk.to_vec()).collect(),
            })],
            _ => todo!(),
        }
    }

    pub fn from_slice(input: &[u8]) -> Result<(Section, Option<u32>), Box<dyn std::error::Error>> {
        let mut count = 0;
        let kind = u8::from_le_bytes([input[count]]);
        count += 1;

        let payload_size = u32::from_le_bytes([
            input[count],
            input[count + 1],
            input[count + 2],
            input[count + 3],
        ]);

        let payload_data = &input[count..count + payload_size as usize];
        count += payload_size as usize;

        // if we ran out of bytes, we don't have a checksum
        if input.len() < count + 4 {
            return Ok((Section::new(kind, payload_data)[0].clone(), None));
        }

        let checksum = u32::from_le_bytes([
            input[count],
            input[count + 1],
            input[count + 2],
            input[count + 3],
        ]);

        Ok((Section::new(kind, payload_data)[0].clone(), Some(checksum)))
    }

    pub fn parse_payload(input: &[u8]) -> IResult<&[u8], &[u8]> {
        let (input, payload_size) = le_u32(input)?;
        let (input, payload) = take(payload_size)(input)?;
        Ok((input, payload))
    }

    pub fn kind(&self) -> u8 {
        match self {
            Section::Body(_) => 0,
            Section::DocumentSequence(_) => 1,
        }
    }

    pub fn documents(&self) -> Vec<Document> {
        match self {
            Section::Body(body) => vec![body.payload.clone()],
            Section::DocumentSequence(sequence) => sequence.as_documents(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BodySection {
    pub payload: Document,
}

#[derive(Clone, Debug)]
pub struct DocumentSequenceSection {
    pub identifier: String,
    pub documents: Vec<Vec<u8>>,
}

impl DocumentSequenceSection {
    pub fn as_documents(&self) -> Vec<Document> {
        self.documents
            .iter()
            .map(|document| bson::from_slice(document).unwrap())
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("parsing error: {0}")]
    Error(String),
}

pub fn parse(input: Vec<u8>) -> Result<OpCode, ParserError> {
    match parse_op_code(&input) {
        Ok((_, msg)) => Ok(msg),
        Err(e) => Err(ParserError::Error(e.to_string())),
    }
}

pub fn parse_op_code(input: &[u8]) -> IResult<&[u8], OpCode> {
    let (input, header) = parse_header(input)?;
    match header.op_code {
        2013 => {
            let (input, op_msg) = parse_op_msg(input, header)?;
            Ok((input, OpCode::OpMsg(op_msg)))
        }
        2004 => {
            let (input, op_query) = parse_op_query(input, header)?;
            Ok((input, OpCode::OpQuery(op_query)))
        }
        _ => panic!("Unsupported op code: {}", header.op_code),
    }
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], MsgHeader> {
    let (input, message_length) = le_u32(input)?;
    let (input, request_id) = le_u32(input)?;
    let (input, response_to) = le_u32(input)?;
    let (input, op_code) = le_u32(input)?;

    Ok((
        input,
        MsgHeader {
            message_length,
            request_id,
            response_to,
            op_code,
        },
    ))
}

pub fn parse_op_msg(input: &[u8], header: MsgHeader) -> IResult<&[u8], OpMsg> {
    let (input, flags) = le_u32(input)?;
    let (input, sections) = parse_sections(input)?;
    let (input, checksum) = parse_checksum(input, flags)?;

    Ok((
        input,
        OpMsg {
            header,
            flags,
            sections,
            checksum,
        },
    ))
}

fn parse_op_query(input: &[u8], header: MsgHeader) -> IResult<&[u8], OpQuery> {
    let (input, flags) = le_u32(input)?;
    let (input, full_collection_name) = parse_cstring(input)?;
    let (input, number_to_skip) = le_u32(input)?;
    let (input, number_to_return) = le_u32(input)?;
    let (input, size) = peek(le_u32)(input)?;
    let (input, payload) = take(size)(input)?;
    let query = bson::from_slice(payload).unwrap();
    let (input, return_field_selector) = if flags & 1 == 1 {
        let (input, return_field_selector) =
            map_res(take_while_m_n(5, 5, |b| b != 0), bson::from_slice)(input)?;
        (input, Some(return_field_selector))
    } else {
        (input, None)
    };

    Ok((
        input,
        OpQuery {
            header,
            flags,
            full_collection_name,
            number_to_skip,
            number_to_return,
            query,
            return_field_selector,
        },
    ))
}

fn parse_sections(input: &[u8]) -> IResult<&[u8], Vec<Section>> {
    let (input, section) = parse_section(input)?;
    Ok((input, vec![section]))
}

fn parse_section(input: &[u8]) -> IResult<&[u8], Section> {
    let (input, section_type) = le_u8(input)?;
    let (input, section) = match section_type {
        0 => parse_body_section(input),
        1 => parse_document_sequence_section(input),
        _ => panic!("Unknown section type: {}", section_type),
    }?;

    Ok((input, section))
}

fn parse_body_section(input: &[u8]) -> IResult<&[u8], Section> {
    let (input, size) = peek(le_u32)(input)?;
    let (input, payload) = take(size)(input)?;

    Ok((
        input,
        Section::Body(BodySection {
            payload: bson::from_slice(&payload.to_vec()).unwrap(),
        }),
    ))
}

fn parse_document_sequence_section(_input: &[u8]) -> IResult<&[u8], Section> {
    todo!()
    // let (input, identifier) = parse_cstring(input)?;
    // let (input, documents) = parse_documents(input)?;

    // Ok((
    //     input,
    //     Section::DocumentSequence(DocumentSequenceSection {
    //         identifier,
    //         documents,
    //     }),
    // ))
}

fn parse_checksum(input: &[u8], flags: u32) -> IResult<&[u8], Option<u32>> {
    if flags & 1 == 1 {
        let (input, checksum) = le_u32(input)?;
        Ok((input, Some(checksum)))
    } else {
        Ok((input, None))
    }
}

fn parse_cstring(input: &[u8]) -> IResult<&[u8], String> {
    let (input, cstring) = take_while(|b| b != 0)(input)?;
    let (input, _) = take(1usize)(input)?;

    Ok((input, String::from_utf8(cstring.to_vec()).unwrap()))
}
