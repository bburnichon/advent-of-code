use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending},
    combinator::{all_consuming, opt},
    multi::separated_list0,
    sequence::{separated_pair, terminated},
    IResult,
};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (towels, patterns)) =
        parse(input).map_err(|err| {
            miette::miette!("Parse error: {}", err)
        })?;

    let result = patterns
        .into_iter()
        .filter(|&pattern| {
            can_match_pattern_with_towels(
                towels.as_slice(),
                pattern,
            )
        })
        .count();

    Ok(result.to_string())
}

fn can_match_pattern_with_towels(
    towels: &[&str],
    pattern: &str,
) -> bool {
    if pattern.is_empty() {
        return true;
    }

    let mut sub_patterns = towels
        .iter()
        .filter(|&towel| !towel.is_empty())
        .filter_map(|&towel| pattern.strip_prefix(towel));

    sub_patterns.any(|pattern| {
        can_match_pattern_with_towels(towels, pattern)
    })
}

fn parse(
    input: &str,
) -> IResult<&str, (Vec<&str>, Vec<&str>)> {
    all_consuming(separated_pair(
        terminated(
            separated_list0(tag(", "), alpha1),
            line_ending,
        ),
        line_ending,
        terminated(
            separated_list0(line_ending, alpha1),
            opt(line_ending),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "r, wr, b, g, bwu, rb, gb, br

brwrr
bggr
gbbr
rrbgbr
ubwu
bwurrg
brgr
bbrgwb",
        "6"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
