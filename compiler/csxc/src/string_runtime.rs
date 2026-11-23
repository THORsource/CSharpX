mod csx_string {
    pub fn substring(value: &str, start: i64, length: Option<i64>) -> Result<String, String> {
        let error = || "ArgumentOutOfRangeException: invalid substring range".to_string();
        let units: Vec<_> = value.encode_utf16().collect();
        let start = usize::try_from(start).map_err(|_| error())?;
        let end = match length {
            Some(length) => start
                .checked_add(usize::try_from(length).map_err(|_| error())?)
                .ok_or_else(error)?,
            None => units.len(),
        };
        let result = units.get(start..end).ok_or_else(error)?;
        String::from_utf16(result)
            .map_err(|_| "CSX1505: substring splits a UTF-16 surrogate pair".to_string())
    }
}
