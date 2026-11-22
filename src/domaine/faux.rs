//! Le faux bureau des tests du domaine : il joue le poste sans fenêtre, note chaque appel dans
//! l'ordre, et se règle par ses champs publics. `avec_appli` le donne dans l'état d'un poste prêt :
//! l'application d'essai seule, au premier plan, rien qui refuse. Il réemploie les constructeurs
//! d'essai de `fenetre` et la disposition d'essai de `texte`.

use crate::domaine::bureau::Bureau;
use crate::domaine::bureau::Duree;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::bureau::EtatSession;
use crate::domaine::bureau::EtatVerrou;
use crate::domaine::bureau::Evenement;
use crate::domaine::bureau::Moment;
use crate::domaine::bureau::Produit;
use crate::domaine::bureau::WTS_ACTIF;
use crate::domaine::bureau::WTS_DEVERROUILLE;
use crate::domaine::fenetre::FenetreLue;
use crate::domaine::fenetre::IdFenetre;
use crate::domaine::fenetre::PremierPlan;
use crate::domaine::fenetre::essai::f;
use crate::domaine::fenetre::essai::p;
use crate::domaine::fenetre::essai::pp;
use crate::domaine::journal::DateLocale;
use crate::domaine::processus::Processus;
use crate::domaine::texte::Disposition;
use crate::domaine::texte::essai::DispositionEssai;
use crate::domaine::touches::vk;
use std::path::Path;
use std::path::PathBuf;

/// Une tenue des modificateurs qui ne finit jamais.
pub const TOUJOURS: Duree = Duree(u64::MAX);

/// L'instant où le temps simulé commence : ni nul, pour qu'une sonde qui lirait l'instant au lieu
/// de la durée écoulée se trompe.
pub const DEBUT: u64 = 10_000;

/// Un appel reçu par le faux bureau, dans l'ordre. `maintenant`, `date_locale`, `session` et la
/// disposition ne se notent pas : ils ne lisent ni n'agissent sur une fenêtre.
#[derive(Clone, Debug, PartialEq)]
pub enum AppelFaux {
    OuvrirJournal,
    Ajout(String),
    LireFichier,
    LireEntree,
    Processus,
    Fenetres,
    PremierPlan,
    EstReduite,
    PrendreVerrou(Duree),
    EtatSession,
    Niveau(Option<u32>),
    TouchesEnfoncees,
    VerrMaj,
    Restaurer(IdFenetre),
    Demande(IdFenetre),
    Envoi(Vec<Evenement>),
    Attendre(Duree),
}

/// Quand Windows accorde le premier plan demandé : jamais ; à toute demande qui suit un mouvement
/// de souris nul ; ou seulement à une demande qui suit l'envoi d'Alt entourant 0xE8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accord {
    Jamais,
    AuMouvementNul,
    ApresAlt,
}

/// La réduction de la fenêtre visée : aucune, jusqu'au premier `restaurer`, ou pour toujours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reduction {
    Aucune,
    JusquARestaurer,
    Toujours,
}

/// Le faux bureau. Ses réglages sont ses champs publics ; ses appels et les lignes reçues par le
/// journal se lisent dans `appels` et `journal`.
pub struct FauxBureau {
    /// Les appels reçus, dans l'ordre.
    pub appels: Vec<AppelFaux>,
    /// Les lignes que le journal a reçues.
    pub journal: Vec<String>,
    /// Le journal ne s'ouvre pas.
    pub journal_ferme: bool,
    /// Le journal s'ouvre, mais refuse chaque ligne.
    pub ajout_rate: bool,
    /// Le journal s'ouvre, mais panique à chaque ligne : une panique de la couche Windows pendant
    /// l'écriture de la ligne finale.
    pub ajout_panique: bool,
    /// Les processus du poste.
    pub processus: Vec<Processus>,
    /// Les fenêtres de premier niveau.
    pub fenetres: Vec<FenetreLue>,
    /// La lecture des processus échoue, avec ce texte.
    pub erreur_processus: Option<ErreurSysteme>,
    /// La lecture des fenêtres échoue, avec ce texte.
    pub erreur_fenetres: Option<ErreurSysteme>,
    /// La lecture des fenêtres panique.
    pub fenetres_panique: bool,
    /// Un processus, toujours dans l'instantané, dont une fenêtre enfant ne paraît sous la fenêtre
    /// 1 qu'à partir de la deuxième lecture des fenêtres : le refus rejugé après l'activation.
    pub enfant_tardif: Option<Processus>,
    /// Les fenêtres que rend chaque lecture à partir de la deuxième, au lieu de `fenetres`.
    pub relecture: Option<Vec<FenetreLue>>,
    /// La fenêtre au premier plan.
    pub premier_plan: Option<PremierPlan>,
    /// Le premier plan qui passe à cette fenêtre juste après le n-ième envoi.
    pub vol: Option<(usize, PremierPlan)>,
    /// Le premier plan qui passe à cette fenêtre pendant la lecture de l'instantané des processus
    /// qui suit le n-ième envoi : une lecture qui prend du temps sur le vrai poste.
    pub vol_pendant_l_instantane: Option<(usize, PremierPlan)>,
    /// Quand Windows accorde le premier plan demandé.
    pub accord: Accord,
    /// La réduction de la fenêtre visée.
    pub reduction: Reduction,
    /// Ce que rend la prise du verrou des gestes.
    pub verrou: Result<EtatVerrou, ErreurSysteme>,
    /// Les faits qui disent si le poste est verrouillé.
    pub session: EtatSession,
    /// Le niveau d'intégrité de tout processus visé.
    pub niveau_cible: Option<u32>,
    /// Le niveau d'intégrité de `pilote`.
    pub niveau_pilote: Option<u32>,
    /// Les touches enfoncées avant l'activation, c'est-à-dire avant la première restauration ou la
    /// première demande du premier plan.
    pub touches_avant: Vec<u8>,
    /// Les touches enfoncées après l'activation.
    pub touches_apres: Vec<u8>,
    /// Le temps simulé pendant lequel Alt gauche reste enfoncée après l'envoi d'Alt entourant
    /// 0xE8 ; `TOUJOURS` pour ne jamais la relâcher.
    pub alt_tenue: Duree,
    /// Le nombre d'événements qu'un envoi insère au plus ; tous sinon.
    pub insere_au_plus: Option<u32>,
    /// Verr. Maj est allumée.
    pub verr_maj: bool,
    /// Les fichiers lisibles et leurs octets ; tout autre fichier est illisible.
    pub fichiers: Vec<(PathBuf, Vec<u8>)>,
    /// Les octets de l'entrée standard ; `None` : illisible.
    pub entree: Option<Vec<u8>>,
    /// La date locale.
    pub date: DateLocale,
    /// La session de Claude Code.
    pub id_session: Option<String>,
    /// La disposition du clavier : l'AZERTY de la disposition d'essai, visible dans la crate
    /// comme elle.
    pub(crate) disposition: DispositionEssai,
    /// Le temps simulé, en millisecondes, que seul `attendre` avance.
    pub temps: u64,
    lectures_de_fenetres: usize,
    restauree: bool,
    activee: bool,
    souris_nulle: bool,
    alt_envoyee_a: Option<u64>,
    envois_faits: usize,
}

impl FauxBureau {
    /// Le poste prêt : le processus 100 « appli-essai.exe » et sa fenêtre 1 « Appli d'essai », au
    /// premier plan ; des niveaux d'intégrité égaux (0x2000) ; le bureau `Default`, la session
    /// déverrouillée et active ; le journal qui s'ouvre ; le verrou libre ; aucune touche
    /// enfoncée ; Verr. Maj éteinte ; l'AZERTY d'essai ; chaque envoi insérant tout son lot ; le
    /// premier plan accordé à toute demande qui suit un mouvement de souris nul.
    pub fn avec_appli() -> FauxBureau {
        FauxBureau {
            appels: Vec::new(),
            journal: Vec::new(),
            journal_ferme: false,
            ajout_rate: false,
            ajout_panique: false,
            processus: vec![p(100, 4, "appli-essai.exe", Some(5))],
            fenetres: vec![f(1, 100, "Appli d'essai")],
            erreur_processus: None,
            erreur_fenetres: None,
            fenetres_panique: false,
            enfant_tardif: None,
            relecture: None,
            premier_plan: Some(pp(1, 100, Vec::new())),
            vol: None,
            vol_pendant_l_instantane: None,
            accord: Accord::AuMouvementNul,
            reduction: Reduction::Aucune,
            verrou: Ok(EtatVerrou::Pris),
            session: EtatSession {
                bureau_d_entree: Some("Default".to_string()),
                drapeaux: Some(WTS_DEVERROUILLE),
                etat: Some(WTS_ACTIF),
            },
            niveau_cible: Some(0x2000),
            niveau_pilote: Some(0x2000),
            touches_avant: Vec::new(),
            touches_apres: Vec::new(),
            alt_tenue: Duree(0),
            insere_au_plus: None,
            verr_maj: false,
            fichiers: Vec::new(),
            entree: Some(Vec::new()),
            date: DateLocale {
                annee: 2026,
                mois: 11,
                jour: 22,
                heure: 10,
                minute: 20,
                seconde: 30,
            },
            id_session: Some("session-essai".to_string()),
            disposition: DispositionEssai::new(),
            temps: DEBUT,
            lectures_de_fenetres: 0,
            restauree: false,
            activee: false,
            souris_nulle: false,
            alt_envoyee_a: None,
            envois_faits: 0,
        }
    }

    /// Les lots envoyés, dans l'ordre.
    pub fn envois(&self) -> Vec<Vec<Evenement>> {
        self.appels
            .iter()
            .filter_map(|a| match a {
                AppelFaux::Envoi(lot) => Some(lot.clone()),
                _ => None,
            })
            .collect()
    }

    /// Le premier plan qu'accorde une demande pour `id` : cette fenêtre, sous le processus que lui
    /// donne `fenetres` (0 si elle n'y est pas), sans propriétaire.
    fn premier_plan_de(&self, id: IdFenetre) -> PremierPlan {
        let pid = self
            .fenetres
            .iter()
            .find(|f| f.id == id)
            .map_or(0, |f| f.pid);
        PremierPlan {
            id,
            pid,
            proprietaires: Vec::new(),
        }
    }
}

/// Vrai si `lot` est l'Alt entourant de l'activation : il enfonce Alt gauche et porte 0xE8.
fn est_alt_entourant(lot: &[Evenement]) -> bool {
    let porte = |cherchee: u8, relachee_cherchee: bool| {
        lot.iter().any(|e| {
            matches!(*e, Evenement::Touche { vk, relachee, .. }
                if vk == cherchee && relachee == relachee_cherchee)
        })
    };
    porte(vk::ALT_G, false) && porte(vk::SANS_EFFET, false)
}

impl Disposition for FauxBureau {
    fn scan(&mut self, unite: u16) -> i16 {
        self.disposition.scan(unite)
    }
    fn produit(&mut self, vk: u8, etat: u8) -> Produit {
        self.disposition.produit(vk, etat)
    }
    fn balayage(&mut self, vk: u8) -> u16 {
        self.disposition.balayage(vk)
    }
}

impl Bureau for FauxBureau {
    fn maintenant(&mut self) -> Moment {
        Moment(self.temps)
    }
    fn attendre(&mut self, duree: Duree) {
        self.appels.push(AppelFaux::Attendre(duree));
        self.temps = self.temps.saturating_add(duree.0);
    }
    fn date_locale(&mut self) -> DateLocale {
        self.date
    }
    fn session(&mut self) -> Option<String> {
        self.id_session.clone()
    }
    fn ouvrir_journal(&mut self) -> Result<(), ErreurSysteme> {
        self.appels.push(AppelFaux::OuvrirJournal);
        if self.journal_ferme {
            Err(ErreurSysteme("partage refusé".to_string()))
        } else {
            Ok(())
        }
    }
    fn ajouter_au_journal(&mut self, ligne: &str) -> Result<(), ErreurSysteme> {
        self.appels.push(AppelFaux::Ajout(ligne.to_string()));
        if self.ajout_panique {
            panic!("ajout au journal");
        }
        if self.ajout_rate {
            return Err(ErreurSysteme("écriture refusée".to_string()));
        }
        self.journal.push(ligne.to_string());
        Ok(())
    }
    fn lire_fichier(&mut self, chemin: &Path) -> Result<Vec<u8>, ErreurSysteme> {
        self.appels.push(AppelFaux::LireFichier);
        self.fichiers
            .iter()
            .find(|(nom, _)| nom == chemin)
            .map(|(_, octets)| octets.clone())
            .ok_or_else(|| ErreurSysteme("fichier introuvable".to_string()))
    }
    fn lire_entree(&mut self) -> Result<Vec<u8>, ErreurSysteme> {
        self.appels.push(AppelFaux::LireEntree);
        self.entree
            .clone()
            .ok_or_else(|| ErreurSysteme("entrée fermée".to_string()))
    }
    fn processus(&mut self) -> Result<Vec<Processus>, ErreurSysteme> {
        self.appels.push(AppelFaux::Processus);
        if let Some(e) = &self.erreur_processus {
            return Err(e.clone());
        }
        if let Some((n, vole)) = &self.vol_pendant_l_instantane
            && *n == self.envois_faits
        {
            self.premier_plan = Some(vole.clone());
        }
        Ok(self
            .processus
            .iter()
            .chain(&self.enfant_tardif)
            .cloned()
            .collect())
    }
    fn fenetres(&mut self) -> Result<Vec<FenetreLue>, ErreurSysteme> {
        self.appels.push(AppelFaux::Fenetres);
        if self.fenetres_panique {
            panic!("lecture des fenêtres");
        }
        if let Some(e) = &self.erreur_fenetres {
            return Err(e.clone());
        }
        self.lectures_de_fenetres += 1;
        if self.lectures_de_fenetres < 2 {
            return Ok(self.fenetres.clone());
        }
        let mut lues = self
            .relecture
            .clone()
            .unwrap_or_else(|| self.fenetres.clone());
        if let Some(enfant) = &self.enfant_tardif {
            for w in lues.iter_mut().filter(|w| w.id == IdFenetre(1)) {
                w.pids_enfants.push(enfant.pid);
            }
        }
        Ok(lues)
    }
    fn premier_plan(&mut self) -> Option<PremierPlan> {
        self.appels.push(AppelFaux::PremierPlan);
        self.premier_plan.clone()
    }
    fn est_reduite(&mut self, _f: IdFenetre) -> bool {
        self.appels.push(AppelFaux::EstReduite);
        match self.reduction {
            Reduction::Aucune => false,
            Reduction::JusquARestaurer => !self.restauree,
            Reduction::Toujours => true,
        }
    }
    fn prendre_verrou(&mut self, delai: Duree) -> Result<EtatVerrou, ErreurSysteme> {
        self.appels.push(AppelFaux::PrendreVerrou(delai));
        self.verrou.clone()
    }
    fn etat_session(&mut self) -> EtatSession {
        self.appels.push(AppelFaux::EtatSession);
        self.session.clone()
    }
    fn niveau_integrite(&mut self, pid: Option<u32>) -> Option<u32> {
        self.appels.push(AppelFaux::Niveau(pid));
        match pid {
            Some(_) => self.niveau_cible,
            None => self.niveau_pilote,
        }
    }
    fn touches_enfoncees(&mut self) -> Vec<u8> {
        self.appels.push(AppelFaux::TouchesEnfoncees);
        let mut enfoncees = if self.activee {
            self.touches_apres.clone()
        } else {
            self.touches_avant.clone()
        };
        let alt_tenue = self
            .alt_envoyee_a
            .is_some_and(|envoi| self.temps < envoi.saturating_add(self.alt_tenue.0));
        if alt_tenue {
            enfoncees.push(vk::ALT_G);
        }
        enfoncees
    }
    fn verr_maj(&mut self) -> bool {
        self.appels.push(AppelFaux::VerrMaj);
        self.verr_maj
    }
    fn restaurer(&mut self, f: IdFenetre) {
        self.appels.push(AppelFaux::Restaurer(f));
        self.restauree = true;
        self.activee = true;
    }
    fn demander_premier_plan(&mut self, f: IdFenetre) {
        self.appels.push(AppelFaux::Demande(f));
        self.activee = true;
        let accorde = match self.accord {
            Accord::Jamais => false,
            Accord::AuMouvementNul => self.souris_nulle,
            Accord::ApresAlt => self.alt_envoyee_a.is_some(),
        };
        if accorde {
            self.premier_plan = Some(self.premier_plan_de(f));
        }
    }
    fn envoyer(&mut self, lot: &[Evenement]) -> u32 {
        self.appels.push(AppelFaux::Envoi(lot.to_vec()));
        self.envois_faits += 1;
        if lot.contains(&Evenement::SourisNulle) {
            self.souris_nulle = true;
        }
        if est_alt_entourant(lot) {
            self.alt_envoyee_a = Some(self.temps);
        }
        if let Some((n, vole)) = &self.vol
            && *n == self.envois_faits
        {
            self.premier_plan = Some(vole.clone());
        }
        let tout = u32::try_from(lot.len()).unwrap_or(u32::MAX);
        self.insere_au_plus.map_or(tout, |n| n.min(tout))
    }
}
