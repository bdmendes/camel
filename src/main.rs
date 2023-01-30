pub mod evaluation;
pub mod position;
pub mod search;
pub mod uci;

fn uci_loop() {
    let mut state = uci::EngineState::new();
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match uci::UCICommand::parse(&input) {
            Ok(command) => state.execute(command),
            Err(error) => {
                println!("{}", error);
                println!("Enter 'help' for program usage details.");
            }
        }
    }
}

fn main() {
    uci_loop();
}
