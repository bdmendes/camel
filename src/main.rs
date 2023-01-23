use camel::{position::Position, uci};

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
    //uci_loop();
    let position =
        Position::from_fen("q5k1/3R2pp/p3pp2/N1b5/4b3/2B2r2/6PP/4QB1K b - - 5 35").unwrap();
    let mut searcher = camel::search::Searcher::new();
    for i in 1..=3 {
        let (moves, _) = searcher.search(&position, i);
        if moves.len() == 0 {
            println!("Depth {}: no moves", i);
        } else {
            println!("Depth {}: {} {}", i, moves[0].0.to_string(), moves[0].1);
        }
        //println!("Depth {}: {} {}", i, moves[0].0.to_string(), moves[0].1);
    }
}
