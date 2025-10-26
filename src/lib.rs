//! Micro-crate for OpenType container parsing

extern crate alloc;

use font_types::{TTC_HEADER_TAG, Tag};
use parser::{FromSlice, LazyArray16, LazyArray32, Stream, TryFromBeBytes, round4};

#[doc(inline)]
pub use font_types as types;
pub mod parser;

pub struct FontFile<'a> {
    data: &'a [u8],
}

impl FontFile<'_> {
    pub fn new<'b>(data: &'b [u8]) -> FontFile<'b> {
        FontFile { data }
    }

    pub fn tag(&self) -> Option<Tag> {
        Tag::try_parse_from_be_bytes(&self.data[0..4])
    }

    pub fn is_collection(&self) -> bool {
        self.tag() == Some(TTC_HEADER_TAG)
    }

    pub fn into_collection(&self) -> Option<Collection<'_>> {
        Collection::new(self.data)
    }
}

pub struct Font<'a> {
    directory: TableDirectory<'a>,
    data: &'a [u8],
}

impl Font<'_> {
    pub fn copy_data(&self) -> Option<Box<[u8]>> {
        // Allocate space
        let mut out = Vec::new();
        out.reserve_exact(self.directory.font_data_size());

        // Copy TableDirectory
        self.directory.write_to_vec(&mut out);

        // Copy tables
        for (i, table) in self.directory.table_records_data.into_iter().enumerate() {
            // Save dest offset
            let dest_offset = out.len() as u32;

            // Copy data
            let src_offset = table.offset as usize;
            let src_end = round4(src_offset + table.length as usize);
            let table_data = self.data.get(src_offset..src_end)?;
            out.extend_from_slice(table_data);

            // Overwrite offset in header
            let header_offset = 12 + 8 + (i * 16);
            out[header_offset..(header_offset + 4)].copy_from_slice(&dest_offset.to_be_bytes());
        }

        Some(out.into_boxed_slice())
    }
}

pub struct Collection<'a> {
    pub header: CollectionHeader<'a>,
    pub data: &'a [u8],
}

impl Collection<'_> {
    pub fn new<'b>(data: &'b [u8]) -> Option<Collection<'b>> {
        Some(Collection {
            data,
            header: CollectionHeader::parse(data)?,
        })
    }

    pub fn get_font(&self, index: u32) -> Option<Font<'_>> {
        let directory_offset = self.header.table_directory_offsets.get(index)? as usize;
        let directory_data = self.data.get(directory_offset..)?;
        let directory = TableDirectory::parse(directory_data)?;

        Some(Font {
            directory,
            data: self.data,
        })
    }
}

pub struct CollectionHeader<'a> {
    pub tag: Tag,
    pub major_version: u16,
    pub minor_version: u16,
    pub num_fonts: u32,
    pub table_directory_offsets: LazyArray32<'a, u32>,
    pub dsig_tag: u32,
    pub dsig_length: u32,
    pub dsig_offset: u32,
}

impl<'a> FromSlice<'a> for CollectionHeader<'a> {
    fn parse(data: &[u8]) -> Option<CollectionHeader<'_>> {
        let mut stream = Stream::new(data);

        let tag = stream.read::<Tag>()?;
        if tag != TTC_HEADER_TAG {
            return None;
        }

        let major_version = stream.read::<u16>()?;
        let minor_version = stream.read::<u16>()?;
        let num_fonts = stream.read::<u32>()?;

        let table_directory_offsets = stream.read_array32::<u32>(num_fonts)?;

        let mut dsig_tag = 0;
        let mut dsig_length = 0;
        let mut dsig_offset = 0;
        if major_version == 2 {
            dsig_tag = stream.read::<u32>()?;
            dsig_length = stream.read::<u32>()?;
            dsig_offset = stream.read::<u32>()?;
        }

        Some(CollectionHeader {
            tag,
            major_version,
            minor_version,
            num_fonts,
            table_directory_offsets,
            dsig_tag,
            dsig_length,
            dsig_offset,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TableDirectory<'a> {
    pub sfnt_version: u32,
    pub num_tables: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
    pub table_records_data: LazyArray16<'a, TableDirectoryRecord>,
}

impl TableDirectory<'_> {
    pub fn directory_size(&self) -> usize {
        12 + (self.table_records_data.len() as usize * 16)
    }

    pub fn font_data_size(&self) -> usize {
        self.directory_size()
            + self
                .table_records_data
                .into_iter()
                .map(|record| round4(record.length as usize))
                .sum::<usize>()
    }

    pub fn write_to_vec(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.sfnt_version.to_be_bytes());
        vec.extend_from_slice(&self.num_tables.to_be_bytes());
        vec.extend_from_slice(&self.search_range.to_be_bytes());
        vec.extend_from_slice(&self.entry_selector.to_be_bytes());
        vec.extend_from_slice(&self.range_shift.to_be_bytes());
        vec.extend_from_slice(self.table_records_data.bytes());
    }
}

impl<'a> FromSlice<'a> for TableDirectory<'a> {
    fn parse(data: &[u8]) -> Option<TableDirectory<'_>> {
        let mut stream = Stream::new(data);

        let sfnt_version = stream.read::<u32>()?;
        let num_tables = stream.read::<u16>()?;
        let search_range = stream.read::<u16>()?;
        let entry_selector = stream.read::<u16>()?;
        let range_shift = stream.read::<u16>()?;
        let table_records_data = stream.read_array16::<TableDirectoryRecord>(num_tables)?;

        Some(TableDirectory {
            sfnt_version,
            num_tables,
            search_range,
            entry_selector,
            range_shift,
            table_records_data,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TableDirectoryRecord {
    pub tag: Tag,
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

impl TryFromBeBytes for TableDirectoryRecord {
    const SIZE: usize = 16;

    fn try_parse_from_be_bytes(data: &[u8]) -> Option<Self> {
        let mut stream = Stream::new(data);
        Some(TableDirectoryRecord {
            tag: stream.read::<Tag>()?,
            checksum: stream.read::<u32>()?,
            offset: stream.read::<u32>()?,
            length: stream.read::<u32>()?,
        })
    }
}
