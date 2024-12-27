use miette::miette;
use nom::{
    character::complete,
    combinator::{all_consuming, opt, value},
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};
use std::collections::HashMap;

fn parse(input: &str) -> IResult<&str, Vec<(u32, u32)>> {
    let (rest, output) = separated_list1(
        complete::line_ending,
        separated_pair(
            complete::u32,
            complete::space1,
            complete::u32,
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

    let (left, right) = pairs.into_iter().fold(
        (HashMap::new(), HashMap::new()),
        |mut acc: (
            HashMap<u32, usize>,
            HashMap<u32, usize>,
        ),
         (l, r)| {
            *acc.0.entry(l).or_default() += 1;
            *acc.1.entry(r).or_default() += 1;
            acc
        },
    );

    let sum = left
        .iter()
        .map(|(k, v)| {
            right.get(k).copied().unwrap_or_default()
                * (*k as usize)
                * *v
        })
        .sum::<usize>();

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
        assert_eq!("31", process(input)?);
        Ok(())
    }
}
