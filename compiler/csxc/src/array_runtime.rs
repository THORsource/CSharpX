#[allow(dead_code)]
mod csx_array {
    use std::{cell::RefCell, rc::Rc};

    type Array<T> = Rc<RefCell<Vec<T>>>;

    pub fn from_vec<T>(data: Vec<T>) -> Array<T> {
        Rc::new(RefCell::new(data))
    }

    pub fn filled<T: Clone>(length: i64, value: T) -> Result<Array<T>, String> {
        let length = i32::try_from(length)
            .ok()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| "OverflowException: invalid array length".to_string())?;
        let mut data = Vec::new();
        data.try_reserve_exact(length)
            .map_err(|_| "OutOfMemoryException: array allocation failed".to_string())?;
        data.resize(length, value);
        Ok(from_vec(data))
    }

    fn bounds() -> String {
        "IndexOutOfRangeException: invalid array index".into()
    }

    pub fn get<T: Clone>(array: &Array<T>, index: i64) -> Result<T, String> {
        let index = usize::try_from(index).map_err(|_| bounds())?;
        array.borrow().get(index).cloned().ok_or_else(bounds)
    }

    pub fn set<T>(array: &Array<T>, index: i64, value: T) -> Result<(), String> {
        let index = usize::try_from(index).map_err(|_| bounds())?;
        let mut data = array.borrow_mut();
        *data.get_mut(index).ok_or_else(bounds)? = value;
        Ok(())
    }
}
