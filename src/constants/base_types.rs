use once_cell::sync::OnceCell;
use std::collections::HashMap;

static BASE_TYPES: OnceCell<HashMap<String, String>> = OnceCell::new();

pub fn get_base_types() -> &'static HashMap<String, String> {
    let base_types = BASE_TYPES.get();
    if base_types.is_none() {
        let mut base_types = HashMap::new();

        base_types.insert('v'.to_string(), "void".to_string());
        base_types.insert('i'.to_string(), "int".to_string());
        base_types.insert('s'.to_string(), "short".to_string());
        base_types.insert('c'.to_string(), "char".to_string());
        base_types.insert('w'.to_string(), "wchar_t".to_string());
        base_types.insert('b'.to_string(), "bool".to_string());
        base_types.insert('f'.to_string(), "float".to_string());
        base_types.insert('d'.to_string(), "double".to_string());
        base_types.insert('l'.to_string(), "long".to_string());
        base_types.insert('L'.to_string(), "long long".to_string());
        base_types.insert('e'.to_string(), "...".to_string());
        base_types.insert('r'.to_string(), "long double".to_string());
        BASE_TYPES.set(base_types).unwrap();
        return get_base_types();
    }
    base_types.unwrap()
}
