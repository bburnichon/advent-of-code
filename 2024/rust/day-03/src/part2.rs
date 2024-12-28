use miette::miette;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    combinator::value,
    multi::{many1, many_till},
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let (_, instructions) = parse(_input)
        .map_err(|err| miette!("Parsing error: {}", err))?;

    let mut current = Instruction::Do;

    let sum = instructions
        .into_iter()
        .map(|instruction| match instruction {
            Instruction::Mul(x, y) => {
                if current == Instruction::Do {
                    x * y
                } else {
                    0
                }
            }
            ins => {
                current = ins;
                0
            }
        })
        .sum::<u32>();

    Ok(sum.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<Instruction>> {
    many1(
        many_till(complete::anychar, instruction)
            .map(|(_, instruction)| instruction),
    )(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    alt((
        multiply,
        value(Instruction::Do, tag("do()")),
        value(Instruction::Dont, tag("don't()")),
    ))(input)
}

fn multiply(input: &str) -> IResult<&str, Instruction> {
    let (input, _operation) = tag("mul")(input)?;

    let (input, pair) = delimited(
        tag("("),
        separated_pair(
            complete::u32,
            tag(","),
            complete::u32,
        ),
        tag(")"),
    )(input)?;

    Ok((input, Instruction::Mul(pair.0, pair.1)))
}

#[derive(Debug, PartialEq, Clone)]
enum Instruction {
    Mul(u32, u32),
    Do,
    Dont,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";
        assert_eq!("48", process(input)?);
        Ok(())
    }
}
