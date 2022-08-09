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

        Ok(values)
    }

    fn read_2bits(&mut self, k: usize) -> error::Result<Vec<u8>> {
        let mut values = self.read_n_bytes_dyn(bytes2store_k(k as u64) as usize)?;

        values.iter_mut().for_each(|v| *v = u8::from_be(*v));

        Ok(values)
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

        assert_eq!(&values, b"Lorem ipsum dolor\0");

        reader.seek(std::io::SeekFrom::Start(values.len() as u64))?;
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
}
