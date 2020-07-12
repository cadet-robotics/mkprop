use nom::IResult;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};

#[cfg(test)]
mod test {
    use nom::error::ErrorKind::Eof;

    use crate::parsing::{DefineStatement, MapStatement, parse_driver_data, parse_template, Template};

    #[test]
    fn test_template() {
        let test = "\n@CLASS  BR . MAP LO-2O 3bar_3 .MAP HIGH BAR OPT .\nMAP HIGH BAR. ";
        assert_eq!(parse_template(test).unwrap(), Template {
            class_name: "BR",
            maps: vec![
                MapStatement {
                    driver_data_name: "LO-2O",
                    const_name: "3bar_3",
                    opt: false,
                },
                MapStatement {
                    driver_data_name: "HIGH",
                    const_name: "BAR",
                    opt: true,
                },
                MapStatement {
                    driver_data_name: "HIGH",
                    const_name: "BAR",
                    opt: false,
                }
            ],
        })
    }

    #[test]
    fn test_template_failure() {
        let test = "\n@CLASS  BR . MAP LO-2O 3bar_3 .MAP HIGH BAR OPT\nMAP HIGH BAR. ";
        assert_eq!(parse_template(test), Err(
            nom::Err::Error(("MAP HIGH BAR OPT\nMAP HIGH BAR. ", Eof))
        ))
    }

    #[test]
    fn test_driver_data() {
        let test = "\n DEF 3LO-2O- -12.DEF HIGH 12 .\nDEF HIGH 3000. ";
        assert_eq!(parse_driver_data(test).unwrap(), vec![
            DefineStatement {
                name: "3LO-2O-",
                id: -12,
            },
            DefineStatement {
                name: "HIGH",
                id: 12,
            },
            DefineStatement {
                name: "HIGH",
                id: 3000,
            }
        ])
    }

    #[test]
    fn test_driver_data_failure() {
        let test = " DEF 3LO-2O- -12.DEF HIGH 12 .\nDEF HIGH 3900000000. ";
        assert_eq!(parse_driver_data(test), Err(
            nom::Err::Error(("DEF HIGH 3900000000. ", Eof))
        ))
    }
}

fn parse_whitespace(s: &str) -> IResult<&str, ()> {
    map(take_while1(|v: char| {
        match v {
            '\n' | '\t' | ' ' => true,
            _ => false
        }
    }), |_| ())(s)
}

fn parse_whitespace_opt(s: &str) -> IResult<&str, ()> {
    map(take_while(|v: char| {
        match v {
            '\n' | '\t' | ' ' => true,
            _ => false
        }
    }), |_| ())(s)
}

fn parse_identifier(s: &str) -> IResult<&str, &str> {
    take_while1(|v: char| {
        match v {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => true,
            _ => false
        }
    })(s)
}

fn parse_i32(s: &str) -> IResult<&str, i32> {
    map_res(take_while1(|v: char| {
        v.is_ascii_digit() || (v == '-')
    }), |r: &str| {
        r.parse::<i32>()
    })(s)
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub(crate) struct MapStatement<'a> {
    pub(crate) driver_data_name: &'a str,
    pub(crate) const_name: &'a str,
    pub(crate) opt: bool,
}

fn parse_map_statement(s: &str) -> IResult<&str, MapStatement> {
    map(tuple((
        tag("MAP"),
        parse_whitespace,
        parse_identifier,
        parse_whitespace,
        parse_identifier,
        opt(preceded(parse_whitespace, tag("OPT"))),
        parse_whitespace_opt,
        tag(".")
    )), |v| {
        MapStatement {
            driver_data_name: v.2,
            const_name: v.4,
            opt: v.5.is_some(),
        }
    })(s)
}

fn parse_class_statement(s: &str) -> IResult<&str, &str> {
    map(tuple((
        tag("@CLASS"),
        parse_whitespace,
        parse_identifier,
        parse_whitespace_opt,
        tag(".")
    )), |v| v.2)(s)
}

#[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub(crate) struct DefineStatement<'a> {
    pub(crate) name: &'a str,
    pub(crate) id: i32,
}

fn parse_define_statement(s: &str) -> IResult<&str, DefineStatement> {
    map(tuple((
        tag("DEF"),
        parse_whitespace,
        parse_identifier,
        parse_whitespace,
        parse_i32,
        parse_whitespace_opt,
        tag(".")
    )), |v| {
        DefineStatement {
            name: v.2,
            id: v.4,
        }
    })(s)
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub(crate) struct Template<'a> {
    pub(crate) class_name: &'a str,
    pub(crate) maps: Vec<MapStatement<'a>>,
}

pub(crate) fn parse_template(s: &str) -> Result<Template, nom::Err<(&str, nom::error::ErrorKind)>> {
    all_consuming(
        pair(
            delimited(
                parse_whitespace_opt,
                parse_class_statement,
                parse_whitespace_opt,
            ),
            many0(terminated(parse_map_statement, parse_whitespace_opt)),
        )
    )(s).map(|v| {
        let (class_name, maps) = v.1;
        Template {
            class_name,
            maps,
        }
    })
}

pub(crate) fn parse_driver_data(s: &str) -> Result<Vec<DefineStatement>, nom::Err<(&str, nom::error::ErrorKind)>> {
    all_consuming(
        preceded(
            parse_whitespace_opt,
            many0(terminated(parse_define_statement, parse_whitespace_opt)),
        )
    )(s).map(|v| v.1)
}
