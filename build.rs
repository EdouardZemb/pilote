//! Le script de construction de `pilote`, sans dépendance : il donne la variable de compilation
//! `PILOTE_COMMIT`, les 12 premiers chiffres hexadécimaux du commit construit, ou « inconnu » quand
//! git manque, échoue ou rend autre chose (une archive, la copie de cargo-mutants, sans `.git`).
//! Il se relance quand ce commit peut avoir changé : un commit change la référence de la branche,
//! pas `HEAD` ; une référence compactée n'a pas de fichier, et `logs/HEAD` s'allonge à chaque
//! commit, `pull`, `checkout` ou `reset`.

use std::path::Path;
use std::process::Command;

/// La sortie de `git <arguments>`, sans sa fin de ligne, ou `None` si git manque, échoue ou ne rend
/// pas de l'UTF-8.
fn git(arguments: &[&str]) -> Option<String> {
    let sortie = Command::new("git").args(arguments).output().ok()?;
    if !sortie.status.success() {
        return None;
    }
    let texte = String::from_utf8(sortie.stdout).ok()?;
    Some(texte.trim_end_matches(['\r', '\n']).to_string())
}

/// Le commit construit : 12 chiffres hexadécimaux, ou « inconnu ».
fn commit() -> String {
    match git(&["rev-parse", "--short=12", "HEAD"]) {
        Some(court) if court.len() == 12 && court.bytes().all(|o| o.is_ascii_hexdigit()) => court,
        _ => "inconnu".to_string(),
    }
}

/// Les fichiers de git dont le changement peut changer le commit construit, ceux qui existent :
/// `HEAD`, la référence qu'il nomme, `packed-refs` et `logs/HEAD`. Sans git, aucun.
fn fichiers_suivis() -> Vec<String> {
    let mut chemins = vec![git(&["rev-parse", "--git-path", "HEAD"])];
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"]) {
        chemins.push(git(&["rev-parse", "--git-path", &reference]));
    }
    chemins.push(git(&["rev-parse", "--git-path", "packed-refs"]));
    chemins.push(git(&["rev-parse", "--git-path", "logs/HEAD"]));
    chemins
        .into_iter()
        .flatten()
        .filter(|chemin| Path::new(chemin).exists())
        .collect()
}

fn main() {
    println!("cargo:rustc-env=PILOTE_COMMIT={}", commit());
    println!("cargo:rerun-if-changed=build.rs");
    for chemin in fichiers_suivis() {
        println!("cargo:rerun-if-changed={chemin}");
    }
}
