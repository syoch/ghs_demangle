use once_cell::sync::OnceCell;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Modifier {
    OnPrefix(String),
    OnSuffix(String),
}

static MODIFIERS: OnceCell<HashMap<char, Modifier>> = OnceCell::new();
pub fn get_name_modifiers() -> &'static HashMap<char, Modifier> {
    let name_modifiers = MODIFIERS.get();
    if name_modifiers.is_none() {
        let mut name_modifiers = HashMap::new();

        name_modifiers.insert('U', Modifier::OnPrefix("unsigned".to_string()));
        name_modifiers.insert('S', Modifier::OnPrefix("signed".to_string()));
        name_modifiers.insert('J', Modifier::OnPrefix("__complex".to_string()));
        name_modifiers.insert('M', Modifier::OnPrefix("[M]".to_string()));
        name_modifiers.insert('P', Modifier::OnPrefix("*".to_string()));
        name_modifiers.insert('R', Modifier::OnPrefix("&".to_string()));
        name_modifiers.insert('C', Modifier::OnPrefix("const".to_string()));
        name_modifiers.insert('V', Modifier::OnPrefix("volatile".to_string()));
        name_modifiers.insert('u', Modifier::OnPrefix("restrict".to_string()));

        MODIFIERS.set(name_modifiers).unwrap();
        return get_name_modifiers();
    }
    name_modifiers.unwrap()
}
