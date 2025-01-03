use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending},
    combinator::{all_consuming, opt},
    multi::separated_list0,
    sequence::{separated_pair, terminated},
    IResult,
};
use std::collections::HashMap;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (towels, patterns)) =
        parse(input).map_err(|err| {
            miette::miette!("Parse error: {}", err)
        })?;

    let result =
        count_possible_matching_patterns_with_towels(
            towels.as_slice(),
            patterns.as_slice(),
        );

    Ok(result.to_string())
}

fn count_possible_matching_pattern<'a>(
    cache: &mut HashMap<&'a str, usize>,
    towels: &[&str],
    pattern: &'a str,
) -> usize {
    if pattern.is_empty() {
        return 1;
    }

    if let Some(value) = cache.get(pattern) {
        return *value;
    }

    let count = towels
        .iter()
        .filter(|&towel| !towel.is_empty())
        .filter_map(|&towel| pattern.strip_prefix(towel))
        .map(|sub_pattern| {
            count_possible_matching_pattern(
                cache,
                towels,
                sub_pattern,
            )
        })
        .sum::<usize>();

    cache.insert(pattern, count);

    count
}

fn count_possible_matching_patterns_with_towels(
    towels: &[&str],
    patterns: &[&str],
) -> usize {
    let mut cache = HashMap::new();

    patterns
        .iter()
        .map(|pattern| {
            count_possible_matching_pattern(
                &mut cache, towels, pattern,
            )
        })
        .sum()
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
        "16"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
