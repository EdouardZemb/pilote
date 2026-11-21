//! L'application d'essai de `pilote` : la cible des essais de bout en bout, sous Windows. Jusqu'à
//! la Tâche 10, elle ne fait que dire qu'elle ne tourne pas, et sort en 64, partout.

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("appli-essai ne tourne que sous Windows");
    ExitCode::from(64)
}
