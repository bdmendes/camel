use clap::{Parser, Subcommand, command};

use crate::search::Depth;

// The UCI protocol omits the prefix "--" from most flags and uses some redundant keywords.
// To simplify modelling with clap, we'll preprocess the input string.
static RELAXED_FLAGS: &[&str] = &[
    "name",
    "value",
    "depth",
    "wtime",
    "btime",
    "winc",
    "binc",
    "ponder",
    "movestogo",
];
static OMITTED_FLAGS: &[&str] = &["infinite", "moves"];

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
    /// Respond when available.
    Isready,
    /// Identify the engine and list available options.
    Uci,
    /// Set debug mode.
    Debug {
        /// "on" or "off".
        value: String,
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

pub fn repl(mut handler: impl FnMut(Command)) {
    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        if buf.trim().is_empty() {
            continue;
        }

        for ommited in OMITTED_FLAGS {
            buf = buf.replace(ommited, "");
        }

        for relaxed in RELAXED_FLAGS {
            buf = buf.replace(
                format!(" {}", relaxed).as_str(),
                format!(" --{}", relaxed).as_str(),
            );
        }

        match Command::try_parse_from(std::iter::once("").chain(buf.split_ascii_whitespace())) {
            Ok(cmd) => handler(cmd),
            Err(err) => print!("{}", err.render().ansi()),
        }
    }
}
