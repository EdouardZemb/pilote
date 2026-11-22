//! La couche Windows : `BureauWindows`, qui lit et agit pour le domaine sans rien décider, puis lui
//! rend des valeurs simples (ADR-0003). Les essais de bout en bout du runner Windows l'éprouvent.
//!
//! Aucune méthode ne panique : une lecture ratée devient une `ErreurSysteme`, `None`, ou la valeur
//! qui ne laisse rien passer (une disposition illisible ne produit aucun caractère). Toute ressource
//! de Windows ouverte ici se referme sur tous les chemins, par `Poignee` et par `Verrou`.

mod clavier;
mod fenetres;
mod gardes;
mod processus;

use crate::domaine::bureau::Bureau;
use crate::domaine::bureau::Duree;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::bureau::EtatSession;
use crate::domaine::bureau::EtatVerrou;
use crate::domaine::bureau::Evenement;
use crate::domaine::bureau::Moment;
use crate::domaine::bureau::Produit;
use crate::domaine::fenetre::FenetreLue;
use crate::domaine::fenetre::IdFenetre;
use crate::domaine::fenetre::PremierPlan;
use crate::domaine::journal::DateLocale;
use crate::domaine::processus::Processus;
use crate::domaine::texte::Disposition;
use crate::systeme;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::HWND;

/// Le poste réel : le journal ouvert, et le verrou des gestes une fois demandé, que `Drop` relâche
/// puis ferme, du fil qui l'a pris.
pub struct BureauWindows {
    journal: Option<File>,
    verrou: Option<gardes::Verrou>,
}

impl BureauWindows {
    /// Un bureau sans journal ouvert ni verrou demandé.
    pub fn new() -> BureauWindows {
        BureauWindows {
            journal: None,
            verrou: None,
        }
    }
}

impl Disposition for BureauWindows {
    fn scan(&mut self, unite: u16) -> i16 {
        clavier::scan(unite)
    }

    fn produit(&mut self, vk: u8, etat: u8) -> Produit {
        clavier::produit(vk, etat)
    }

    fn balayage(&mut self, vk: u8) -> u16 {
        clavier::balayage(vk)
    }
}

impl Bureau for BureauWindows {
    fn maintenant(&mut self) -> Moment {
        systeme::maintenant()
    }

    fn attendre(&mut self, duree: Duree) {
        systeme::dormir(duree);
    }

    fn date_locale(&mut self) -> DateLocale {
        systeme::date_locale()
    }

    fn session(&mut self) -> Option<String> {
        systeme::session()
    }

    fn ouvrir_journal(&mut self) -> Result<(), ErreurSysteme> {
        self.journal = Some(systeme::ouvrir_journal()?);
        Ok(())
    }

    fn ajouter_au_journal(&mut self, ligne: &str) -> Result<(), ErreurSysteme> {
        let journal = self
            .journal
            .as_mut()
            .ok_or_else(|| ErreurSysteme("le journal n'est pas ouvert".into()))?;
        journal
            .write_all(ligne.as_bytes())
            .map_err(|e| ErreurSysteme(format!("écriture du journal : {e}")))
    }

    fn lire_fichier(&mut self, chemin: &Path, max: usize) -> Result<Vec<u8>, ErreurSysteme> {
        let fichier =
            File::open(chemin).map_err(|e| ErreurSysteme(format!("lecture du fichier : {e}")))?;
        lire_au_plus(fichier, max).map_err(|e| ErreurSysteme(format!("lecture du fichier : {e}")))
    }

    fn lire_entree(&mut self, max: usize) -> Result<Vec<u8>, ErreurSysteme> {
        lire_au_plus(std::io::stdin().lock(), max)
            .map_err(|e| ErreurSysteme(format!("lecture de l'entrée standard : {e}")))
    }

    fn processus(&mut self) -> Result<Vec<Processus>, ErreurSysteme> {
        processus::processus()
    }

    fn fenetres(&mut self) -> Result<Vec<FenetreLue>, ErreurSysteme> {
        fenetres::fenetres()
    }

    fn premier_plan(&mut self) -> Option<PremierPlan> {
        fenetres::premier_plan()
    }

    fn est_reduite(&mut self, f: IdFenetre) -> bool {
        fenetres::est_reduite(f)
    }

    fn prendre_verrou(&mut self, delai: Duree) -> Result<EtatVerrou, ErreurSysteme> {
        let (etat, verrou) = gardes::prendre_verrou(delai)?;
        self.verrou = Some(verrou);
        Ok(etat)
    }

    fn etat_session(&mut self) -> EtatSession {
        gardes::etat_session()
    }

    fn niveau_integrite(&mut self, pid: Option<u32>) -> Option<u32> {
        processus::niveau_integrite(pid)
    }

    fn touches_enfoncees(&mut self) -> Vec<u8> {
        clavier::touches_enfoncees()
    }

    fn verr_maj(&mut self) -> bool {
        clavier::verr_maj()
    }

    fn restaurer(&mut self, f: IdFenetre) {
        fenetres::restaurer(f);
    }

    fn demander_premier_plan(&mut self, f: IdFenetre) {
        fenetres::demander_premier_plan(f);
    }

    fn envoyer(&mut self, lot: &[Evenement]) -> u32 {
        clavier::envoyer(lot)
    }
}

/// Les octets de `source`, jusqu'à sa fin ou jusqu'à `max` octets, au premier des deux : rien de
/// plus ne se lit, ni ne se garde en mémoire.
fn lire_au_plus(source: impl Read, max: usize) -> std::io::Result<Vec<u8>> {
    let mut octets = Vec::new();
    source
        .take(u64::try_from(max).unwrap_or(u64::MAX))
        .read_to_end(&mut octets)?;
    Ok(octets)
}

/// Un handle de Windows, fermé (`CloseHandle`) quand il sort de portée, sur tous les chemins.
struct Poignee(HANDLE);

impl Drop for Poignee {
    fn drop(&mut self) {
        // SAFETY: le handle a été ouvert par Windows pour cette seule `Poignee`, qui le ferme une
        // fois.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

/// L'identifiant de la fenêtre `h`.
fn id_de(h: HWND) -> IdFenetre {
    IdFenetre(h.0 as usize as u64)
}

/// La fenêtre `f`, telle que Windows la nomme.
fn hwnd_de(f: IdFenetre) -> HWND {
    HWND(f.0 as usize as *mut core::ffi::c_void)
}

/// L'identifiant de la fenêtre `h`, `None` si elle est nulle.
fn id_non_nul(h: HWND) -> Option<IdFenetre> {
    (!h.is_invalid()).then(|| id_de(h))
}

/// La taille de `T` en octets, telle que les structures de Windows la portent (`cbSize`,
/// `dwSize`).
fn taille<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

/// Le texte d'un tampon UTF-16, jusqu'au premier NUL ou à sa fin.
fn jusqu_au_nul(tampon: &[u16]) -> String {
    let fin = tampon.iter().position(|&c| c == 0).unwrap_or(tampon.len());
    String::from_utf16_lossy(&tampon[..fin])
}
