use miette::miette;
use nom::{
    bytes::complete::tag,
    character::{complete, complete::line_ending},
    combinator::{all_consuming, opt},
    multi::{many1, separated_list1},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use std::{collections::HashMap, ops::Not};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (rules, updates)) = parse(input)
        .map_err(|err| miette!("Parse error: {}", err))?;

    let precedence = rules.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<_, Vec<_>>, rule| {
            acc.entry(rule.0).or_default().push(rule.1);
            acc
        },
    );

    let sum: u32 = updates
        .iter()
        .filter_map(|update| {
            if update.is_valid(&precedence) {
                return update
                    .0
                    .get(update.0.len() / 2)
                    .copied();
            }
            None
        })
        .sum();

    Ok(sum.to_string())
}

type Rule = (u32, u32);
struct Update(Vec<u32>);

impl Update {
    fn is_valid(
        self: &Self,
        precedence: &HashMap<u32, Vec<u32>>,
    ) -> bool {
        let mut checked = Vec::with_capacity(self.0.len());
        self.0.iter().all(|x| {
            let mut valid = true;
            if let Some(to_check) = precedence.get(x) {
                valid = to_check
                    .iter()
                    .all(|y| checked.contains(y).not())
            }
            if valid {
                checked.push(*x);
            }

            valid
        })
    }
}

fn parse(
    input: &str,
) -> IResult<&str, (Vec<Rule>, Vec<Update>)> {
    let (input, rules) = many1(rule)(input)?;
    let (input, updates) = all_consuming(preceded(
        line_ending,
        many1(update),
    ))(input)?;

    Ok((input, (rules, updates)))
}

fn rule(input: &str) -> IResult<&str, Rule> {
    terminated(
        separated_pair(
            complete::u32,
            tag("|"),
            complete::u32,
        ),
        line_ending,
    )(input)
}

fn update(input: &str) -> IResult<&str, Update> {
    let (input, update) = terminated(
        separated_list1(tag(","), complete::u32),
        opt(line_ending),
    )(input)?;

    Ok((input, Update(update)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";
        assert_eq!("143", process(input)?);
        Ok(())
    }
}
