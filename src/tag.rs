//! OpenType tag type.

/// The tag identifying a font collection (`ttcf`).
pub const TTC_HEADER_TAG: Tag = Tag::new(b"ttcf");

/// An OpenType tag: four bytes identifying a table, script, feature, etc.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag([u8; 4]);

impl Tag {
    /// Creates a `Tag` from a byte array.
    pub const fn new(bytes: &[u8; 4]) -> Self {
        Tag(*bytes)
    }

    /// Creates a `Tag` from big-endian bytes.
    pub const fn from_be_bytes(bytes: [u8; 4]) -> Self {
        Tag(bytes)
    }

    /// Returns the tag as big-endian bytes.
    pub const fn to_be_bytes(self) -> [u8; 4] {
        self.0
    }

    /// Returns a reference to the tag's bytes.
    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

impl core::fmt::Debug for Tag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Tag({self})")
    }
}

impl core::fmt::Display for Tag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for byte in self.0 {
            let c = if byte.is_ascii_graphic() || byte == b' ' {
                byte as char
            } else {
                char::REPLACEMENT_CHARACTER
            };
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl From<[u8; 4]> for Tag {
    fn from(bytes: [u8; 4]) -> Self {
        Tag(bytes)
    }
}

impl From<Tag> for [u8; 4] {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

