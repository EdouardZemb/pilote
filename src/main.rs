//! L'exécutable `pilote`. Sous Windows, il lance l'outil et sort avec son code ; ailleurs, il dit
//! qu'il ne tourne que sous Windows et sort en 64 (ADR-0003).

use std::process::ExitCode;

#[cfg(windows)]
fn main() -> ExitCode {
    ExitCode::from(pilote::lancer())
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("pilote ne tourne que sous Windows");
    ExitCode::from(64)
}
