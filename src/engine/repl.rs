use clap::{Error, Parser, Subcommand};

use crate::search::Depth;

// The UCI protocol omits the prefix "--" from most flags and uses some redundant keywords.
// To simplify modelling with clap, we'll preprocess the input string.
const UCI_FLAGS: &[&str] = &[
    "name",
    "value",
    "depth",
    "wtime",
    "btime",
    "winc",
    "binc",
    "ponder",
    "movestogo",
    "movetime",
];
const OMITTED_FLAGS: &[&str] = &["infinite", "moves"];

#[derive(Parser, Debug)]
#[command(name = "")]
pub enum Command {
    /// Set the current engine position.
    Position {
        #[command(subcommand)]
        subcommand: PositionCommand,
    },
    /// Search from the current position.
    Go {
        /// Search in pondering mode, i.e. with no time limit.
        #[arg(long, default_value_t = false)]
        ponder: bool,

        /// A fixed depth to search to.
        #[arg(long)]
        depth: Option<Depth>,

        /// The remaining time for white, in milliseconds.
        #[arg(long)]
        wtime: Option<u32>,

        /// The remaining time for black, in milliseconds.
        #[arg(long)]
        btime: Option<u32>,

        /// The increment set for white in this time control, in milliseconds.
        #[arg(long)]
        winc: Option<u32>,

        /// The increment set for black in this time control, in milliseconds.
        #[arg(long)]
        binc: Option<u32>,

        /// The moves to go until the next time control.
        #[arg(long)]
        movestogo: Option<u8>,

        /// The maximum time the engine should search for.
        #[arg(long)]
        movetime: Option<u32>,
    },
    /// Set an engine option.
    Setoption {
        /// The name of the option.
        #[arg(long)]
        name: String,

        /// The value of the option.
        #[arg(long)]
        value: Option<String>,
    },
    /// Statically evaluate the current position.
    Evaluate,
    /// Run a move generation test in the current position.
    Perft { depth: Depth },
    /// Stop the current search.
    Stop,
    /// Signal that the expected move was played.
    Ponderhit,
    /// Signal that a new game is to be played.
    Ucinewgame,
    /// List the moves available in the current position.
    List,
    /// Display the current position.
    Display,
    /// Execute a move in the current position.
    Move { r#move: String },
    /// Respond when available.
    Isready,
    /// Identify the engine and list available options.
    Uci,
    /// Set debug mode.
    Debug {
        /// "on" or "off".
        value: String,
    },
    /// Serialize auxiliary engine data structures to disk.
    Dump {
        #[command(subcommand)]
        subcommand: DumpCommand,
    },
    /// Quit the process.
    Quit,
}

#[derive(Subcommand, Debug)]
pub enum PositionCommand {
    /// From a Forsyth–Edwards Notation string.
    Fen {
        /// The Forsyth–Edwards Notation describing the position.
        fen: Vec<String>,
    },
    /// From the starting position.
    Startpos { moves: Vec<String> },
    /// The Kiwipete position.
    Kiwi,
}

#[derive(Subcommand, Debug)]
pub enum DumpCommand {
    /// The random numbers for each square, used for Zobrist hashing.
    Zobrist,
    /// The magic numbers, including masks, used for slider move generation.
    Magics,
}

fn parse(input: &String) -> Result<Command, Error> {
    let mut input = input.to_owned();

    for ommited in OMITTED_FLAGS {
        input = input.replace(ommited, "");
    }

    for relaxed in UCI_FLAGS {
        input = input.replace(format!(" {}", relaxed).as_str(), format!(" --{}", relaxed).as_str());
    }

    Command::try_parse_from(std::iter::once("").chain(input.split_ascii_whitespace()))
}

pub fn repl(mut handler: impl FnMut(Command)) {
    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        if buf.trim().is_empty() {
            continue;
        }

        match parse(&buf) {
            Ok(cmd) => handler(cmd),
            Err(err) => print!("{}", err.render().ansi()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::repl::parse;
    use rstest::rstest;

    #[rstest]
    #[case("uci")]
    #[case("ucinewgame")]
    #[case("isready")]
    #[case("setoption name Hash value 64")]
    #[case("setoption name ClearHash")]
    #[case("position startpos")]
    #[case("position startpos moves e2e4 d7d5")]
    #[case("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
    #[case("go")]
    #[case("go infinite")]
    #[case("go depth 6")]
    #[case("go ponder")]
    #[case("go depth 8 movetime 6000")]
    #[case("go wtime 53000 btime 50000 winc 3000 binc 3000")]
    #[case("stop")]
    #[case("ponderhit")]
    #[case("debug on")]
    #[case("quit")]
    fn uci_ok(#[case] input: String) {
        assert!(parse(&input).is_ok());
    }

    #[rstest]
    #[case("position middlegame")]
    #[case("setoption Hash 64")]
    #[case("position moves e2e4")]
    #[case("position e2e4")]
    #[case("e2e4")]
    #[case("go wtime infinite")]
    #[case("go depth -2")]
    fn uci_err(#[case] input: String) {
        assert!(parse(&input).is_err());
    }
}
