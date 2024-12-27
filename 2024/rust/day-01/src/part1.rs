use miette::miette;
use nom::{
    character::complete,
    combinator::{all_consuming, opt, value},
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};

fn parse(input: &str) -> IResult<&str, Vec<(i32, i32)>> {
    let (rest, output) = separated_list1(
        complete::line_ending,
        separated_pair(
            complete::i32,
            complete::space1,
            complete::i32,
        ),
    )(input)?;

    all_consuming(value(
        output,
        opt(complete::line_ending),
    ))(rest)
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, pairs) = parse(input)
        .map_err(|e| miette!("parse failed {}", e))?;

    let (mut left, mut right): (Vec<_>, Vec<_>) =
        pairs.into_iter().unzip();

    left.sort();
    right.sort();

    let sum: i32 = std::iter::zip(left, right)
        .map(|(l, r)| (r - l).abs())
        .sum();

    Ok(sum.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "3   4
4   3
2   5
1   3
3   9
3   3";
        assert_eq!("11", process(input)?);
        Ok(())
    }
}
