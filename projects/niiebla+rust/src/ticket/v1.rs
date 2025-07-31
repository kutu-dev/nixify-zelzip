// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of the Ticket V1 extension.

use crate::title_id::TitleId;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};
use thiserror::Error;
use util::StreamPin;

// WARNING! HAZMAT! ACHTUNG! PELIGRO! THIS FORMAT IS REALLY SHITTY SO THIS IS
// THE CLEANEST WAY TO WRITE THIS AND PRESERVE PROPER TYPING.
//
// As a side note, this extension has barely seen any usage outside some DLC management on
// [Wii no Ma](https://en.wikipedia.org/wiki/Wii_no_Ma), so unless someone requests better support
// don't waste time improving this.
//
// "When I wrote this code, only god and I know how it worked.
// Now, only god knows it" - Kutu 2025-06-19 21:12:52Z

#[derive(Debug)]
/// Extra data available on V1 tickets.
pub struct PreSwitchTicketV1 {
    /// The set of data sections present on the V1 ticket.
    pub sections: Vec<PreSwitchTicketV1Section>,

    // TODO(DISCOVER)
    /// Extra flags for the V1 ticket itself, its meaning is still unknown.
    pub flags: u32,
}

impl PreSwitchTicketV1 {
    const HEADER_SIZE: u16 = 20;
    const SECTION_HEADER_SIZE: u16 = 20;

    pub(super) fn new<T: Read + Seek>(stream: T) -> Result<Self, PreSwitchTicketV1Error> {
        let mut stream = StreamPin::new(stream)?;

        let version = stream.read_u16::<BE>()?;
        if version != 1 {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1Version(version));
        };

        let header_size = stream.read_u16::<BE>()?;
        if header_size != Self::HEADER_SIZE {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1HeaderSize(
                header_size,
            ));
        }

        let v1_data_size = stream.read_u32::<BE>()?;

        let first_section_header_offset = stream.read_u32::<BE>()?;
        let number_of_sections = stream.read_u16::<BE>()?;

        let section_header_size = stream.read_u16::<BE>()?;
        if section_header_size != Self::SECTION_HEADER_SIZE {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1SectionHeaderSize(
                section_header_size,
            ));
        }

        let flags = stream.read_u32::<BE>()?;

        let mut sections = Vec::new();

        stream.seek_from_pin(first_section_header_offset.into())?;

        for _ in 0..number_of_sections {
            sections.push(PreSwitchTicketV1Section::new(&mut stream)?);
        }

        let v1 = Self { sections, flags };

        if v1_data_size != v1.size() {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1TotalSize(
                v1_data_size,
            ));
        }

        Ok(v1)
    }

    pub(super) fn dump<T: Write + Seek>(&self, stream: T) -> io::Result<()> {
        let mut stream = StreamPin::new(stream)?;

        // Ticket V1 version
        stream.write_u16::<BE>(1)?;

        stream.write_u16::<BE>(Self::HEADER_SIZE)?;
        stream.write_u32::<BE>(self.size())?;

        // Skip this for now as we cannot know the position of the first section yet
        let first_section_byte_header_position = stream.relative_position()? as u64;
        stream.seek_relative(4)?;

        stream.write_u16::<BE>(self.sections.len() as u16)?;
        stream.write_u16::<BE>(Self::SECTION_HEADER_SIZE)?;
        stream.write_u32::<BE>(self.flags)?;

        let mut start_of_records = vec![];
        for section in &self.sections {
            start_of_records.push(stream.relative_position()? as u32);
            section.records.dump(&mut stream)?;
        }

        for (i, section) in self.sections.iter().enumerate() {
            if i == 0 {
                let first_section_byte_position = stream.relative_position()? as u64;

                stream.seek_from_pin(first_section_byte_header_position as i64)?;
                stream.write_u32::<BE>(first_section_byte_position as u32)?;

                stream.seek_from_pin(first_section_byte_position as i64)?;
            }

            stream.write_u32::<BE>(start_of_records[i])?;

            stream.write_u32::<BE>(section.records.len())?;
            stream.write_u32::<BE>(section.records.size_of_one_record())?;
            stream.write_u32::<BE>(Self::SECTION_HEADER_SIZE.into())?;

            stream.write_u16::<BE>(match section.records {
                PreSwitchTicketV1Records::Permanent(_) => 1,
                PreSwitchTicketV1Records::Subscription(_) => 2,
                PreSwitchTicketV1Records::Content(_) => 3,
                PreSwitchTicketV1Records::ContentConsumption(_) => 4,
                PreSwitchTicketV1Records::AccessTitle(_) => 5,
            })?;

            stream.write_u16::<BE>(section.flags)?;
        }

        Ok(())
    }

    pub(super) fn size(&self) -> u32 {
        let mut size = Self::HEADER_SIZE as u32
            + (Self::SECTION_HEADER_SIZE as u32 * self.sections.len() as u32);

        for section in &self.sections {
            size += section.records.size();
        }

        size
    }
}

#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum PreSwitchTicketV1Error {
    #[error("Unknown ticket v1 version: {0}")]
    UnknownTicketV1Version(u16),

    #[error("Unknown ticket v1 header size: {0}")]
    UnknownTicketV1HeaderSize(u16),

    #[error("Unknown ticket v1 section header size: {0}")]
    UnknownTicketV1SectionHeaderSize(u16),

    #[error("Unknown ticket v1 section type: {0}")]
    UnknownTicketV1SectionKind(u16),

    #[error("Unknown ticket v1 total size: {0}")]
    UnknownTicketV1TotalSize(u32),

    #[error("Unknown ticket record size: {0}")]
    UnknownTicketV1RecordSize(u32),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug)]
/// The data of a section inside a V1 ticket.
pub struct PreSwitchTicketV1Section {
    /// The records inside the section.
    pub records: PreSwitchTicketV1Records,

    /// Extra flags for the V1 ticket section itself, its meaning is still unknown.
    pub flags: u16,
}

#[derive(Debug)]
/// The set of records a section can have. Due to technical limitations on the format itself, all
/// records in a section must be of the same kind.
pub enum PreSwitchTicketV1Records {
    /// A set of "permanent" records.
    Permanent(Vec<PreSwitchTicketV1RecordPermanent>),

    /// A set of "subscription" records.
    Subscription(Vec<PreSwitchTicketV1RecordSubscription>),

    /// A set of "content" records.
    Content(Vec<PreSwitchTicketV1RecordContent>),

    /// A set of "content consumption" records.
    ContentConsumption(Vec<PreSwitchTicketV1RecordContentConsumption>),

    /// A set of "access title" records.
    AccessTitle(Vec<PreSwitchTicketV1RecordAccessTitle>),
}

impl PreSwitchTicketV1Records {
    fn size(&self) -> u32 {
        self.size_of_one_record() * self.len()
    }

    fn size_of_one_record(&self) -> u32 {
        match self {
            Self::Permanent(_) => 16 + 4,
            Self::Subscription(_) => 16 + 4 + 4,
            Self::Content(_) => 128 + 4,
            Self::ContentConsumption(_) => 2 + 2 + 4,
            Self::AccessTitle(_) => 8 + 8,
        }
    }

    fn len(&self) -> u32 {
        (match self {
            Self::Permanent(data) => data.len(),
            Self::Subscription(data) => data.len(),
            Self::Content(data) => data.len(),
            Self::ContentConsumption(data) => data.len(),
            Self::AccessTitle(data) => data.len(),
        }) as u32
    }

    pub(crate) fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        match self {
            Self::Permanent(data) => {
                for record in data {
                    record.reference_id.dump(&mut stream)?;
                }
            }

            Self::Subscription(data) => {
                for record in data {
                    stream.write_u32::<BE>(record.expiration_time)?;
                    record.reference_id.dump(&mut stream)?;
                }
            }

            Self::Content(data) => {
                for record in data {
                    stream.write_u32::<BE>(record.offset_content_index)?;
                    stream.write_all(&record.access_mask)?;
                }
            }

            Self::ContentConsumption(data) => {
                for record in data {
                    stream.write_u16::<BE>(record.content_index)?;
                    stream.write_u16::<BE>(record.limit_code)?;
                    stream.write_u32::<BE>(record.limit_value)?;
                }
            }

            Self::AccessTitle(data) => {
                for record in data {
                    record.title_id.dump(&mut stream)?;
                    stream.write_u64::<BE>(record.title_mask)?;
                }
            }
        }

        Ok(())
    }
}

impl PreSwitchTicketV1Section {
    fn new<T: Read + Seek>(stream: &mut StreamPin<T>) -> Result<Self, PreSwitchTicketV1Error> {
        let section_records_offset = stream.read_u32::<BE>()?;
        let number_of_records = stream.read_u32::<BE>()?;

        // NOTE: Not worth checking
        let _size_of_a_record = stream.read_u32::<BE>()?;
        let _section_total_size = stream.read_u32::<BE>()?;

        let section_kind = stream.read_u16::<BE>()?;
        let flags = stream.read_u16::<BE>()?;

        let next_section_position = stream.stream_position()?;

        let mut records = match section_kind {
            1 => PreSwitchTicketV1Records::Permanent(vec![]),
            2 => PreSwitchTicketV1Records::Subscription(vec![]),
            3 => PreSwitchTicketV1Records::Content(vec![]),
            4 => PreSwitchTicketV1Records::ContentConsumption(vec![]),
            5 => PreSwitchTicketV1Records::AccessTitle(vec![]),

            kind => return Err(PreSwitchTicketV1Error::UnknownTicketV1SectionKind(kind)),
        };

        stream.seek_from_pin(section_records_offset.into())?;

        for _ in 0..number_of_records {
            match records {
                PreSwitchTicketV1Records::Permanent(ref mut data) => {
                    let reference_id = PreSwitchTicketV1RefereceId::new(&mut *stream)?;

                    data.push(PreSwitchTicketV1RecordPermanent { reference_id });
                }

                PreSwitchTicketV1Records::Subscription(ref mut data) => {
                    let expiration_time = stream.read_u32::<BE>()?;
                    let reference_id = PreSwitchTicketV1RefereceId::new(&mut *stream)?;

                    data.push(PreSwitchTicketV1RecordSubscription {
                        expiration_time,
                        reference_id,
                    })
                }

                PreSwitchTicketV1Records::Content(ref mut data) => {
                    let offset_content_index = stream.read_u32::<BE>()?;
                    let access_mask = util::read_exact!(stream, 128)?;

                    data.push(PreSwitchTicketV1RecordContent {
                        offset_content_index,
                        access_mask,
                    })
                }

                PreSwitchTicketV1Records::ContentConsumption(ref mut data) => {
                    let content_index = stream.read_u16::<BE>()?;
                    let limit_code = stream.read_u16::<BE>()?;
                    let limit_value = stream.read_u32::<BE>()?;

                    data.push(PreSwitchTicketV1RecordContentConsumption {
                        content_index,
                        limit_code,
                        limit_value,
                    })
                }

                PreSwitchTicketV1Records::AccessTitle(ref mut data) => {
                    let title_id = TitleId::new(stream.read_u64::<BE>()?);
                    let title_mask = stream.read_u64::<BE>()?;

                    data.push(PreSwitchTicketV1RecordAccessTitle {
                        title_id,
                        title_mask,
                    })
                }
            }
        }

        stream.seek(SeekFrom::Start(next_section_position))?;
        Ok(Self { records, flags })
    }
}

/// A reference ID, its meaning and use are still unknown.
// TODO(DISCOVER)
#[derive(Debug)]
pub struct PreSwitchTicketV1RefereceId {
    /// The ID value.
    pub id: [u8; 16],

    /// Attributes attached to the ID.
    pub attributes: u32,
}

impl PreSwitchTicketV1RefereceId {
    fn new<T: Read>(mut stream: T) -> Result<Self, PreSwitchTicketV1Error> {
        let id = util::read_exact!(stream, 16)?;
        let attributes = stream.read_u32::<BE>()?;

        Ok(Self { id, attributes })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_all(&self.id)?;
        stream.write_u32::<BE>(self.attributes)?;

        Ok(())
    }
}

/// A record of kind "permanent", its meaning is still unknown.
// TODO(DISCOVER)
#[derive(Debug)]
pub struct PreSwitchTicketV1RecordPermanent {
    /// The reference ID attach to the record.
    pub reference_id: PreSwitchTicketV1RefereceId,
}

/// A record of kind "subscription", its meaning is still unknown.
#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1RecordSubscription {
    /// The time for the record to expire (measured in UNIX time).
    pub expiration_time: u32,

    /// The reference ID attach to the record.
    pub reference_id: PreSwitchTicketV1RefereceId,
}

/// A record of kind "content", its meaning is still unknown.
#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1RecordContent {
    /// The index offset of the content.
    // TODO(DISCOVER)
    pub offset_content_index: u32,

    /// The access mark.
    // TODO(DISCOVER)
    pub access_mask: [u8; 128],
}

/// A record of kind "content consumption", its meaning is still unknown.
#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1RecordContentConsumption {
    /// The index of the content.
    // TODO(DISCOVER)
    pub content_index: u16,

    /// The kind of limit applied.
    // TODO(DISCOVER)
    pub limit_code: u16,

    /// The value attached to the limit kind.
    // TODO(DISCOVER)
    pub limit_value: u32,
}

/// A record of kind "access title", its meaning is still unknown.
#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1RecordAccessTitle {
    /// The title ID whose access has been given.
    pub title_id: TitleId,

    /// The mask of title IDs.
    pub title_mask: u64,
}
