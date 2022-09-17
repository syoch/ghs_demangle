use std::fs;

use constants::{get_base_types, get_name_modifiers, Modifier};
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take},
    character::complete::{digit1, one_of},
    combinator::opt,
    multi::{count, many0},
    sequence::{delimited, preceded, terminated},
    Parser,
};
use wasm_bindgen::prelude::wasm_bindgen;

mod constants;

#[derive(Debug, Clone)]
pub enum Name {
    Identifier(String),                    // <String>
    BaseType(char),                        // {base_types::get_base_types().keys |> one_of}
    WithArguments(Box<Name>, Vec<Name>),   // <Name>[C][S]F<Names>
    Template(Box<Name>, Vec<Name>),        // __tm__<UnderString=Names>
    Modifier(Modifier, Box<Name>),         // <Modifier><Name>
    Namespace(Vec<Name>),                  // Q<n>_[String; n]
    InName(Box<Name>, Box<Name>),          // <Name>__<Name>
    WithReturnValue(Box<Name>, Box<Name>), // <Name>_<Name>
    FunctionPointer(Vec<Name>, Box<Name>), // F<Names>_<Name>
    ValueArgument(Box<Name>, String),
    /*
        X[
            (<String> -> value, "typename" -> type) |
            (<Name> -> type, L_<UnderString> -> value)
        ]
    */
    SizedArray(usize, Box<Name>), // A[integer -> size]_[<Name> -> type]
    Names_Ref(usize),
    Names_Multi(usize, usize),
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(x) => write!(f, "{x}"),
            Self::BaseType(x) => write!(f, "{}", get_base_types()[x]),
            Self::WithArguments(base, args) => write!(
                f,
                "{}({})",
                base,
                args.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Template(base, args) => write!(
                f,
                "{}<{}>",
                base,
                args.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Modifier(x, y) => match x {
                Modifier::OnPrefix(s) => write!(f, "{} {}", s, y),
                Modifier::OnSuffix(s) => write!(f, "{} {}", y, s),
            },
            Self::Namespace(x) => write!(
                f,
                "{}",
                x.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            ),
            Self::InName(leaf, parent) => write!(f, "{}::{}", parent, leaf),
            Self::WithReturnValue(base, ret) => write!(f, "{} {}", ret, base),
            Self::FunctionPointer(args, ret) => {
                write!(
                    f,
                    "({} *({}))",
                    ret,
                    args.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::ValueArgument(ty, val) => write!(f, "{val} as {ty}"),
            Self::SizedArray(size, ty) => write!(f, "{ty}[{size}]"),
            Self::Names_Ref(x) => write!(f, "<NameRef {}>", x),
            Self::Names_Multi(x, y) => write!(f, "<NameRepeat {} times of {}>", y, x),
        }
    }
}

impl Name {
    fn identifier_from_string(ident: String) -> Name {
        Name::Identifier(ident)
    }
    fn Identifier_from_str(ident: &str) -> Name {
        Name::Identifier(ident.to_string())
    }
}

fn read_name_identifier(input: &str) -> nom::IResult<&str, Name> {
    let (input, length) = digit1(input)?;
    let length = length.parse::<usize>().unwrap();
    let (input, ident) = take(length)(input)?;

    Ok((input, Name::Identifier(ident.to_string())))
}

fn extract_string(input: &str) -> nom::IResult<&str, &str> {
    let (input, length) = digit1(input)?;
    let length = length.parse::<usize>().unwrap();
    let (input, string) = take(length)(input)?;

    Ok((input, string))
}
fn extract_string_with_under_bar(input: &str) -> nom::IResult<&str, &str> {
    let (input, length) = digit1(input)?;
    let length = length.parse::<usize>().unwrap();

    let (input, _) = tag("_")(input)?;
    let (input, string) = take(length)(input)?;

    Ok((input, string))
}

fn read_modifier(input: &str) -> nom::IResult<&str, Name> {
    let modifiers = get_name_modifiers()
        .keys()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("");

    let (input, modifier) = one_of(modifiers.as_str())(input)?;
    let modifier = get_name_modifiers().get(&modifier).unwrap().clone();

    let (input, name) = read_name(input)?;

    Ok((input, Name::Modifier(modifier, Box::new(name))))
}

fn value_argument(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("X")(input)?;
    alt((
        extract_string.map(|x| {
            Name::ValueArgument(
                Box::new(Name::Identifier_from_str("typename")),
                x.to_string(),
            )
        }),
        permutation((read_name, tag("L_"), extract_string_with_under_bar))
            .map(|(t, _, v)| Name::ValueArgument(Box::new(t), v.to_string())),
    ))
    .parse(input)
}
fn sized_array(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("A")(input)?;
    let (input, size) = digit1(input)?;
    let size = size.parse::<usize>().unwrap();
    let (input, _) = tag("_")(input)?;
    let (input, t) = read_name(input)?;

    Ok((input, Name::SizedArray(size, Box::new(t))))
}
fn type_ref(input: &str) -> nom::IResult<&str, Name> {
    // TODO: Z\d_\dZ
    let (input, t) = delimited(
        tag("Z"),
        terminated(digit1, opt(permutation((tag("_"), digit1)))),
        tag("Z"),
    )(input)?;
    let t = t.parse::<usize>().unwrap();

    Ok((input, Name::Identifier(t.to_string())))
}

fn read_name_ref(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("T")(input)?;
    let (input, index) = one_of("0123456789")(input)?;

    Ok((
        input,
        Name::Names_Ref(index.to_string().parse::<usize>().unwrap()),
    ))
}

fn read_name_repeat(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("N")(input)?;
    let (input, index) = one_of("0123456789")(input)?;
    let (input, index2) = one_of("0123456789")(input)?;
    let index = index.to_string().parse::<usize>().unwrap();
    let index2 = index2.to_string().parse::<usize>().unwrap();
    Ok((input, Name::Names_Multi(index, index2)))
}

fn read_names(input: &str) -> nom::IResult<&str, Vec<Name>> {
    let (input, names) = many0(alt((
        read_name,
        read_name_ref,
        read_name_repeat,
        value_argument,
    )))(input)?;

    let mut ret: Vec<Name> = Vec::new();

    for name in names {
        match name {
            Name::Names_Ref(index) => ret.push(ret[index - 1].clone()),
            Name::Names_Multi(count, index) => {
                for _ in 1..=count {
                    ret.push(ret[index - 1].clone());
                }
            }
            _ => ret.push(name),
        }
    }

    Ok((input, ret))
}

fn arguments(input: &str) -> nom::IResult<&str, Vec<Name>> {
    preceded(alt((tag("F"), tag("CF"), tag("SF"))), read_names)(input)
}

fn namespace(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("Q")(input)?;
    let (input, depth) = digit1(input)?;
    let depth = depth.parse::<usize>().unwrap();
    let (input, _) = tag("_")(input)?;
    let (input, path) = count(read_name, depth)(input)?;

    Ok((input, Name::Namespace(path)))
}

fn base_type(input: &str) -> nom::IResult<&str, Name> {
    let base_types = get_base_types()
        .keys()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("");

    let (input, base_type) = one_of(base_types.as_str())(input)?;

    Ok((input, Name::BaseType(base_type)))
}

fn function_pointer(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("F")(input)?;
    permutation((read_names, tag("_"), read_name))
        .map(|x| {
            let (args, _, ret) = x;
            Name::FunctionPointer(args, Box::new(ret))
        })
        .parse(input)
}

fn read_name(input: &str) -> nom::IResult<&str, Name> {
    let (mut input, mut name) = alt((
        read_name_identifier,
        namespace,
        read_modifier,
        type_ref,
        base_type,
        function_pointer,
        sized_array,
    ))(input)?;

    loop {
        let res = template(input);
        if let Ok((new_input, args)) = res {
            input = new_input;
            name = Name::Template(Box::new(name), args);
            continue;
        }

        let res = preceded(tag("__"), read_name)(input);
        if let Ok((new_input, parent)) = res {
            input = new_input;
            name = Name::InName(Box::new(name), Box::new(parent));
            continue;
        }
        break;
    }

    Ok((input, name))
}

fn read_function(input: &str) -> nom::IResult<&str, Name> {
    let (mut input, mut name) = read_name(input)?;
    loop {
        let res: nom::IResult<&str, &str> = tag("__S")(input);
        if let Ok((new_input, _)) = res {
            input = new_input;
            // TODO: make name to static function
            continue;
        }

        let res = preceded(opt(tag("__")), arguments)(input);
        if let Ok((new_input, args)) = res {
            input = new_input;
            name = Name::WithArguments(Box::new(name), args);
            continue;
        }

        let res = preceded(tag("_"), read_name)(input);
        if let Ok((new_input, return_value_type)) = res {
            input = new_input;
            name = Name::WithReturnValue(Box::new(name), Box::new(return_value_type));
            continue;
        }

        let res = preceded(tag("__"), read_name)(input);
        if let Ok((new_input, parent)) = res {
            input = new_input;
            name = Name::InName(Box::new(name), Box::new(parent));
            continue;
        }
        break;
    }

    Ok((input, name))
}

fn template(input: &str) -> nom::IResult<&str, Vec<Name>> {
    let (input, string) = preceded(tag("__tm__"), extract_string)(input)?;
    let string = &string[1..];
    //template_value
    let (string, names) = read_names(string)?;

    if !string.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )));
    }

    Ok((input, names))
}

fn decompress(input: &str) -> nom::IResult<&str, String> {
    let input = if input.starts_with("__ghs_thunk__") {
        &input[25..]
    } else {
        input
    };

    let input = if input.starts_with("__CPR") {
        let input = &input[5..];

        let (input, decompressed_length) = digit1(input)?;
        let (input, _) = tag("__")(input)?;
        let raw_data = input;
        let _decompressed_length = decompressed_length.parse::<usize>().unwrap();

        let tokens = raw_data.split("J");
        let mut decompressed = "".to_string();

        for (i, c) in tokens.enumerate() {
            if i % 2 == 0 {
                decompressed += c;
            } else {
                if c.is_empty() {
                    decompressed += "J";
                } else {
                    let offset = c.parse::<usize>().unwrap();

                    let mut s = "".to_string();
                    for c in decompressed[offset..].chars() {
                        s.push(c);
                    }

                    let t = extract_string(&s).unwrap().1;
                    decompressed += &(t.len().to_string() + t);
                }
            }
        }
        // println!("{}", decompressed);
        decompressed
    } else {
        input.to_string()
    };

    Ok(("", input))
}

fn _demangle(input: &str) -> nom::IResult<&str, Name> {
    let (input, name_obj) = read_function(input)?;
    // if !input.is_empty() {
    //     println!("");
    //     println!("### Error ###");
    //     println!("name: {:}", name_obj);
    //     println!("Remain: {:}", input);
    //     return Err(nom::Err::Error(nom::error::Error::new(
    //         input,
    //         nom::error::ErrorKind::NonEmpty,
    //     )));
    // }
    Ok((input, name_obj))
}

fn preprocess(x: String, dunder_search_index: usize) -> String {
    if x.starts_with("__")
        && x[2..]
            .to_string()
            .into_bytes()
            .iter()
            .all(|x| x.is_ascii_alphanumeric())
    {
        println!("{x} -> Returns with rule #0");
        return format!("{}{}", x.len(), x);
    }

    if x.starts_with("__") && !x[2..].contains("__") {
        // __[^__]*
        println!("{x} -> Returns with rule #1");
        return format!("{}{}", x.len(), x);
    }

    let mut predictions = vec![];

    if let Some(i) = x.find("__F") {
        predictions.push(i);
    }

    if let Some(i) = x.find("__tm__") {
        predictions.push(i);
    }

    for i in 1..9 {
        if let Some(i) = x.find(format!("Q{i}_").as_str()) {
            predictions.push(i - 2);
        }
    }

    if predictions.is_empty() {
        println!("{x} -> Returns with rule #2");
        return format!("{}{}", x.len(), x);
    }

    let mut min_index = 0;
    for i in 0..predictions.len() {
        if predictions[i] < predictions[min_index] {
            min_index = i;
        }
    }
    println!("{x} -> Selected #{min_index} of {predictions:?}");
    return format!("{}{}", predictions[min_index], x);
}

pub fn demangle(x: String) -> Name {
    let x = if let Ok(x) = decompress(&x) {
        x.1
    } else {
        return Name::Identifier(x);
    };
    let x = preprocess(x, 0);
    let x = if let Ok((_, x)) = _demangle(&x) {
        x
    } else {
        return Name::Identifier(x);
    };
    x
}

#[wasm_bindgen]
pub fn demangle_str(x: &str) -> String {
    format!("{:?}", demangle(x.to_string()))
}

fn main() {
    let _ = fs::read_to_string("a.txt")
        .unwrap()
        .split("\n")
        .map(|x| demangle(x.to_string()))
        .collect::<Vec<_>>();
}
