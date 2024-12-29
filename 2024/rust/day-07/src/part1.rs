use itertools::Itertools;
use miette::miette;
use nom::{
    bytes::complete::tag,
    character::{complete, complete::line_ending},
    combinator::{all_consuming, opt},
    multi::{many1, separated_list1},
    sequence::{separated_pair, terminated},
    IResult,
};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, operations) = parse(input)
        .map_err(|err| miette!("Parse error: {}", err))?;

    let sum: u64 = operations
        .iter()
        .filter_map(|(target, operands)| {
            if has_solution(*target, operands.as_slice()) {
                Some(*target)
            } else {
                None
            }
        })
        .sum();

    Ok(sum.to_string())
}

fn parse(
    input: &str,
) -> IResult<&str, Vec<(u64, Vec<u64>)>> {
    all_consuming(many1(operation))(input)
}

fn operation(
    input: &str,
) -> IResult<&str, (u64, Vec<u64>)> {
    terminated(
        separated_pair(
            complete::u64,
            tag(": "),
            separated_list1(tag(" "), complete::u64),
        ),
        opt(line_ending),
    )(input)
}

fn has_solution(result: u64, operands: &[u64]) -> bool {
    if operands.is_empty() {
        return false;
    }

    let operator_count = operands.len() - 1;
    (0..operator_count)
        .map(|_| [Operator::Multiply, Operator::Plus])
        .multi_cartesian_product()
        .filter_map(|candidates| {
            let mut operands = operands.iter();
            let first_operand = operands
                .next()
                .expect("at least one operand");

            let candidate_result =
                candidates.iter().zip(operands).try_fold(
                    *first_operand,
                    |acc, (operator, operand)| {
                        let partial_result = match *operator
                        {
                            Operator::Plus => {
                                acc + *operand
                            }
                            Operator::Multiply => {
                                acc * *operand
                            }
                        };

                        if partial_result > result {
                            None
                        } else {
                            Some(partial_result)
                        }
                    },
                );
            candidate_result
        })
        .any(|candidate| candidate == result)
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Operator {
    Plus,
    Multiply,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20";
        assert_eq!("3749", process(input)?);
        Ok(())
    }
}
