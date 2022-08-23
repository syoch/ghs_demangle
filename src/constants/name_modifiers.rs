use once_cell::sync::OnceCell;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Modifier {
    OnPrefix(String),
    OnSuffix(String),
}

static MODIFIERS: OnceCell<HashMap<String, Modifier>> = OnceCell::new();
pub fn get_name_modifiers() -> &'static HashMap<String, Modifier> {
    let name_modifiers = MODIFIERS.get();
    if name_modifiers.is_none() {
        let mut name_modifiers = HashMap::new();

        name_modifiers.insert("U".to_string(), Modifier::OnPrefix("unsigned".to_string()));
        name_modifiers.insert("S".to_string(), Modifier::OnPrefix("signed".to_string()));
        name_modifiers.insert("J".to_string(), Modifier::OnPrefix("__complex".to_string()));
        name_modifiers.insert("M".to_string(), Modifier::OnPrefix("[M]".to_string()));
        name_modifiers.insert("P".to_string(), Modifier::OnPrefix("*".to_string()));
        name_modifiers.insert("R".to_string(), Modifier::OnPrefix("&".to_string()));
        name_modifiers.insert("C".to_string(), Modifier::OnPrefix("const".to_string()));
        name_modifiers.insert("V".to_string(), Modifier::OnPrefix("volatile".to_string()));
        name_modifiers.insert("u".to_string(), Modifier::OnPrefix("restrict".to_string()));

        MODIFIERS.set(name_modifiers).unwrap();
        return get_name_modifiers();
    }
    name_modifiers.unwrap()
}
