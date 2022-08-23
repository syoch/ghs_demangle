use std::fs;

use constants::{get_name_modifiers, Modifier};
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take},
    character::complete::{digit1, one_of},
    multi::many0,
    sequence::preceded,
};

mod constants;

#[derive(Debug, Clone)]
enum Name {
    Identifier(String),
    WithArguments(Box<Name>, Vec<Name>),
    Modifier(Modifier, Box<Name>),
}

fn read_name_identifier(input: &str) -> nom::IResult<&str, Name> {
    let (input, length) = digit1(input)?;
    let length = length.parse::<usize>().unwrap();
    let (input, ident) = take(length)(input)?;

    Ok((input, Name::Identifier(ident.to_string())))
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

fn read_names(input: &str) -> nom::IResult<&str, Vec<Name>> {
    many0(read_name)(input)
}

fn arguments(input: &str) -> nom::IResult<&str, Vec<Name>> {
    preceded(tag("F"), read_names)(input)
}

fn read_name(input: &str) -> nom::IResult<&str, Name> {
    let (mut input, mut name) =
        alt((read_name_identifier, read_name_identifier, read_modifier))(input)?;
    loop {
        let res = preceded(tag("__"), arguments)(input);
        if let Ok((new_input, args)) = res {
            input = new_input;
            name = Name::WithArguments(Box::new(name), args);
        }

        break;
    }

    Ok((input, name))
}

fn demangle(input: &str) -> nom::IResult<&str, Name> {
    let (input, name_obj) = read_name(input)?;
    if !input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )));
    }
    Ok((input, name_obj))
}

fn main() {
    fs::read_to_string("/shared/WiiU/functions")
        .unwrap()
        .split("\n")
        .map(|x| {
            if x.contains("__") {
                format!("{}{}", x.find("__").unwrap(), x)
            } else {
                format!("{}{}", x.len(), x)
            }
        })
        .map(|x| demangle(x.as_str()).unwrap().1.to_owned())
        .collect::<Vec<_>>();
}
