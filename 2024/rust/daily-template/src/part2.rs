#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let result = parse(input)
        .map_err(|err| miette::miette!("Parse error: {}", err))?;

    Ok(result.to_string())
}

fn parse(_input: &str) -> Result<String, &'static str> {
    Err("{{crate_name}} - part 2")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "",
        ""
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
