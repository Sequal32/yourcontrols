use yourcontrols_definitions::DefinitionsParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = DefinitionsParser::new();

    parser.load_scripts("definitions/scripts")?;
    parser.load_file("definitions/templates/ToggleSwitch.yaml")?;
    parser.load_file("definitions/templates/NumSet.yaml")?;

    println!(
        "{:#?} {:#?} {:#?}",
        parser.load_file("definitions/aircraft/Asobo_C172.yaml"),
        parser.get_parsed_datums(),
        parser.get_parsed_vars().clone()
    );
    // parser.load_definition_file("definitions/aircraft/Asobo_330LT.yaml");
    // println!("{:?}", parser.get_datums());

    Ok(())
}
