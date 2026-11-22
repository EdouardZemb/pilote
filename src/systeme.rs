//! Le système : le seul module qui résout les chemins de données, ouvre le journal, lit l'horloge,
//! la date, la session et l'environnement (ADR-0003). Hors de lui, le contrôle statique,
//! `tests/statique.rs`, refuse les chemins devinés, l'horloge, l'environnement et le hasard.

use crate::domaine::bureau::Duree;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::bureau::Moment;
use crate::domaine::journal::DateLocale;
use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::create_dir_all;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::time::Duration;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
use windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext;
use windows::Win32::UI::Shell::FOLDERID_LocalAppData;
use windows::Win32::UI::Shell::KF_FLAG_DEFAULT;
use windows::Win32::UI::Shell::SHGetKnownFolderPath;

/// Le dossier de `pilote`, sous le dossier local de l'application du profil.
const DOSSIER: &str = "pilote";

/// Le fichier du journal, dans le dossier de `pilote`.
const JOURNAL: &str = "journal.tsv";

/// La variable que Claude Code pose dans les processus qu'une session lance.
const SESSION: &str = "CLAUDE_CODE_SESSION_ID";

/// Le chemin du journal : `pilote\journal.tsv` sous le dossier local de l'application du profil
/// (`FOLDERID_LocalAppData`), que Windows résout depuis `USERPROFILE`. Rien n'est créé.
pub fn chemin_journal() -> Result<PathBuf, ErreurSysteme> {
    // SAFETY: le GUID vit le temps de l'appel ; le pointeur rendu, alloué par Windows, se lit
    // avant d'être libéré une seule fois par `CoTaskMemFree`, comme la documentation le demande.
    let dossier = unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, None) }
        .map_err(|e| ErreurSysteme(format!("SHGetKnownFolderPath : {e}")))?;
    let lu = (!dossier.is_null()).then(|| OsString::from_wide(unsafe { dossier.as_wide() }));
    unsafe { CoTaskMemFree(Some(dossier.0.cast_const().cast())) };
    let racine = lu.ok_or_else(|| ErreurSysteme("SHGetKnownFolderPath : chemin nul".into()))?;
    Ok(PathBuf::from(racine).join(DOSSIER).join(JOURNAL))
}

/// Le journal, ouvert en ajout et créé s'il manque, son dossier compris. Un journal qu'une autre
/// application tient sans partage refuse l'ouverture.
pub fn ouvrir_journal() -> Result<File, ErreurSysteme> {
    let chemin = chemin_journal()?;
    if let Some(dossier) = chemin.parent() {
        create_dir_all(dossier)
            .map_err(|e| ErreurSysteme(format!("création du dossier du journal : {e}")))?;
    }
    OpenOptions::new()
        .append(true)
        .create(true)
        .open(&chemin)
        .map_err(|e| ErreurSysteme(format!("ouverture du journal : {e}")))
}

/// L'instant présent, en millisecondes depuis le démarrage du poste : une horloge monotone
/// (`GetTickCount64`), qui avance pendant `dormir`.
pub fn maintenant() -> Moment {
    // SAFETY: `GetTickCount64` ne prend rien et ne peut échouer.
    Moment(unsafe { GetTickCount64() })
}

/// Attend `duree`.
pub fn dormir(duree: Duree) {
    std::thread::sleep(Duration::from_millis(duree.0));
}

/// La date et l'heure locales, à la seconde (`GetLocalTime`).
pub fn date_locale() -> DateLocale {
    // SAFETY: `GetLocalTime` ne prend rien et ne peut échouer.
    let t = unsafe { GetLocalTime() };
    DateLocale {
        annee: t.wYear,
        mois: octet(t.wMonth),
        jour: octet(t.wDay),
        heure: octet(t.wHour),
        minute: octet(t.wMinute),
        seconde: octet(t.wSecond),
    }
}

/// Un champ de `SYSTEMTIME` sur un octet ; 255 s'il n'y tenait pas, ce que Windows ne rend pas.
fn octet(valeur: u16) -> u8 {
    u8::try_from(valeur).unwrap_or(u8::MAX)
}

/// La session de Claude Code : `CLAUDE_CODE_SESSION_ID`, `None` si elle est absente, vide ou
/// illisible.
pub fn session() -> Option<String> {
    std::env::var(SESSION).ok().filter(|s| !s.is_empty())
}

/// Rend `pilote` sensible au DPI de chaque écran, pour que les coordonnées qu'il lit soient
/// celles des pixels. Un refus, celui d'un réglage déjà fait compris, est sans suite.
pub fn sensible_au_dpi() {
    // SAFETY: la valeur est une constante que Windows définit.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le journal se résout sous le dossier local de l'application du profil, que la sentinelle de
    /// la CI redirige : un chemin résolu ailleurs écrirait dans le vrai profil (ADR-0003).
    #[ignore = "le garde-fou des chemins de données : la sentinelle de la CI le lance"]
    #[test]
    fn journal_tombe_dans_la_sentinelle() {
        let sentinelle = std::path::PathBuf::from(std::env::var_os("PILOTE_SENTINELLE").expect(
            "PILOTE_SENTINELLE manque : ce garde-fou ne tourne que dans la sentinelle de la CI",
        ));
        let chemin = chemin_journal().unwrap();
        assert!(
            chemin.starts_with(&sentinelle),
            "{chemin:?} hors de {sentinelle:?}"
        );
        assert_eq!(chemin, sentinelle.join(r"AppData\Local\pilote\journal.tsv"));
    }
}
