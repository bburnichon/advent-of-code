use itertools::Itertools;
use std::{collections::HashMap, iter::successors};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let buyers = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    let map = buyers.iter().fold(
        HashMap::<[i32; 4], usize>::with_capacity(2000 - 4),
        |mut map, buyer| {
            let line_map = successors(
                Some(*buyer),
                compute_next_number,
            )
            .take(2000)
            .map(|secret| (secret % 10) as i32)
            .tuple_windows()
            .map(|(old_price, new_price)| {
                (
                    new_price as usize,
                    new_price - old_price,
                )
            })
            .tuple_windows()
            .fold(
                HashMap::<[i32; 4], usize>::with_capacity(
                    2000 - 4,
                ),
                |mut map, (a, b, c, d)| {
                    let key = [a.1, b.1, c.1, d.1];
                    map.entry(key).or_insert(d.0);
                    map
                },
            );

            for (key, val) in line_map {
                let entry = map.entry(key).or_default();
                *entry += val;
            }
            map
        },
    );

    Ok(map
        .values()
        .max()
        .copied()
        .unwrap_or_default()
        .to_string())
}

fn compute_next_number(secret: &usize) -> Option<usize> {
    let mix = |value: usize, secret: usize| value ^ secret;
    let prune = |secret: usize| secret % 16777216;

    let next = prune(mix(secret * 64, *secret));
    let next = prune(mix(next / 32, next));
    let next = prune(mix(next * 2048, next));

    Some(next)
}

fn parse(input: &str) -> Result<Vec<usize>, &'static str> {
    let result = input
        .lines()
        .map(|buyer| buyer.parse::<usize>())
        .collect::<Result<Vec<_>, _>>();

    result.map_err(|_| "Invalid input")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
    123 , &[
    15887950,
    16495136,
    527345,
    704524,
    1553684,
    12683156,
    11100544,
    12249484,
    7753432,
    5908254
    ])]
    fn test_successors(
        #[case] input: usize,
        #[case] expected: &[usize],
    ) -> miette::Result<()> {
        let next_10_values: Vec<usize> =
            successors(Some(input), compute_next_number)
                .skip(1)
                .take(10)
                .collect();
        assert_eq!(expected, next_10_values.as_slice());
        Ok(())
    }

    #[rstest]
    #[case(
        "1
2
3
2024",
        "23"
    )]

    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
