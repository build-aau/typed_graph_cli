use crate::GenResult;

pub trait Process<Meta> {
    fn process(&self, meta: &Meta) -> GenResult<()>;
}

pub fn hex_u64(s: &str) -> Result<u64, String> {
    const PREFIX: &str = "#";
    const PREFIX_LEN: usize = PREFIX.len();

    let result = u64::from_str_radix(&s[PREFIX_LEN..], 16);

    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("{}", e)),
    }
}