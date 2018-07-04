pub mod parse;
pub mod flat;
pub mod referenced;


fn main() {
    let parsed = ::flat::FlatObject::parse(r#"

        ok_text: 'Ok'

        Button: {
            visible: 'true'
            text: 'Click Here'
        }

        ok_button: Button { text: ok_text }

    "#);

    println!("parsed: \n{:#?}", parsed.expect("Parsing Error"));
}
