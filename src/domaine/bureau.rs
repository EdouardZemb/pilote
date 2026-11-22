//! Le bureau abstrait : tout ce que le domaine lit du poste, et tout ce qu'il y fait, passe par le
//! trait `Bureau` (ADR-0003). La couche Windows le réalise sans rien décider ; le faux bureau des
//! tests le joue sans fenêtre. Le temps y est `Moment` et `Duree`, des millisecondes : le domaine
//! ne lit jamais l'horloge lui-même, et n'attend que par `attendre`.

use crate::domaine::fenetre::FenetreLue;
use crate::domaine::fenetre::IdFenetre;
use crate::domaine::fenetre::PremierPlan;
use crate::domaine::journal::DateLocale;
use crate::domaine::processus::Processus;
use crate::domaine::texte::Disposition;
use std::path::Path;

pub use crate::domaine::texte::Produit;
pub use crate::domaine::touches::Evenement;

/// Un instant d'une horloge monotone, en millisecondes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment(pub u64);

/// Une durée, en millisecondes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duree(pub u64);

/// L'issue de la prise du verrou des gestes (`Local\pilote-gestes`) : pris ; repris d'un détenteur
/// mort sans le rendre (`WAIT_ABANDONED`, qui en donne la propriété) ; ou occupé au-delà du délai.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EtatVerrou {
    Pris,
    Repris,
    Occupe,
}

/// Les trois faits lus qui disent si le poste est verrouillé (décision 10) : le nom du bureau
/// d'entrée (`OpenInputDesktop`), puis les drapeaux et l'état de la session
/// (`WTSQuerySessionInformationW`, `WTSSessionInfoEx`). `None` : le fait ne se lit pas.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EtatSession {
    pub bureau_d_entree: Option<String>,
    pub drapeaux: Option<i32>,
    pub etat: Option<i32>,
}

/// Une lecture ou une action du système qui a échoué, et ce qui a échoué, en clair.
#[derive(Clone, Debug, PartialEq)]
pub struct ErreurSysteme(pub String);

/// Les drapeaux de la session : verrouillée (`WTS_SESSIONSTATE_LOCK`).
pub const WTS_VERROUILLE: i32 = 0;
/// Les drapeaux de la session : déverrouillée (`WTS_SESSIONSTATE_UNLOCK`).
pub const WTS_DEVERROUILLE: i32 = 1;
/// Les drapeaux de la session : inconnus (`WTS_SESSIONSTATE_UNKNOWN`, 0xFFFFFFFF).
pub const WTS_INCONNU: i32 = -1;
/// L'état de la session : active (`WTSActive`).
pub const WTS_ACTIF: i32 = 0;

/// Le poste, tel que le domaine le lit et y agit. La disposition du clavier du fil focalisé
/// (décision 2) en fait partie, par le super-trait `Disposition`.
pub trait Bureau: Disposition {
    /// L'instant présent, sur une horloge monotone.
    fn maintenant(&mut self) -> Moment;
    /// Attend `duree`.
    fn attendre(&mut self, duree: Duree);
    /// La date et l'heure locales, pour le journal.
    fn date_locale(&mut self) -> DateLocale;
    /// La session de Claude Code (`CLAUDE_CODE_SESSION_ID`), si la variable existe.
    fn session(&mut self) -> Option<String>;
    /// Ouvre le journal en ajout ; une erreur s'il ne s'ouvre pas, tenu par une autre application.
    fn ouvrir_journal(&mut self) -> Result<(), ErreurSysteme>;
    /// Ajoute `ligne`, retour final compris, au journal ouvert.
    fn ajouter_au_journal(&mut self, ligne: &str) -> Result<(), ErreurSysteme>;
    /// Les octets du fichier `chemin`.
    fn lire_fichier(&mut self, chemin: &Path) -> Result<Vec<u8>, ErreurSysteme>;
    /// Les octets de l'entrée standard, jusqu'à sa fin.
    fn lire_entree(&mut self) -> Result<Vec<u8>, ErreurSysteme>;
    /// L'instantané des processus.
    fn processus(&mut self) -> Result<Vec<Processus>, ErreurSysteme>;
    /// Les fenêtres de premier niveau.
    fn fenetres(&mut self) -> Result<Vec<FenetreLue>, ErreurSysteme>;
    /// La fenêtre au premier plan ; `None` si elle est nulle.
    fn premier_plan(&mut self) -> Option<PremierPlan>;
    /// Vrai si la fenêtre `f` est réduite.
    fn est_reduite(&mut self, f: IdFenetre) -> bool;
    /// Prend le verrou des gestes, en attendant au plus `delai`.
    fn prendre_verrou(&mut self, delai: Duree) -> Result<EtatVerrou, ErreurSysteme>;
    /// Les faits qui disent si le poste est verrouillé.
    fn etat_session(&mut self) -> EtatSession;
    /// Le niveau d'intégrité du processus `pid` ; `None` pour `pid` : celui de `pilote` lui-même.
    /// `None` en retour : le niveau ne se lit pas.
    fn niveau_integrite(&mut self, pid: Option<u32>) -> Option<u32>;
    /// Les touches de modification enfoncées, parmi `vk::MODIFICATEURS`.
    fn touches_enfoncees(&mut self) -> Vec<u8>;
    /// Vrai si Verr. Maj est allumée.
    fn verr_maj(&mut self) -> bool;
    /// Restaure la fenêtre `f`, sans attendre qu'elle le soit : `est_reduite` le dit ensuite.
    fn restaurer(&mut self, f: IdFenetre);
    /// Demande le premier plan pour la fenêtre `f`, sans savoir si Windows l'accorde.
    fn demander_premier_plan(&mut self, f: IdFenetre);
    /// Envoie `lot` d'un seul tenant (`SendInput`), et rend le nombre d'événements insérés.
    fn envoyer(&mut self, lot: &[Evenement]) -> u32;
}
