// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Implementation of the binary file format used by Nintendo to store title metadata.

use crate::signed_blob_header::{SignedBlobHeader, SignedBlobHeaderError};
use crate::title_id::TitleId;
use byteorder::{BE, LE, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::string::FromUtf8Error;
use thiserror::Error;
use util::{ReadEx, WriteEx};

pub mod content_selector;

use content_selector::{ContentSelector, ContentSelectorMethod};

/// Manifest data regard the title itself, its structure and allowed system access (Also known as
/// `TMD` data).
///
/// Compatible with both versions zero (V0) and one (V1), present on the Nintendo Wii, Wii U
/// DSi and 3DS
///
/// Not compatible with "PackagedContentMeta" (aka CNMT) used on the Nintendo Switch and forward.
#[derive(Debug)]
pub struct TitleMetadata {
    /// Header with data to prove the authenticity that this data
    /// has being created by an authorized entity.
    pub signed_blob_header: SignedBlobHeader,

    /// Version of the
    /// [Certificate revocation list](https://en.wikipedia.org/wiki/Certificate_revocation_list)
    /// used for the Certificate Authority (CA) certificate.
    pub certificate_authority_certificate_revocation_list_version: u8,

    /// Version of the
    /// [Certificate revocation list](https://en.wikipedia.org/wiki/Certificate_revocation_list)
    /// used for the signer certificate.
    pub signer_certificate_revocation_list_version: u8,

    /// Title ID of the title used as "System runtime", its exact meaning depends on the platform
    /// the title is for:
    ///
    /// # iQue NetCard
    /// The product was never released, so its use is unknown.
    ///
    /// # Nintendo Wii
    /// If `Some` it's the title of the IOS to be used for this title, if `None` then the title is
    /// itself an IOS.
    ///
    /// If the title is a boot2 program (title ID: `00000001-00000001`) then this entry will be its
    /// same title ID.
    ///
    /// # Nintendo Wii U
    /// If `Some` it's the title of the IOSU to be used for this title, if `None` then the title is
    /// itself an IOSU.
    ///
    /// # DSi
    /// Given that on the DS games run on the bare hardware this is left unused (`None`).
    ///
    /// # 3DS
    /// Given that the 3DS games run inside a proper OS as a process this is left unused (`None`).
    pub system_runtime_title_id: Option<TitleId>,

    /// Title ID of the title.
    pub title_id: TitleId,

    /// Group ID of the title.
    // TODO(DISCOVER)
    pub group_id: u16,

    /// Bitflags of access right to the hardware, its meaning depends on the platform, the access
    /// to this entry is recommended to use platform aware methods like [Self::has_ppc_access_wii] or [Self::has_dvd_access_wii].
    pub access_rights: u32,

    /// The version of the title.
    pub title_version: u16,

    /// The index value of the content entry where the boot data is located.
    pub boot_content_index: u16,

    /// Platform dependant data.
    pub platform_data: TitleMetadataPlatformData,

    /// Extra data only present on the v1 version of a title metadata.
    pub version_1_extension: Option<TitleMetadataV1>,

    /// Entries to the different content chunks.
    pub content_chunk_entries: Vec<TitleMetadataContentEntry>,
}

impl TitleMetadata {
    /// Create a new installable Wad representation.
    pub fn new<T: Read + Seek>(mut stream: T) -> Result<Self, TitleMetadataError> {
        let signed_blob_header = SignedBlobHeader::new(&mut stream)?;

        let format_version = stream.read_u8()?;

        let certificate_authority_certificate_revocation_list_version = stream.read_u8()?;
        let signer_certificate_revocation_list_version = stream.read_u8()?;

        // On some platforms this byte has a meaning as a bool
        let first_reserved_byte = stream.read_bool()?;

        let system_runtime_title_id = match stream.read_u64::<BE>()? {
            0 => None,
            title_id => Some(TitleId::new(title_id)),
        };

        let title_id = TitleId::new(stream.read_u64::<BE>()?);

        let mut platform_data =
            TitleMetadataPlatformData::new_dummy_from_identifier(stream.read_u32::<BE>()?)?;

        let group_id = stream.read_u16::<BE>()?;

        match platform_data {
            TitleMetadataPlatformData::DSi | TitleMetadataPlatformData::WiiU => {
                stream.seek_relative(62)?;
            }

            TitleMetadataPlatformData::Console3ds {
                ref mut public_save_data_size,
                ref mut private_save_data_size,
                ref mut srl_flag,
            } => {
                *public_save_data_size = stream.read_u32::<LE>()?;
                *private_save_data_size = stream.read_u32::<LE>()?;

                // Skip four unknown bytes
                stream.seek_relative(4)?;

                *srl_flag = stream.read_u8()?;

                // Skip 49 unknown bytes
                stream.seek_relative(49)?;
            }

            TitleMetadataPlatformData::Wii {
                ref mut is_wii_u_vwii_only_title,
                ref mut region,
                ref mut ratings,
                ref mut ipc_mask,
            } => {
                *is_wii_u_vwii_only_title = first_reserved_byte;

                // Skip 2 zeroed bytes
                stream.seek_relative(2)?;

                *region =
                    TitleMetadataPlatformDataWiiRegion::from_identifier(stream.read_u16::<BE>()?)?;

                *ratings = util::read_exact!(stream, 16)?;

                // Skip 12 reserved bytes
                stream.seek_relative(12)?;

                *ipc_mask = util::read_exact!(stream, 12)?;

                // Skip 18 reserved bytes
                stream.seek_relative(18)?;
            }
        }

        let access_rights = stream.read_u32::<BE>()?;
        let title_version = stream.read_u16::<BE>()?;
        let number_of_content_entries = stream.read_u16::<BE>()?;
        let boot_content_index = stream.read_u16::<BE>()?;

        // Skip the title minor version as it was never used
        stream.seek_relative(2)?;

        let version_1_extension = match format_version {
            0 => None,
            1 => Some(TitleMetadataV1::new(&mut stream)?),
            version => return Err(TitleMetadataError::IncompatibleVersion(version)),
        };

        let mut content_chunk_entries = Vec::new();

        for _ in 0..number_of_content_entries {
            content_chunk_entries.push(TitleMetadataContentEntry::new(
                &mut &mut stream,
                version_1_extension.is_some(),
            )?);
        }

        Ok(Self {
            signed_blob_header,
            certificate_authority_certificate_revocation_list_version,
            signer_certificate_revocation_list_version,
            system_runtime_title_id,
            title_id,
            platform_data,
            group_id,
            title_version,
            boot_content_index,
            access_rights,
            version_1_extension,
            content_chunk_entries,
        })
    }

    /// Dump into a stream.
    pub fn dump<T: Write + Seek>(&self, mut stream: T) -> io::Result<()> {
        self.signed_blob_header.dump(&mut stream)?;
        stream.write_bool(self.version_1_extension.is_some())?;
        stream.write_u8(self.certificate_authority_certificate_revocation_list_version)?;
        stream.write_u8(self.signer_certificate_revocation_list_version)?;

        // Weird reserved byte that only has meaning on the Wii
        stream.write_u8(match self.platform_data {
            TitleMetadataPlatformData::DSi
            | TitleMetadataPlatformData::WiiU
            | TitleMetadataPlatformData::Console3ds {
                public_save_data_size: _,
                private_save_data_size: _,
                srl_flag: _,
            } => 0,
            TitleMetadataPlatformData::Wii {
                is_wii_u_vwii_only_title,
                region: _,
                ratings: _,
                ipc_mask: _,
            } => {
                if is_wii_u_vwii_only_title {
                    1
                } else {
                    0
                }
            }
        })?;

        match &self.system_runtime_title_id {
            None => stream.write_zeroed(8)?,
            Some(title_id) => title_id.dump(&mut stream)?,
        };

        self.title_id.dump(&mut stream)?;
        self.platform_data.dump_identifier(&mut stream)?;
        stream.write_u16::<BE>(self.group_id)?;

        match &self.platform_data {
            TitleMetadataPlatformData::DSi | TitleMetadataPlatformData::WiiU => {
                stream.write_zeroed(62)?;
            }

            TitleMetadataPlatformData::Console3ds {
                public_save_data_size,
                private_save_data_size,
                srl_flag,
            } => {
                stream.write_u32::<LE>(*public_save_data_size)?;
                stream.write_u32::<LE>(*private_save_data_size)?;

                // Skip four unknown bytes
                stream.write_zeroed(4)?;

                stream.write_u8(*srl_flag)?;

                // Skip 49 unknown bytes
                stream.write_zeroed(49)?;
            }

            TitleMetadataPlatformData::Wii {
                is_wii_u_vwii_only_title: _,
                region,
                ratings,
                ipc_mask,
            } => {
                stream.write_zeroed(2)?;

                region.dump_identifier(&mut stream)?;

                stream.write_all(ratings)?;
                stream.write_zeroed(12)?;
                stream.write_all(ipc_mask)?;
                stream.write_zeroed(18)?;
            }
        }

        stream.write_u32::<BE>(self.access_rights)?;
        stream.write_u16::<BE>(self.title_version)?;
        stream.write_u16::<BE>(self.content_chunk_entries.len() as u16)?;
        stream.write_u16::<BE>(self.boot_content_index)?;

        // Skip the title minor version as it was never used
        stream.seek_relative(2)?;

        if let Some(version_1_extension) = &self.version_1_extension {
            version_1_extension.dump(&mut stream)?;
        }

        for content_entry in &self.content_chunk_entries {
            content_entry.dump(&mut stream)?;
        }

        Ok(())
    }

    /// If the title has access to the DVD drive. Only on Wii (and Wii U vWii) platform.
    pub fn has_dvd_access_wii(&self) -> Result<bool, TitleMetadataError> {
        if let TitleMetadataPlatformData::Wii {
            is_wii_u_vwii_only_title: _,
            region: _,
            ratings: _,
            ipc_mask: _,
        } = self.platform_data
        {
            return Ok((self.access_rights & 0b10) != 0);
        }

        Err(TitleMetadataError::ActionInvalid())
    }

    /// If the title has access to all hardware from its main PPC chip without using a IOS between
    /// the communication (aka disable the `AHBPROT` protection).
    /// Only on Wii (and Wii U vWii) platform.
    pub fn has_ppc_access_wii(&self) -> Result<bool, TitleMetadataError> {
        if let TitleMetadataPlatformData::Wii {
            is_wii_u_vwii_only_title: _,
            region: _,
            ratings: _,
            ipc_mask: _,
        } = self.platform_data
        {
            return Ok((self.access_rights & 0b1) != 0);
        }

        Err(TitleMetadataError::ActionInvalid())
    }

    /// Get the sizes of the title metadata in bytes.
    pub fn size(&self) -> u32 {
        let num_of_entries = self.content_chunk_entries.len() as u32;

        let mut size = 100 + self.signed_blob_header.size() + 16 * num_of_entries;

        if self.version_1_extension.is_some() {
            // The size of the hash per each content plus the hash of all the content entries
            // groups plus the size of all (64) content entries groups
            size += 32 * num_of_entries + 32 + (4 + 32) * 64
        } else {
            size += 20 * num_of_entries
        }

        size
    }

    /// Select the content with the given physical position.
    pub fn select_with_physical_position(&self, position: usize) -> ContentSelector {
        ContentSelector {
            method: ContentSelectorMethod::WithPhysicalPosition(position),
        }
    }

    /// Select the first content with the given ID.
    pub fn select_with_id(&self, id: u32) -> ContentSelector {
        ContentSelector {
            method: ContentSelectorMethod::WithId(id),
        }
    }

    /// Select the first content with the given index.
    pub fn select_with_index(&self, index: u16) -> ContentSelector {
        ContentSelector {
            method: ContentSelectorMethod::WithIndex(index),
        }
    }

    /// Select the first content stored inside the title (given its physicial position).
    pub fn select_first(&self) -> ContentSelector {
        self.select_with_physical_position(0)
    }

    /// Select the last content stored inside the title (given its physicial position). Be aware
    /// that **this selection is lazy evaluated**.
    pub fn select_last(&self) -> ContentSelector {
        ContentSelector {
            method: ContentSelectorMethod::Last,
        }
    }
}

#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum TitleMetadataError {
    #[error("An IO error has occurred: {0}")]
    IoError(#[from] io::Error),

    #[error("Unable to parse the signed blob header: {0}")]
    SignedBlobHeaderError(#[from] SignedBlobHeaderError),

    #[error("Converting into UTF-8 failed: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Invalid value for the 'is vWii title' flag")]
    InvalidIsVWiiValue(u8),

    #[error("The given title metadata platform is not known: {0}")]
    UnknownPlatform(u32),

    #[error(
        "The given title metadata Nintendo Wii 
        region is not known: {0}"
    )]
    UnknownWiiRegion(u16),

    #[error("The given content entry kind is not known: {0}")]
    UnknownContentEntryKind(u16),

    #[error("The action is invalid for the platform of the title")]
    ActionInvalid(),

    #[error("The version of the title metadata is not compatible (version: {0})")]
    IncompatibleVersion(u8),

    #[error("Content not found")]
    ContentNotFound(),
}

#[derive(Debug)]
/// Data relevant for the platform of the title.
// NOTE: Parsing and dumping of this data is done on the TitleMetadata itself because for some
// reason the data is not sequential and its split along the stream.
pub enum TitleMetadataPlatformData {
    /// The title is for the Nintendo DSi (DSiWare title).
    DSi,

    /// The title is for the Nintendo Wii.
    Wii {
        /// If the title is made to only run on Wii U vWii (The virtual Wii system inside the
        /// Nintendo Wii U).
        is_wii_u_vwii_only_title: bool,

        /// The region of the title
        region: TitleMetadataPlatformDataWiiRegion,

        /// The "ratings" of the title.
        // TODO(DISCOVER)
        ratings: [u8; 16],

        /// The IPC mask of the title.
        // TODO(DISCOVER)
        ipc_mask: [u8; 12],
    },

    /// The title is for the Nintendo 3DS
    Console3ds {
        /// The size of the public save data section.
        public_save_data_size: u32,

        /// The size of the private save data section.
        private_save_data_size: u32,

        /// The SRL flags of the title.
        // TODO(DISCOVER)
        srl_flag: u8,
    },

    /// The title is for the Nintendo Wii U
    WiiU,
}

impl TitleMetadataPlatformData {
    fn new_dummy_from_identifier(identifier: u32) -> Result<Self, TitleMetadataError> {
        match identifier {
            0 => Ok(Self::DSi),
            1 => Ok(Self::Wii {
                is_wii_u_vwii_only_title: false,
                region: TitleMetadataPlatformDataWiiRegion::RegionFree,
                ratings: [0; 16],
                ipc_mask: [0; 12],
            }),
            64 => Ok(Self::Console3ds {
                public_save_data_size: 0,
                private_save_data_size: 0,
                srl_flag: 0,
            }),

            256 => Ok(Self::WiiU),
            identifier => Err(TitleMetadataError::UnknownPlatform(identifier)),
        }
    }

    fn dump_identifier<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u32::<BE>(match self {
            Self::DSi => 0,

            Self::Wii {
                is_wii_u_vwii_only_title: _,
                region: _,
                ratings: _,
                ipc_mask: _,
            } => 1,

            Self::Console3ds {
                public_save_data_size: _,
                private_save_data_size: _,
                srl_flag: _,
            } => 64,

            Self::WiiU => 256,
        })?;

        Ok(())
    }
}

/// The different regions a title can be on a Wii console.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum TitleMetadataPlatformDataWiiRegion {
    Japan,
    USA,
    Europe,
    RegionFree,
    Korea,
}

impl TitleMetadataPlatformDataWiiRegion {
    fn from_identifier(identifier: u16) -> Result<Self, TitleMetadataError> {
        match identifier {
            0 => Ok(Self::Japan),
            1 => Ok(Self::USA),
            2 => Ok(Self::Europe),
            3 => Ok(Self::RegionFree),
            4 => Ok(Self::Korea),

            identifier => Err(TitleMetadataError::UnknownWiiRegion(identifier)),
        }
    }

    fn dump_identifier<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u16::<BE>(match &self {
            Self::Japan => 0,
            Self::USA => 1,
            Self::Europe => 2,
            Self::RegionFree => 3,
            Self::Korea => 4,
        })?;

        Ok(())
    }
}

/// An entry of a content of a title, a content is just a signed
#[derive(Clone, Debug)]
pub struct TitleMetadataContentEntry {
    /// The ID of the content. Unique per title.
    pub id: u32,

    /// The index of the content. Unique per title "bundle" (WAD file, disc image, etc).
    pub index: u16,

    /// The kind of the content.
    pub kind: TitleMetadataContentEntryKind,

    /// The size of the content.
    pub size: u64,

    /// The hash of the content.
    pub hash: TitleMetadataContentEntryHashKind,
}

/// The hash of the content.
#[derive(Clone, Debug)]
pub enum TitleMetadataContentEntryHashKind {
    /// A SHA-1 hash.
    Version0([u8; 20]),

    /// A SHA-256 hash. On Wii U titles this is a SHA-1 hash padded with zeroes.
    Version1([u8; 32]),
}

#[derive(Clone, Copy, Debug)]
/// The kind (behaviour of the content inside the system) of the content.
pub enum TitleMetadataContentEntryKind {
    /// A normal content.
    Normal,

    /// A normal content, present on the Wii U.
    NormalWiiUKind1,

    /// A normal content, present on the Wii U (Stored with a different value in the metadata)
    NormalWiiUKind2,

    /// A normal content, present on the Wii U (Stored with a different value in the metadata)
    NormalWiiUKind3,

    /// A downloadable content for a title.
    Dlc,

    /// A content that can be shared between different title, the system may store then on its
    /// internal memory for reuse.
    Shared,
}

impl TitleMetadataContentEntry {
    fn new<T: Read + Seek>(mut stream: T, version_1: bool) -> Result<Self, TitleMetadataError> {
        let id = stream.read_u32::<BE>()?;
        let index = stream.read_u16::<BE>()?;

        let kind = match stream.read_u16::<BE>()? {
            0x0001 => TitleMetadataContentEntryKind::Normal,
            0x2001 => TitleMetadataContentEntryKind::NormalWiiUKind1,
            0x2003 => TitleMetadataContentEntryKind::NormalWiiUKind2,
            0x6003 => TitleMetadataContentEntryKind::NormalWiiUKind3,
            0x4001 => TitleMetadataContentEntryKind::Dlc,
            0x8001 => TitleMetadataContentEntryKind::Shared,

            identifier => return Err(TitleMetadataError::UnknownContentEntryKind(identifier)),
        };

        let size = stream.read_u64::<BE>()?;
        let hash = if version_1 {
            TitleMetadataContentEntryHashKind::Version1(util::read_exact!(stream, 32)?)
        } else {
            TitleMetadataContentEntryHashKind::Version0(util::read_exact!(stream, 20)?)
        };

        Ok(Self {
            id,
            index,
            kind,
            size,
            hash,
        })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u32::<BE>(self.id)?;
        stream.write_u16::<BE>(self.index)?;

        stream.write_u16::<BE>(match &self.kind {
            TitleMetadataContentEntryKind::Normal => 0x0001,
            TitleMetadataContentEntryKind::NormalWiiUKind1 => 0x2001,
            TitleMetadataContentEntryKind::NormalWiiUKind2 => 0x2003,
            TitleMetadataContentEntryKind::NormalWiiUKind3 => 0x6003,
            TitleMetadataContentEntryKind::Dlc => 0x4001,
            TitleMetadataContentEntryKind::Shared => 0x8001,
        })?;

        stream.write_u64::<BE>(self.size)?;

        match &self.hash {
            TitleMetadataContentEntryHashKind::Version0(value) => stream.write_all(value)?,
            TitleMetadataContentEntryHashKind::Version1(value) => stream.write_all(value)?,
        }

        Ok(())
    }
}

/// The extra data added by the V1 extension of the title metadata.
#[derive(Debug)]
pub struct TitleMetadataV1 {
    /// The hash of all the contents entries groups stored at [Self::content_entries_groups].
    pub content_entries_groups_hash_sha256: [u8; 32],

    /// A set with all the content entries, with all their data default to zero.
    pub content_entries_groups: [TitleMetadataV1ContentEntriesGroup; 64],
}

impl TitleMetadataV1 {
    fn new<T: Read + Seek>(mut stream: T) -> Result<Self, TitleMetadataError> {
        let content_entries_groups_hash_sha256 = util::read_exact!(stream, 32)?;
        let mut content_entries_groups = [TitleMetadataV1ContentEntriesGroup::new_dummy(); 64];

        for group in &mut content_entries_groups {
            *group = TitleMetadataV1ContentEntriesGroup::new(&mut stream)?;
        }

        Ok(Self {
            content_entries_groups_hash_sha256,
            content_entries_groups,
        })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_all(&self.content_entries_groups_hash_sha256)?;

        for content_entry_group in self.content_entries_groups {
            content_entry_group.dump(&mut stream)?;
        }

        Ok(())
    }
}

/// A group of content entries.
#[derive(Copy, Clone, Debug)]
pub struct TitleMetadataV1ContentEntriesGroup {
    /// The index of the first content that is inside the group.
    pub first_content_index: u16,

    /// The number of entries that the group has.
    pub content_entries_in_the_group: u16,

    /// The SHA-256 hash of the content entries stored inside the group.
    pub content_entries_group_hash_sha256: [u8; 32],
}

impl TitleMetadataV1ContentEntriesGroup {
    fn new_dummy() -> Self {
        Self {
            first_content_index: 0,
            content_entries_in_the_group: 0,
            content_entries_group_hash_sha256: [0; 32],
        }
    }

    fn new<T: Read + Seek>(mut stream: T) -> Result<Self, TitleMetadataError> {
        let first_content_index = stream.read_u16::<BE>()?;
        let content_entries_in_the_group = stream.read_u16::<BE>()?;

        let content_entries_group_hash_sha256 = util::read_exact!(stream, 32)?;

        Ok(Self {
            first_content_index,
            content_entries_in_the_group,
            content_entries_group_hash_sha256,
        })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_u16::<BE>(self.first_content_index)?;
        stream.write_u16::<BE>(self.content_entries_in_the_group)?;
        stream.write_all(&self.content_entries_group_hash_sha256)?;

        Ok(())
    }
}
