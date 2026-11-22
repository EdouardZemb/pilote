//! `pilote` : piloter une application de bureau de Windows, le clavier d'abord, sûr dès le premier
//! jour. Le domaine, pur et portable, décide ; sous Windows, la couche Windows lit et agit sans
//! décider, et `systeme` ouvre le journal, lit l'horloge, la date et l'environnement (ADR-0003).

pub mod domaine;

#[cfg(windows)]
mod bureau;
#[cfg(windows)]
mod systeme;

/// Lance l'outil sur les arguments du processus, et rend son code de sortie. Le crochet de panique
/// est muet : `executer` rattrape une panique et la dit en une seule ligne, « échec du système :
/// panique », que le crochet par défaut précéderait de plusieurs autres. Le verrou des gestes se
/// relâche avant que l'issue s'écrive : chaque ligne de sa sortie sur la sortie standard, son
/// erreur sur la sortie d'erreur, en UTF-8.
#[cfg(windows)]
pub fn lancer() -> u8 {
    use std::io::Write;

    std::panic::set_hook(Box::new(|_| {}));
    systeme::sensible_au_dpi();
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let issue = domaine::executer(&args, &mut bureau::BureauWindows::new());
    let mut sortie = std::io::stdout().lock();
    for ligne in &issue.sortie {
        let _ = writeln!(sortie, "{ligne}");
    }
    let _ = sortie.flush();
    if let Some(erreur) = &issue.erreur {
        let _ = writeln!(std::io::stderr().lock(), "{erreur}");
    }
    issue.code.valeur()
}
