use once_cell::sync::OnceCell;
use std::collections::HashMap;

static BASE_TYPES: OnceCell<HashMap<char, String>> = OnceCell::new();

pub fn get_base_types() -> &'static HashMap<char, String> {
    let base_types = BASE_TYPES.get();
    if base_types.is_none() {
        let mut base_types = HashMap::new();

        base_types.insert('v', "void".to_string());
        base_types.insert('i', "int".to_string());
        base_types.insert('s', "short".to_string());
        base_types.insert('c', "char".to_string());
        base_types.insert('w', "wchar_t".to_string());
        base_types.insert('b', "bool".to_string());
        base_types.insert('f', "float".to_string());
        base_types.insert('d', "double".to_string());
        base_types.insert('l', "long".to_string());
        base_types.insert('L', "long long".to_string());
        base_types.insert('e', "...".to_string());
        base_types.insert('r', "long double".to_string());
        BASE_TYPES.set(base_types).unwrap();
        return get_base_types();
    }
    base_types.unwrap()
}
