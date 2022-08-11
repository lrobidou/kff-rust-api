//! Parse and access to Header information

/* std use */

/* crate use */

/* project use */
use crate::error;

/// Struct to parse and store Header information
#[derive(std::fmt::Debug)]
pub struct Header {
    major_version: u8,
    minor_version: u8,
    encoding: u8,
    uniq_kmer: bool,
    canonical_kmer: bool,
    free_block: Vec<u8>,
}

impl Header {
    /// Constructor of header
    pub fn new(
        major_version: u8,
        minor_version: u8,
        encoding: u8,
        uniq_kmer: bool,
        canonical_kmer: bool,
        free_block: Vec<u8>,
    ) -> error::Result<Self> {
        let obj = Self {
            major_version,
            minor_version,
            encoding,
            uniq_kmer,
            canonical_kmer,
            free_block,
        };

        obj.check()
    }

    /// Read a readable to create a new header
    pub fn read<R>(inner: &mut R) -> error::Result<Self>
    where
        R: std::io::Read + crate::KffRead,
    {
        let mut obj = Self {
            major_version: 0,
            minor_version: 0,
            encoding: 0b00101110,
            uniq_kmer: false,
            canonical_kmer: false,
            free_block: Vec::new(),
        };

        let magic_number = inner.read_n_bytes::<3>()?;
        if &magic_number != b"KFF" {
            return Err(error::Kff::MissingMagic("start".to_string()).into());
        }

        obj.major_version = inner.read_u8()?;
        obj.minor_version = inner.read_u8()?;
        obj.encoding = inner.read_u8()?;
        obj.uniq_kmer = inner.read_bool()?;
        obj.canonical_kmer = inner.read_bool()?;

        let free_block_size = inner.read_u32()? as usize;
        println!("free block size {}", free_block_size);
        obj.free_block = inner.read_n_bytes_dyn(free_block_size)?;

        obj.check()
    }

    /// Function run after construction of header to check value
    fn check(self) -> error::Result<Self> {
        self.check_version()?.check_encoding()
    }

    fn check_version(self) -> error::Result<Self> {
        if self.major_version > 1 {
            return Err(error::Kff::HighMajorVersionNumber(self.major_version).into());
        }

        if self.minor_version > 0 {
            return Err(error::Kff::HighMinorVersionNumber(self.minor_version).into());
        }

        Ok(self)
    }

    fn check_encoding(self) -> error::Result<Self> {
        let a = self.encoding >> 6;
        let c = (self.encoding >> 4) & 0b11;
        let t = (self.encoding >> 2) & 0b11;
        let g = self.encoding & 0b11;

        if a != c && a != t && a != g && c != t && t != g {
            Ok(self)
        } else {
            Err(error::Kff::BadEncoding(self.encoding).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &[u8] = &[
        b'K', b'F', b'F', 1, 0, 0b00101110, 1, 0, 0, 0, 0, 4, b't', b'e', b's', b't',
    ];

    const BAD_MAGIC_NUMBER: &[u8] = b"KKF";

    #[test]
    fn new() -> error::Result<()> {
        assert!(Header::new(1, 0, 0b00101110, true, false, b"test".to_vec()).is_ok());

        assert!(Header::new(2, 0, 0b00101110, true, false, b"test".to_vec()).is_err());
        assert!(Header::new(1, 1, 0b00101110, true, false, b"test".to_vec()).is_err());
        assert!(Header::new(1, 0, 0b11111111, true, false, b"test".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn read() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(VALID);

        assert!(Header::read(&mut reader).is_ok());

        let mut reader = std::io::Cursor::new(BAD_MAGIC_NUMBER);

        assert!(Header::read(&mut reader).is_err());

        Ok(())
    }
}
