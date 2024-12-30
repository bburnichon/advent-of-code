use miette::miette;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let result = parse(input)
        .map_err(|err| miette!("Parse error: {}", err))?;

    Ok(result.to_string())
}

fn parse(input: &str) -> Result<String, &'static str> {
    Err("day_11 - part 2")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "";
        assert_eq!("", process(input)?);
        Ok(())
    }
}
