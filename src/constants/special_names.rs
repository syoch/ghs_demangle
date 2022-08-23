use once_cell::sync::OnceCell;
use std::collections::HashMap;

static SPECIAL_NAMES: OnceCell<HashMap<String, String>> = OnceCell::new();

pub fn get_special_names() -> &'static HashMap<String, String> {
    let special_names = SPECIAL_NAMES.get();
    if special_names.is_none() {
        let mut special_names = HashMap::new();
        special_names.insert("__ct".to_string(), "#".to_string());
        special_names.insert("__vtbl".to_string(), "virtual table".to_string());
        special_names.insert("__dt".to_string(), "~#".to_string());
        special_names.insert("__as".to_string(), "operator=".to_string());
        special_names.insert("__eq".to_string(), "operator==".to_string());
        special_names.insert("__ne".to_string(), "operator!=".to_string());
        special_names.insert("__gt".to_string(), "operator>".to_string());
        special_names.insert("__lt".to_string(), "operator<".to_string());
        special_names.insert("__ge".to_string(), "operator>=".to_string());
        special_names.insert("__le".to_string(), "operator<=".to_string());
        special_names.insert("__pp".to_string(), "operator++".to_string());
        special_names.insert("__pl".to_string(), "operator+".to_string());
        special_names.insert("__apl".to_string(), "operator+=".to_string());
        special_names.insert("__mi".to_string(), "operator-".to_string());
        special_names.insert("__ami".to_string(), "operator-=".to_string());
        special_names.insert("__ml".to_string(), "operator*".to_string());
        special_names.insert("__amu".to_string(), "operator*=".to_string());
        special_names.insert("__dv".to_string(), "operator/".to_string());
        special_names.insert("__adv".to_string(), "operator/=".to_string());
        special_names.insert("__nw".to_string(), "operator new".to_string());
        special_names.insert("__dl".to_string(), "operator delete".to_string());
        special_names.insert("__vn".to_string(), "operator new[]".to_string());
        special_names.insert("__vd".to_string(), "operator delete[]".to_string());
        special_names.insert("__md".to_string(), "operator%".to_string());
        special_names.insert("__amd".to_string(), "operator%=".to_string());
        special_names.insert("__mm".to_string(), "operator--".to_string());
        special_names.insert("__aa".to_string(), "operator&&".to_string());
        special_names.insert("__oo".to_string(), "operator||".to_string());
        special_names.insert("__or".to_string(), "operator|".to_string());
        special_names.insert("__aor".to_string(), "operator|=".to_string());
        special_names.insert("__er".to_string(), "operator^".to_string());
        special_names.insert("__aer".to_string(), "operator^=".to_string());
        special_names.insert("__ad".to_string(), "operator&".to_string());
        special_names.insert("__aad".to_string(), "operator&=".to_string());
        special_names.insert("__co".to_string(), "operator~".to_string());
        special_names.insert("__cl".to_string(), "operator".to_string());
        special_names.insert("__ls".to_string(), "operator<<".to_string());
        special_names.insert("__als".to_string(), "operator<<=".to_string());
        special_names.insert("__rs".to_string(), "operator>>".to_string());
        special_names.insert("__ars".to_string(), "operator>>=".to_string());
        special_names.insert("__rf".to_string(), "operator->".to_string());
        special_names.insert("__vc".to_string(), "operator[]".to_string());
        SPECIAL_NAMES.set(special_names).unwrap();
        return get_special_names();
    }
    special_names.unwrap()
}
