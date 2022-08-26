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

mod constants;

#[derive(Debug, Clone)]
enum Name {
    Identifier(String),
    BaseType(char),
    WithArguments(Box<Name>, Vec<Name>),
    Template(Box<Name>, Vec<Name>),
    Modifier(Modifier, Box<Name>),
    Namespace(Vec<Name>),
    InName(Box<Name>, Box<Name>),
    WithReturnValue(Box<Name>, Box<Name>),

    Names_Ref(usize),
    Names_Multi(usize, usize),
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

fn type_ref(input: &str) -> nom::IResult<&str, Name> {
    // TODO: Z\d_\dZ
    let (input, t) = delimited(
        tag("Z"),
        terminated(digit1, permutation((tag("_"), digit1))),
        tag("Z"),
    )(input)?;
    let t = t.parse::<usize>().unwrap();

    Ok((input, Name::Identifier(t.to_string())))
}

fn read_name_ref(input: &str) -> nom::IResult<&str, Name> {
    let (input, _) = tag("T")(input)?;
    let (input, index) = digit1(input)?;

    Ok((input, Name::Names_Ref(index.parse::<usize>().unwrap())))
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
    let (input, names) = many0(alt((read_name, read_name_ref, read_name_repeat)))(input)?;

    let mut ret: Vec<Name> = Vec::new();
    for name in names {
        match name {
            Name::Names_Ref(index) => ret.push(ret[index - 1].clone()),
            Name::Names_Multi(count, index) => {
                for i in 0..=count {
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

fn read_name(input: &str) -> nom::IResult<&str, Name> {
    let (mut input, mut name) = alt((
        read_name_identifier,
        namespace,
        read_modifier,
        type_ref,
        base_type,
    ))(input)?;
    loop {
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

fn template(input: &str) -> nom::IResult<&str, Vec<Name>> {
    let (input, string) = preceded(tag("__tm__"), extract_string)(input)?;
    let string = &string[1..];

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
        decompressed
    } else {
        input.to_string()
    };

    Ok(("", input))
}

fn demangle(input: &str) -> nom::IResult<&str, Name> {
    let (input, name_obj) = read_name(input)?;
    if !input.is_empty() {
        println!("Error");
        println!("{:?}", name_obj);
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )));
    }
    Ok((input, name_obj))
}

fn main() {
    fs::read_to_string("/shared/WiiU/GhidraScript/a.txt")
        .unwrap()
        .split("\n")
        .map(|x| decompress(x).unwrap().1)
        .map(|x| {
            if x.contains("__") {
                if x.find("__") == Some(0) {
                    format!("{}{}", 2 + x[2..].find("__").unwrap(), x)
                } else {
                    format!("{}{}", x.find("__").unwrap(), x)
                }
            } else {
                format!("{}{}", x.len(), x)
            }
        })
        .map(|x| demangle(x.as_str()).unwrap().1.to_owned())
        .collect::<Vec<_>>();
}
