use itertools::{chain, Itertools};
use miette::miette;
use nom::{
    character::{
        complete,
        complete::{line_ending, space1},
    },
    combinator::{all_consuming, opt, value},
    multi::separated_list1,
    IResult,
};

#[derive(Debug, Clone)]
struct Report {
    levels: Vec<u32>,
}

impl Report {
    fn from_levels(levels: Vec<u32>) -> Report {
        Report { levels }
    }

    fn is_safe(&self) -> bool {
        let diff = match self.levels.iter().next_tuple() {
            Some((a, b)) if *a < *b => {
                |(low, high): (&u32, &u32)| {
                    *low < *high
                        && ((*low + 1u32)..=(*low + 3))
                            .contains(high)
                }
            }
            Some((_, _)) => |(high, low): (&u32, &u32)| {
                *low < *high
                    && ((*low + 1u32)..=(*low + 3))
                        .contains(high)
            },
            None => return false,
        };

        self.levels.iter().tuple_windows().all(diff)
    }

    fn allow_one_error(&self) -> bool {
        if self.is_safe() {
            return true;
        }

        for index in 0..self.levels.len() {
            let test_report =
                Report::from_levels(Vec::from_iter(
                    chain(
                        &self.levels[..index],
                        &self.levels[(index + 1)..],
                    )
                    .copied(),
                ));

            if test_report.is_safe() {
                return true;
            }
        }

        false
    }
}

fn report(input: &str) -> IResult<&str, Report> {
    let (remaining, levels) =
        separated_list1(space1, complete::u32)(input)?;

    let report = Report::from_levels(levels);

    Ok((remaining, report))
}

fn parse(input: &str) -> IResult<&str, Vec<Report>> {
    let (remaining, reports) =
        separated_list1(line_ending, report)(input)?;

    all_consuming(value(reports, opt(line_ending)))(
        remaining,
    )
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, reports) = parse(input)
        .map_err(|e| miette!("Parsing error: {}", e))?;

    let count = reports
        .iter()
        .filter(|r| r.allow_one_error())
        .count();
    Ok(count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("7 6 4 2 1", true)]
    #[case("1 2 7 8 9", false)]
    #[case("9 7 6 2 1", false)]
    #[case("1 3 2 4 5", true)]
    #[case("8 6 4 4 1", true)]
    #[case("1 3 6 7 9", true)]
    fn test_report_is_safe_when_allowing_one_error(
        #[case] input: &str,
        #[case] is_safe: bool,
    ) -> miette::Result<()> {
        let (_, report) = all_consuming(report)(input)
            .map_err(|e| {
                miette!("Report parsing error: {}", e)
            })?;

        assert_eq!(report.allow_one_error(), is_safe);
        Ok(())
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";
        assert_eq!("4", process(input)?);
        Ok(())
    }
}
