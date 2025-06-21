use clap::{Parser, Subcommand};
use clap_repl::{
    ClapEditor,
    reedline::{DefaultPrompt, DefaultPromptSegment},
};

#[allow(dead_code)]
pub mod core;
#[allow(dead_code)]
pub mod evaluation;
#[allow(dead_code)]
pub mod search;

#[derive(Parser)]
#[command(name = "")]
enum Command {
    /// Set the current engine position.
    Position {
        #[command(subcommand)]
        subcommand: PositionCommand,
    },
}

#[derive(Subcommand)]
enum PositionCommand {
    /// From a Forsyth–Edwards Notation string.
    Fen {
        /// The Forsyth–Edwards Notation describing the position.
        fen: Vec<String>,
    },
    /// From the starting position of regular chess.
    Startpos {
        #[command(subcommand)]
        continuation: Option<PositionStartposCommand>,
    },
}

#[derive(Subcommand)]
enum PositionStartposCommand {
    Moves {
        /// A sequence of moves from the start position in long algebraic notation. For example, "e4e5 g8f6".
        moves: Vec<String>,
    },
}

fn main() {
    println!("WARNING: Camel v2 is in development and is not a complete chess engine yet.");

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("camel".to_string()),
        right_prompt: DefaultPromptSegment::Empty,
    };
    let rl = ClapEditor::<Command>::builder()
        .with_prompt(Box::new(prompt))
        .build();
    rl.repl(|cmd| match cmd {
        Command::Position { subcommand } => match subcommand {
            PositionCommand::Startpos { continuation } => match continuation {
                Some(PositionStartposCommand::Moves { moves }) => {
                    println!(
                        "Setting position from start position with moves: {:?}",
                        moves
                    );
                }
                None => {
                    println!("Setting position to the starting position of regular chess.");
                }
            },
            PositionCommand::Fen { fen } => {
                println!("Setting position from FEN: {:?}", fen);
            }
        },
    });
}
