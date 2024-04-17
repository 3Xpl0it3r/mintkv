use std::fs::File;
use std::io::Read;

// Random[#TODO] (shoule add some comments )
pub(crate) struct Random;

// Random[#TODO] (should add some comments)
impl Random {
    pub(crate) fn int32() -> Result<i32, std::io::Error> {
        let mut file = File::open("/dev/random")?;
        let mut random_bytes = [0u8; 4];
        file.read_exact(&mut random_bytes)?;
        let random_integer = i32::from_be_bytes(random_bytes);
        Ok(random_integer)
    }

    pub(crate) fn u32() -> Result<u32, std::io::Error> {
        let mut file = File::open("/dev/random")?;
        let mut random_bytes = [0u8; 4];
        file.read_exact(&mut random_bytes)?;
        let random_integer = u32::from_be_bytes(random_bytes);
        Ok(random_integer)
    }

    pub(crate) fn f64() -> Result<f64, std::io::Error> {
        let mut file = File::open("/dev/random")?;
        let mut random_bytes = [0u8; 8];
        file.read_exact(&mut random_bytes)?;
        let random_integer = f64::from_be_bytes(random_bytes);
        Ok(random_integer)
    }

    pub(crate) fn u64() -> Result<u64, std::io::Error> {
        let mut file = File::open("/dev/random")?;
        let mut random_bytes = [0u8; 8];
        file.read_exact(&mut random_bytes)?;
        let random_integer = u64::from_be_bytes(random_bytes);
        Ok(random_integer)
    }
}
