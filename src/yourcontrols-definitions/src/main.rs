use yourcontrols_definitions::DefinitionsParser;
use yourcontrols_types::Error;

fn main() -> Result<(), Error> {
    let mut parser = DefinitionsParser::with_mapping_path("definitions/mappings")?;
    parser.load_definition_file("definitions/aircraft/Asobo_330LT.yaml");
    println!("{:?}", parser.get_datums());

    Ok(())
}
