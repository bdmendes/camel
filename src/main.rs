mod position;
mod uci;

fn uci_loop() {
    let state = uci::EngineState::new();
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match uci::UCICommand::parse(&input) {
            Ok(command) => state.execute(command),
            Err(error) => println!("Error: {}", error),
        }
    }
}

fn main() {
    uci_loop();
}
