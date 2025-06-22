use clap::{Parser, Subcommand, command};
use clap_repl::reedline;

use crate::search::Depth;

pub struct EmptyPrompt;

impl reedline::Prompt for EmptyPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_indicator(
        &self,
        _prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        "".into()
    }
}

#[derive(Parser)]
#[command(name = "")]
pub enum Command {
    /// Set the current engine position.
    Position {
        #[command(subcommand)]
        subcommand: PositionCommand,
    },
    /// Search from the current position.
    Go { subcommands: Vec<String> },
    /// Set an engine option.
    Setoption {
        #[command(subcommand)]
        name: SetoptionNameCommand,
    },
    /// Statically evaluate the current position.
    Evaluate,
    /// Run a move generation test in the current position.
    Perft { depth: Depth },
    /// Stop the current search.
    Stop,
    /// Signal that the expected move was played.
    Ponderhit,
    /// List the moves available in the current position.
    List,
    /// Display the current position.
    Display,
    /// Respond when available.
    Isready,
    /// Identify the engine.
    Uci,
    /// Exit the process.
    Exit,
}

#[derive(Subcommand)]
pub enum PositionCommand {
    /// From a Forsyth–Edwards Notation string.
    Fen {
        /// The Forsyth–Edwards Notation describing the position.
        fen: Vec<String>,
    },
    /// From the starting position.
    Startpos {
        #[command(subcommand)]
        continuation: Option<PositionStartposCommand>,
    },
    /// The Kiwipete position.
    Kiwi,
}

#[derive(Subcommand)]
pub enum PositionStartposCommand {
    /// A sequence of moves from the start position in long algebraic notation. For example, "e4e5 g8f6".
    Moves {
        /// The sequence of moves.
        moves: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum SetoptionNameCommand {
    /// The name of the option.
    Name {
        /// The name of the option.
        name: String,

        #[command(subcommand)]
        value: SetoptionValueCommand,
    },
}

#[derive(Subcommand)]
pub enum SetoptionValueCommand {
    /// The value of the option.
    Value {
        /// The value of the option.
        value: String,
    },
}
