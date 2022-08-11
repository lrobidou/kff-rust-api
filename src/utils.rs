//! Utils function for KFF

/* std use */

/* crate use */

/* project use */
use crate::error;

#[inline]
pub(crate) fn ceil_to_8(n: u64) -> u64 {
    (n + 7) & !(7)
}

#[inline]
pub(crate) fn bits2store_k(k: u64) -> u64 {
    k * 2
}

#[inline]
pub(crate) fn bytes2store_k(k: u64) -> u64 {
    ceil_to_8(bits2store_k(k)) / 8
}

/// Define trait containts utils function to parsing kff
pub trait KffRead {
    /// Function read N bytes (N define at compile time) in a readable
    fn read_n_bytes<const N: usize>(&mut self) -> error::Result<[u8; N]>;

    /// Function read N bytes (N define at run time) in a readable
    fn read_n_bytes_dyn(&mut self, n: usize) -> error::Result<Vec<u8>>;

    /// Function read a Kff 'ascii'
    fn read_ascii(&mut self) -> error::Result<Vec<u8>>;

    /// Function some base in 2bits representation
    fn read_2bits(&mut self, k: usize) -> error::Result<Vec<u8>>;

    /// Function that read one bit and convert it as bool
    fn read_bool(&mut self) -> error::Result<bool> {
        self.read_u8().map(|x| x != 0)
    }

    /// Function that read u8
    fn read_u8(&mut self) -> error::Result<u8> {
        self.read_n_bytes::<1>()
            .map(|x| unsafe { *x.get_unchecked(0) })
    }

    /// Function that read u16
    fn read_u16(&mut self) -> error::Result<u16> {
        self.read_n_bytes::<2>().map(u16::from_be_bytes)
    }

    /// Function that read u32
    fn read_u32(&mut self) -> error::Result<u32> {
        self.read_n_bytes::<4>().map(u32::from_be_bytes)
    }

    /// Function that read u64
    fn read_u64(&mut self) -> error::Result<u64> {
        self.read_n_bytes::<8>().map(u64::from_be_bytes)
    }
}

impl<T> KffRead for T
where
    T: std::io::BufRead,
{
    fn read_n_bytes<const N: usize>(&mut self) -> error::Result<[u8; N]> {
        let mut values = [0; N];

        self.read_exact(&mut values)?;

        Ok(values)
    }

    fn read_n_bytes_dyn(&mut self, n: usize) -> error::Result<Vec<u8>> {
        let mut values = vec![0; n];

        self.read_exact(&mut values)?;

        Ok(values)
    }

    fn read_ascii(&mut self) -> error::Result<Vec<u8>> {
        let mut values = Vec::with_capacity(50);

        self.read_until(0, &mut values)?;

        if let Some(0) = values.last() {
            values.pop();
        }

        Ok(values)
    }

    fn read_2bits(&mut self, k: usize) -> error::Result<Vec<u8>> {
        let mut values = self.read_n_bytes_dyn(bytes2store_k(k as u64) as usize)?;

        values.iter_mut().for_each(|v| *v = u8::from_be(*v));

        Ok(values)
    }
}

/// Define trait containts utils function to write kff
pub trait KffWrite {
    /// Function that write all bytes
    fn write_bytes(&mut self, bytes: &[u8]) -> error::Result<()>;

    /// Function that write bytes plus a '\0' at end
    fn write_ascii(&mut self, ascii: &[u8]) -> error::Result<()> {
        self.write_bytes(ascii)?;
        self.write_bytes(&[b'\0'])
    }

    /// Function that write one bit and convert it as bool
    fn write_bool(&mut self, value: &bool) -> error::Result<()> {
        self.write_bytes(&((*value as u8).to_be_bytes()))
    }

    /// Function that write u8
    fn write_u8(&mut self, value: &u8) -> error::Result<()> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Function that write u16
    fn write_u16(&mut self, value: &u16) -> error::Result<()> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Function that write u32
    fn write_u32(&mut self, value: &u32) -> error::Result<()> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Function that write u64
    fn write_u64(&mut self, value: &u64) -> error::Result<()> {
        self.write_bytes(&value.to_be_bytes())
    }
}

impl<T> KffWrite for T
where
    T: std::io::Write,
{
    fn write_bytes(&mut self, bytes: &[u8]) -> error::Result<()> {
        self.write_all(bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::distributions::Distribution;

    #[test]
    fn ceil_to_8_() -> error::Result<()> {
        assert_eq!(ceil_to_8(1), 8);
        assert_eq!(ceil_to_8(7), 8);

        assert_eq!(ceil_to_8(8), 8);

        assert_eq!(ceil_to_8(9), 16);
        assert_eq!(ceil_to_8(15), 16);

        Ok(())
    }

    #[test]
    fn bits2store_k_() -> error::Result<()> {
        let range = rand::distributions::Uniform::from(0..100);
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let value = range.sample(&mut rng);

            assert_eq!(bits2store_k(value), value * 2);
        }

        Ok(())
    }

    #[test]
    fn bytes2store_k_() -> error::Result<()> {
        assert_eq!(bytes2store_k(1), 1);
        assert_eq!(bytes2store_k(4), 1);
        assert_eq!(bytes2store_k(5), 2);
        assert_eq!(bytes2store_k(16), 4);
        assert_eq!(bytes2store_k(17), 5);

        Ok(())
    }

    use std::io::Seek;

    const LOREM: &[u8] = b"Lorem ipsum dolor\0sit amet, consectetur adipiscing elit.";

    #[test]
    fn read_n_bytes() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        let values = reader.read_n_bytes::<11>()?;

        assert_eq!(&values, b"Lorem ipsum");

        let values = reader.read_n_bytes::<11>()?;

        assert_eq!(&values, b" dolor\0sit ");

        let values = reader.read_n_bytes::<400>();

        assert!(values.is_err());

        Ok(())
    }

    #[test]
    fn read_n_bytes_dyn() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        let values = reader.read_n_bytes_dyn(11)?;

        assert_eq!(&values, b"Lorem ipsum");

        let values = reader.read_n_bytes_dyn(11)?;

        assert_eq!(&values, b" dolor\0sit ");

        let values = reader.read_n_bytes_dyn(400);

        assert!(values.is_err());

        Ok(())
    }

    #[test]
    fn read_ascii() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        let values = reader.read_ascii()?;

        assert_eq!(&values, b"Lorem ipsum dolor");

        reader.seek(std::io::SeekFrom::Start(values.len() as u64 + 1))?; // Move after first \0
        let values = reader.read_ascii()?;

        assert_eq!(&values, b"sit amet, consectetur adipiscing elit.");

        reader.seek(std::io::SeekFrom::End(0))?;
        let values = reader.read_ascii()?;

        assert_eq!(&values, b"");

        Ok(())
    }

    #[test]
    fn read_2bits() -> error::Result<()> {
        let mut reader = std::io::Cursor::new([0b11101110, 0b00010001]);

        let kmer = reader.read_2bits(5)?;

        assert_eq!(&kmer, &[238, 17]);

        Ok(())
    }

    #[test]
    fn read_bool() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        assert!(reader.read_bool()?);

        let _ = reader.read_n_bytes::<16>()?;

        assert!(!reader.read_bool()?);

        Ok(())
    }

    #[test]
    fn read_u8() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        assert_eq!(reader.read_u8()?, b'L');
        assert_eq!(reader.read_u8()?, b'o');

        Ok(())
    }

    #[test]
    fn read_u16() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        assert_eq!(reader.read_u16()?, 19567);
        assert_eq!(reader.read_u16()?, 29285);

        Ok(())
    }

    #[test]
    fn read_u32() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        assert_eq!(reader.read_u32()?, 1282372197);
        assert_eq!(reader.read_u32()?, 1830840688);

        Ok(())
    }

    #[test]
    fn read_u64() -> error::Result<()> {
        let mut reader = std::io::Cursor::new(LOREM);

        assert_eq!(reader.read_u64()?, 5507746649245510000);
        assert_eq!(reader.read_u64()?, 8319675872528264303);

        Ok(())
    }

    #[test]
    fn write_bytes() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_bytes(b"Lorem")?;

        assert_eq!(writer, b"Lorem");

        Ok(())
    }

    #[test]
    fn write_ascii() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_ascii(b"Lorem")?;

        assert_eq!(writer, &[b'L', b'o', b'r', b'e', b'm', 0]);

        Ok(())
    }

    #[test]
    fn write_bool() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_bool(&true)?;
        writer.write_bool(&false)?;

        assert_eq!(writer, &[1, 0]);

        Ok(())
    }

    #[test]
    fn write_u8() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_u8(&1)?;
        writer.write_u8(&128)?;
        writer.write_u8(&130)?;

        assert_eq!(writer, &[1, 128, 130]);

        Ok(())
    }

    #[test]
    fn write_u16() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_u16(&250)?;
        writer.write_u16(&500)?;

        assert_eq!(writer, &[0, 250, 1, 244]);

        Ok(())
    }

    #[test]
    fn write_u32() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_u32(&3511)?;
        writer.write_u32(&511110)?;

        assert_eq!(writer, &[0, 0, 13, 183, 0, 7, 204, 134]);

        Ok(())
    }

    #[test]
    fn write_u64() -> error::Result<()> {
        let mut writer = Vec::new();

        writer.write_u64(&35191823831)?;
        writer.write_u64(&444799335191823831)?;

        assert_eq!(
            writer,
            &[0, 0, 0, 8, 49, 152, 157, 215, 6, 44, 62, 163, 130, 112, 69, 215]
        );

        Ok(())
    }
}
