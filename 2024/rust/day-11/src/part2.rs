use miette::miette;
use nom::{
    bytes::complete::tag,
    character::{complete, complete::line_ending},
    combinator::{all_consuming, opt},
    multi::separated_list1,
    sequence::terminated,
    IResult,
};
use std::{collections::HashMap, iter};

#[tracing::instrument(skip(input))]
pub fn process(
    input: &str,
    blinks: u32,
) -> miette::Result<String> {
    let (_, rocks) = parse(input)
        .map_err(|err| miette!("Parse err: {}", err))?;

    let mut cache = HashMap::new();
    for rock in rocks {
        cache
            .entry(rock)
            .and_modify(|v| *v += 1u64)
            .or_insert(1u64);
    }

    let mut rocks = cache;
    for _ in 0..blinks {
        rocks = blink_count(rocks);
    }

    let count = rocks.values().sum::<u64>();

    Ok(count.to_string())
}

fn blink_count(
    rocks: HashMap<u64, u64>,
) -> HashMap<u64, u64> {
    let next: Vec<(u64, u64)> = rocks
        .iter()
        .flat_map(|(rock, count)| {
            let rock_digits =
                rock.checked_ilog10().unwrap_or(0) + 1;

            let rocks = match (rock, rock_digits % 2) {
                (0, _) => vec![1],
                (number, 0) => {
                    let power = 10u64.pow(rock_digits / 2);
                    vec![number / power, number % power]
                }
                (number, _) => vec![number * 2024],
            };

            rocks
                .into_iter()
                .zip(iter::repeat(*count))
                .into_iter()
        })
        .into_iter()
        .collect();

    let mut new_counts: HashMap<u64, u64> = HashMap::new();
    for (k, v) in next {
        new_counts
            .entry(k)
            .and_modify(|count| *count += v)
            .or_insert(v);
    }

    new_counts
}

fn parse(input: &str) -> IResult<&str, Vec<u64>> {
    all_consuming(terminated(
        separated_list1(tag(" "), complete::u64),
        opt(line_ending),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("125 17", 6, "22")]
    #[case("125 17", 25, "55312")]
    fn test_process(
        #[case] input: &str,
        #[case] blinks: u32,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input, blinks)?);
        Ok(())
    }
}
