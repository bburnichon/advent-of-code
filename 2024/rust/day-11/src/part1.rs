use miette::miette;
use nom::{
    bytes::complete::tag,
    character::{complete, complete::line_ending},
    combinator::{all_consuming, opt},
    multi::separated_list1,
    sequence::terminated,
    IResult,
};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, rocks) = parse(input)
        .map_err(|err| miette!("Parse err: {}", err))?;

    let mut rocks: Box<dyn Iterator<Item = u64>> =
        Box::new(rocks.iter().copied());
    for _ in 0..25 {
        rocks = Box::new(blink(rocks));
    }

    let count = rocks.count();

    Ok(count.to_string())
}

fn blink(
    rocks: impl Iterator<Item = u64>,
) -> impl Iterator<Item = u64> {
    rocks.flat_map(|rock| {
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

        rocks.into_iter()
    })
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
    #[case(&[125,17], &[253000, 1, 7])]
    #[case(&[253000, 1, 7], &[253,0,2024, 14168])]
    #[case(&[512072,1,20,24,28676032],&[512,72,2024,2,0,2,4,2867,6032])]
    #[case(&[1036288,7,2,20,24,4048,1,4048,8096,28,67,60,32],&[2097446912,14168,4048,2,0,2,4,40,48,2024,40,48,80,96,2,8,6,7,6,0,3,2])]
    fn test_blink(
        #[case] input: &[u64],
        #[case] expected: &[u64],
    ) {
        let input = input.to_vec();
        let expected = expected.to_vec();
        let it = blink(input.into_iter());
        let actual = it.collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "125 17";
        assert_eq!("55312", process(input)?);
        Ok(())
    }
}
