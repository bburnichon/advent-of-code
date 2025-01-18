use std::iter::successors;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let buyers = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    let sum = buyers
        .iter()
        .filter_map(|buyer| {
            successors(
                Some(*buyer as usize),
                compute_next_number,
            )
            .skip(2000)
            .next()
        })
        .sum::<usize>();

    Ok(sum.to_string())
}

fn compute_next_number(secret: &usize) -> Option<usize> {
    let mix = |value: usize, secret: usize| value ^ secret;
    let prune = |secret: usize| secret % 16777216;

    let next = prune(mix(secret * 64, *secret));
    let next = prune(mix(next / 32, next));
    let next = prune(mix(next * 2048, next));

    Some(next)
}

fn parse(input: &str) -> Result<Vec<u32>, &'static str> {
    let result = input
        .lines()
        .map(|buyer| buyer.parse::<u32>())
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
10
100
2024",
        "37327623"
    )]

    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
