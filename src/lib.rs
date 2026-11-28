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
    use windows::Win32::System::Com::COINIT_MULTITHREADED;
    use windows::Win32::System::Com::CoInitializeEx;

    std::panic::set_hook(Box::new(|_| {}));
    systeme::sensible_au_dpi();
    // COM en MTA sur ce fil (ADR-0005, « Ce que la tranche 2 précise ou change ») : le client de
    // l'automatisation y vit sans fil d'interface, et une application figée ne bloque pas `pilote`
    // sur une pompe de messages. Les AUMID se lisent à part, sur un fil neuf en STA
    // (`bureau::fenetres`). Le résultat s'ignore : S_FALSE dit COM déjà initialisé en MTA sur ce
    // fil ; RPC_E_CHANGED_MODE le dirait déjà en STA, ce qui ne peut arriver ici, rien
    // n'initialisant COM avant cet appel ; tout autre échec fait échouer les appels COM qui
    // suivent, qui rendent alors une erreur que le domaine juge. Le processus finit sans
    // `CoUninitialize`.
    // SAFETY: aucun argument réservé ; un appel unique, sur le fil qui exécute la commande.
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
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
