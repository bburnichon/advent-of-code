#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let result = parse(input)?;

    Ok(result.to_string())
}

fn parse(input: &str) -> Result<String, &'static str> {
    Err("{{crate_name}} - part 1")
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
