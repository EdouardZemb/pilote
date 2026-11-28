//! L'exécution d'une commande : l'analyse, le journal, puis, pour un geste (`activer`, `touche`,
//! `taper`), les gardes dans leur ordre, l'activation et l'envoi par unités, le premier plan
//! vérifié avant chacune (ADR-0005, « Les gestes et l'ordre des gardes »). Tout se décide ici
//! contre le trait `Bureau`.
//!
//! L'ordre, chaque section citée étant d'ADR-0005 : l'analyse, qui ne lit aucune fenêtre ; le
//! journal, qu'exige toute commande valide ; la fenêtre retenue, avec la liste de refus et la règle
//! de KeePassXC (« KeePassXC ») ; le verrou des gestes ; le poste verrouillé (« Le poste
//! verrouillé ») ; le niveau d'intégrité (« Le niveau d'intégrité se juge par comparaison ») ; les
//! touches enfoncées ; Verr. Maj, pour `taper --comme-au-clavier` seulement ; l'activation
//! (« L'activation ») ; le refus et la règle de KeePassXC rejugés sur W relue ; les touches
//! enfoncées de nouveau ; le calcul de tous les lots ; puis, pour chaque unité, le premier plan
//! accepté (« Le premier plan accepté »), l'envoi et la cadence (« La cadence »). Un refus d'avant
//! l'activation n'a rien restauré, rien demandé, rien envoyé.

use crate::domaine::bureau::Bureau;
use crate::domaine::bureau::Duree;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::bureau::EtatSession;
use crate::domaine::bureau::EtatVerrou;
use crate::domaine::bureau::Evenement;
use crate::domaine::bureau::WTS_ACTIF;
use crate::domaine::bureau::WTS_VERROUILLE;
use crate::domaine::cli::Cible;
use crate::domaine::cli::Commande;
use crate::domaine::cli::Source;
use crate::domaine::cli::analyser;
use crate::domaine::fenetre::FenetreLue;
use crate::domaine::fenetre::IdFenetre;
use crate::domaine::fenetre::PremierPlan;
use crate::domaine::fenetre::Retenue;
use crate::domaine::fenetre::accepte;
use crate::domaine::fenetre::aucune_fenetre;
use crate::domaine::fenetre::ligne_de;
use crate::domaine::fenetre::lister;
use crate::domaine::fenetre::refus_du_processus;
use crate::domaine::fenetre::retenir;
use crate::domaine::journal::Appel;
use crate::domaine::journal::ligne;
use crate::domaine::journal::sous_commande;
use crate::domaine::processus::Processus;
use crate::domaine::processus::sans_exe;
use crate::domaine::refus::fenetre_refusee;
use crate::domaine::refus::keepassxc_dans;
use crate::domaine::sortie::Arret;
use crate::domaine::sortie::Code;
use crate::domaine::sortie::Issue;
use crate::domaine::sortie::echapper;
use crate::domaine::sortie::formater;
use crate::domaine::texte::LECTURE_MAX;
use crate::domaine::texte::Unite;
use crate::domaine::texte::admettre;
use crate::domaine::texte::decoder;
use crate::domaine::texte::lot_clavier;
use crate::domaine::texte::lot_unicode;
use crate::domaine::texte::unites;
use crate::domaine::touches::Combinaison;
use crate::domaine::touches::analyser_touche;
use crate::domaine::touches::caractere;
use crate::domaine::touches::evenement;
use crate::domaine::touches::lot_combinaison;
use crate::domaine::touches::lot_touches;
use crate::domaine::touches::refuser_l_interdite;
use crate::domaine::touches::vk;
use std::ffi::OsString;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

/// Le délai d'attente du verrou des gestes.
pub const ATTENTE_DU_VERROU: Duree = Duree(5_000);

/// L'attente après chaque unité frappée, avant de vérifier le premier plan pour la suivante
/// (ADR-0005, « La cadence ») : sans elle, des milliers d'événements resteraient en file et
/// partiraient vers une fenêtre qui aurait volé le premier plan après le dernier contrôle.
pub const CADENCE: Duree = Duree(5);

/// Le pas des sondes de la restauration et du premier plan.
const PAS_DES_SONDES: Duree = Duree(100);

/// La plus longue attente d'une fenêtre restaurée.
const RESTAURATION_MAX: Duree = Duree(2_000);

/// La plus longue attente du premier plan, après chaque demande.
const PREMIER_PLAN_MAX: Duree = Duree(1_000);

/// Le pas de la sonde des modificateurs, après l'envoi d'Alt entourant.
const PAS_DES_MODIFICATEURS: Duree = Duree(10);

/// La plus longue attente du relâchement des modificateurs, après l'envoi d'Alt entourant.
const MODIFICATEURS_MAX: Duree = Duree(500);

/// Le seul bureau d'entrée d'un poste déverrouillé.
const BUREAU_PAR_DEFAUT: &str = "Default";

const JOURNAL_INACCESSIBLE: &str = "journal inaccessible : ouvert dans une autre application ?";
const KEEPASSXC: &str = "KeePassXC : touche et taper sont refusés dans cette version";
const VERROU_OCCUPE: &str = "un autre geste est en cours depuis 5 s";
const POSTE_VERROUILLE: &str = "poste verrouillé";
const NIVEAU_PLUS_HAUT: &str = "fenêtre plus élevée que pilote";
const NIVEAU_ILLISIBLE: &str = "niveau d'intégrité illisible";
const MODIFICATEUR_ENFONCE: &str = "touche de modification enfoncée";
const VERR_MAJ: &str = "Verr. Maj est allumée";
const PREMIER_PLAN_REFUSE: &str = "Windows a refusé le premier plan";
const LIGNE_PERDUE: &str = "le journal n'a pas reçu sa ligne";

/// Ce que demande une commande valide, une fois analysée.
enum Travail {
    /// `fenetres` : une lecture, sans verrou ni activation.
    Lister,
    /// `activer`, `touche` ou `taper` : la fenêtre visée, et ce qui s'y frappe, rien pour
    /// `activer`.
    Geste {
        cible: Cible,
        frappe: Option<Frappe>,
    },
    /// Une sous-commande que clap accepte et que la tranche ne livre pas, par son nom : une
    /// commande valide, qui exige le journal avant de sortir en 64 (ADR-0006, « L'analyse »).
    PasEncoreLivree(&'static str),
}

/// Ce que frappe un geste.
enum Frappe {
    /// `touche` : les combinaisons, dans l'ordre, et le nombre de fois que leur suite se frappe.
    Touches {
        combinaisons: Vec<Combinaison>,
        fois: u32,
    },
    /// `taper` : les unités du texte, et si elles passent par la disposition du clavier.
    Texte {
        unites: Vec<Unite>,
        par_la_disposition: bool,
    },
}

/// L'état du journal pendant une commande.
#[derive(Clone, Copy, Default)]
enum Journal {
    /// Pas encore ouvert : l'analyse n'en a pas besoin.
    #[default]
    PasOuvert,
    /// Ouvert : la commande valide l'a exigé, et il l'a été.
    Ouvert,
    /// Exigé, il ne s'est pas ouvert : aucune ligne ne peut y aller.
    Inaccessible,
}

/// Ce que la ligne du journal garde d'une commande, au fil de son exécution, et si elle a agi sur
/// le poste.
#[derive(Default)]
struct Suivi {
    journal: Journal,
    /// Le nom, sans `.exe`, du processus de la fenêtre retenue : un nom lu du système, jamais un
    /// texte de la ligne de commande.
    processus: Option<String>,
    /// La longueur du texte de `taper`, en caractères : jamais le texte.
    longueur: Option<usize>,
    /// Vrai dès qu'une restauration, une demande du premier plan ou un envoi est parti.
    agi: bool,
}

impl Suivi {
    /// Restaure la fenêtre `f`.
    fn restaurer(&mut self, b: &mut dyn Bureau, f: IdFenetre) {
        self.agi = true;
        b.restaurer(f);
    }

    /// Demande le premier plan pour la fenêtre `f`.
    fn demander(&mut self, b: &mut dyn Bureau, f: IdFenetre) {
        self.agi = true;
        b.demander_premier_plan(f);
    }

    /// Envoie `lot` d'un seul tenant (`SendInput`), et rend le nombre d'événements que Windows en a
    /// insérés : tous, aucun quand UIPI le bloque, sans erreur, ou un début du lot.
    fn envoyer(&mut self, b: &mut dyn Bureau, lot: &[Evenement]) -> u32 {
        self.agi = true;
        b.envoyer(lot)
    }

    /// Envoie `lot`, le mouvement nul ou l'Alt entourant de l'activation, d'un seul tenant. Un lot
    /// que `SendInput` n'insère pas du tout ne sort pas : UIPI le bloque sans erreur quand une
    /// fenêtre plus élevée a le premier plan, un refus prévisible de Windows et non un échec de
    /// l'outil, et l'activation se juge alors au premier plan relu (1, « Windows a refusé le
    /// premier plan », s'il ne vient pas). Une insertion partielle reste un échec (3), après le
    /// relâchement tenté de ce qu'elle a enfoncé (`insertion_complete`). L'envoi compte comme un
    /// geste dans tous les cas.
    fn envoyer_pour_activer(&mut self, b: &mut dyn Bureau, lot: &[Evenement]) -> Result<(), Arret> {
        match self.envoyer(b, lot) {
            0 => Ok(()),
            inseres => insertion_complete(b, lot, inseres),
        }
    }
}

/// `Ok` si `SendInput` a inséré les `inseres` événements de `lot`, tous ; sinon 3, « SendInput a
/// inséré <k> événements sur <n> ». `SendInput` insère dans l'ordre : une insertion partielle a
/// enfoncé, dans le début inséré du lot, des touches que la fin devait relâcher, et qui resteraient
/// enfoncées pour tout le poste. Leurs relâchements (`relachements`) partent alors d'un seul envoi,
/// sans garde du premier plan, puisqu'ils ne tapent rien, et le message dit combien Windows en a
/// insérés : « ; relâchement : <m> sur <p> ». Le relâchement est tenté, sans garantie : ce qui a
/// refusé la fin du lot, comme un bureau sécurisé, peut le refuser aussi. Quand le début inséré
/// n'a rien laissé enfoncé, rien ne part, et le message s'arrête à l'envoi.
fn insertion_complete(b: &mut dyn Bureau, lot: &[Evenement], inseres: u32) -> Result<(), Arret> {
    let k = usize::try_from(inseres).unwrap_or(usize::MAX);
    if k == lot.len() {
        return Ok(());
    }
    let envoi = format!("SendInput a inséré {inseres} événements sur {}", lot.len());
    let a_relacher = lot
        .get(..k)
        .map(|debut| relachements(debut, &mut |v| b.balayage(v)))
        .unwrap_or_default();
    if a_relacher.is_empty() {
        return Err(echec(&envoi));
    }
    let relaches = b.envoyer(&a_relacher);
    Err(echec(&format!(
        "{envoi} ; relâchement : {relaches} sur {}",
        a_relacher.len()
    )))
}

/// Les relâchements de ce que `debut`, le début inséré d'un lot, a enfoncé sans le relâcher :
/// chaque touche et chaque unité UTF-16 restées enfoncées, relâchées dans l'ordre inverse de leur
/// enfoncement, une touche avec le balayage et l'extension de son enfoncement. Quand une Alt, gauche
/// ou droite, est à relâcher, 0xE8, que Windows n'attribue à rien, est d'abord enfoncée puis
/// relâchée : une Alt relâchée seule ouvrirait le mode menu de la fenêtre au premier plan. Cette
/// frappe de 0xE8 relâche aussi 0xE8, si le début l'a laissée enfoncée.
fn relachements(debut: &[Evenement], balayage: &mut dyn FnMut(u8) -> u16) -> Vec<Evenement> {
    let mut enfoncees: Vec<Evenement> = Vec::new();
    for e in debut {
        match *e {
            Evenement::Touche {
                relachee: false, ..
            }
            | Evenement::Unicode {
                relachee: false, ..
            } => enfoncees.push(*e),
            Evenement::Touche { .. } | Evenement::Unicode { .. } => {
                if let Some(i) = enfoncees.iter().rposition(|x| meme_touche(x, e)) {
                    enfoncees.remove(i);
                }
            }
            Evenement::SourisNulle => {}
        }
    }
    let mut lot = Vec::new();
    let alt = |e: &Evenement| matches!(*e, Evenement::Touche { vk, .. } if vk == vk::ALT_G || vk == vk::ALT_D);
    if enfoncees.iter().any(alt) {
        let sans_effet = balayage(vk::SANS_EFFET);
        lot.push(evenement(vk::SANS_EFFET, sans_effet, false));
        lot.push(evenement(vk::SANS_EFFET, sans_effet, true));
        enfoncees.retain(|e| !meme_touche(e, &lot[0]));
    }
    lot.extend(enfoncees.iter().rev().map(|&e| relachee(e)));
    lot
}

/// Vrai si `a` et `b` portent la même touche virtuelle, ou la même unité UTF-16, enfoncée ou
/// relâchée.
fn meme_touche(a: &Evenement, b: &Evenement) -> bool {
    match (*a, *b) {
        (Evenement::Touche { vk: x, .. }, Evenement::Touche { vk: y, .. }) => x == y,
        (Evenement::Unicode { unite: x, .. }, Evenement::Unicode { unite: y, .. }) => x == y,
        _ => false,
    }
}

/// L'événement `e`, relâché.
fn relachee(e: Evenement) -> Evenement {
    match e {
        Evenement::Touche {
            vk, scan, etendue, ..
        } => Evenement::Touche {
            vk,
            scan,
            etendue,
            relachee: true,
        },
        Evenement::Unicode { unite, .. } => Evenement::Unicode {
            unite,
            relachee: true,
        },
        Evenement::SourisNulle => Evenement::SourisNulle,
    }
}

/// Exécute la commande que disent `args`, le nom du programme en tête, et rend son issue. Une
/// panique se rattrape : sa ligne va au journal au code 3, et l'issue est la seule erreur
/// « échec du système : panique ». Chaque appel a sa ligne au journal quand il s'ouvre ; une ligne
/// qu'il refuse après un geste fait sortir en 3, sauf si la commande échouait déjà en 3. Une
/// panique pendant l'écriture de cette ligne se rattrape aussi : la ligne est perdue, comme une
/// ligne refusée, et aucune panique ne sort d'ici.
pub fn executer(args: &[OsString], b: &mut dyn Bureau) -> Issue {
    let mut suivi = Suivi::default();
    let issue = catch_unwind(AssertUnwindSafe(|| derouler(args, &mut suivi, &mut *b)))
        .unwrap_or_else(|_| Issue::arret(echec("panique")));
    let code = issue.code;
    let recue = catch_unwind(AssertUnwindSafe(|| {
        journaliser(sous_commande(args), code, &suivi, &mut *b)
    }))
    .unwrap_or(false);
    if !recue && suivi.agi && code != Code::Echec {
        return Issue::arret(echec(LIGNE_PERDUE));
    }
    issue
}

/// Le corps d'`executer`, hors du journal final : l'analyse, puis le journal, qu'exige toute
/// commande valide, puis son travail : `fenetres`, un geste, ou le 64 d'une sous-commande pas
/// encore livrée.
fn derouler(args: &[OsString], s: &mut Suivi, b: &mut dyn Bureau) -> Issue {
    // 1. L'analyse (ADR-0006, « L'analyse ») : clap, les touches de `touche`, le texte de
    // `taper`. Rien n'y lit une fenêtre, et son refus garde son code, que le journal s'ouvre ou
    // non.
    let travail = match analyser_le_travail(args, s, b) {
        Ok(travail) => travail,
        Err(issue) => return issue,
    };
    // 2. Le journal, qu'exige toute commande valide, avant toute lecture de fenêtre et tout geste.
    if b.ouvrir_journal().is_err() {
        s.journal = Journal::Inaccessible;
        return Issue::arret(Arret::Refus(JOURNAL_INACCESSIBLE.into()));
    }
    s.journal = Journal::Ouvert;
    let fait = match travail {
        Travail::Lister => lister_les_fenetres(b),
        Travail::Geste { cible, frappe } => geste(&cible, frappe.as_ref(), s, b),
        // Une sous-commande pas encore livrée : 64, le journal ouvert, qui reçoit sa ligne.
        Travail::PasEncoreLivree(nom) => Err(Arret::Usage(format!("pas encore livrée : {nom}"))),
    };
    fait.unwrap_or_else(Issue::arret)
}

/// L'analyse : la commande de clap, puis les combinaisons de `touche`, les interdites de tous les
/// arguments jugées avant aucun nom, puis le texte de `taper`, lu, décodé et jugé quelle que soit
/// sa source. L'aide et la version rendent leur texte, ligne à
/// ligne, au code 0. Une sous-commande que la tranche ne livre pas est une commande valide : elle
/// rend son travail à part, que `derouler` juge après le journal.
fn analyser_le_travail(
    args: &[OsString],
    s: &mut Suivi,
    b: &mut dyn Bureau,
) -> Result<Travail, Issue> {
    let commande = analyser(args).map_err(|aide| match aide.code {
        Code::Fait => Issue::fait(aide.texte.lines().map(String::from).collect()),
        _ => Issue::arret(Arret::Usage(aide.texte)),
    })?;
    match commande {
        Commande::Fenetres => Ok(Travail::Lister),
        Commande::Activer(cible) => Ok(Travail::Geste {
            cible,
            frappe: None,
        }),
        Commande::Touche {
            cible,
            touches,
            fois,
        } => {
            // Les interdites de tous les arguments, puis les noms : une touche inconnue placée
            // avant une interdite ne la cache pas (1 avant 64).
            for t in &touches {
                refuser_l_interdite(t).map_err(Issue::arret)?;
            }
            let combinaisons = touches
                .iter()
                .map(|t| analyser_touche(t))
                .collect::<Result<Vec<Combinaison>, Arret>>()
                .map_err(Issue::arret)?;
            Ok(Travail::Geste {
                cible,
                frappe: Some(Frappe::Touches { combinaisons, fois }),
            })
        }
        Commande::Taper {
            cible,
            source,
            comme_au_clavier,
        } => {
            let texte = lire_le_texte(source, b).map_err(Issue::arret)?;
            s.longueur = Some(texte.chars().count());
            Ok(Travail::Geste {
                cible,
                frappe: Some(Frappe::Texte {
                    unites: unites(&texte),
                    par_la_disposition: comme_au_clavier,
                }),
            })
        }
        autre => Ok(Travail::PasEncoreLivree(autre.nom())),
    }
}

/// Le texte de `taper` : `--texte` jugé (`admettre`) ; un fichier ou l'entrée standard, lus
/// `LECTURE_MAX` octets au plus, décodés et jugés (`decoder`). Une lecture ratée est un usage (64),
/// le chemin échappé.
fn lire_le_texte(source: Source, b: &mut dyn Bureau) -> Result<String, Arret> {
    match source {
        Source::Texte(texte) => admettre(texte),
        Source::Fichier(chemin) => {
            let octets = b.lire_fichier(&chemin, LECTURE_MAX).map_err(|_| {
                Arret::Usage(format!(
                    "fichier illisible : {}",
                    echapper(&chemin.display().to_string())
                ))
            })?;
            decoder(&octets)
        }
        Source::Entree => {
            let octets = b
                .lire_entree(LECTURE_MAX)
                .map_err(|_| Arret::Usage("entrée standard illisible".into()))?;
            decoder(&octets)
        }
    }
}

/// Les fenêtres, puis l'instantané des processus, lu après elles : il voit ainsi chaque processus
/// d'une fenêtre lue. Une lecture ratée sort en 3, son texte échappé.
fn lire_fenetres_et_processus(
    b: &mut dyn Bureau,
) -> Result<(Vec<FenetreLue>, Vec<Processus>), Arret> {
    let fenetres = b.fenetres().map_err(echec_systeme)?;
    let tous = b.processus().map_err(echec_systeme)?;
    Ok((fenetres, tous))
}

/// `fenetres` : les lignes des fenêtres candidates, sans verrou ni activation.
fn lister_les_fenetres(b: &mut dyn Bureau) -> Result<Issue, Arret> {
    let (fenetres, tous) = lire_fenetres_et_processus(b)?;
    let pp = b.premier_plan();
    Ok(Issue::fait(
        lister(&fenetres, &tous, pp.as_ref())
            .iter()
            .map(formater)
            .collect(),
    ))
}

/// Un geste, de la fenêtre retenue au dernier envoi.
fn geste(
    cible: &Cible,
    frappe: Option<&Frappe>,
    s: &mut Suivi,
    b: &mut dyn Bureau,
) -> Result<Issue, Arret> {
    // 3. La fenêtre retenue, la liste de refus comprise, puis la règle de KeePassXC.
    let (fenetres, tous) = lire_fenetres_et_processus(b)?;
    let r = retenir(cible, &fenetres, &tous)?;
    s.processus = Some(sans_exe(&r.processus.nom).to_string());
    refuser_keepassxc(frappe.is_some(), &r.fenetre, &tous)?;
    // 4. Le verrou des gestes.
    match b.prendre_verrou(ATTENTE_DU_VERROU).map_err(echec_systeme)? {
        EtatVerrou::Occupe => return Err(Arret::Refus(VERROU_OCCUPE.into())),
        EtatVerrou::Pris | EtatVerrou::Repris => {}
    }
    // 5. Le poste verrouillé.
    if poste_verrouille(&b.etat_session()) {
        return Err(Arret::Refus(POSTE_VERROUILLE.into()));
    }
    // 6. Le niveau d'intégrité.
    juger_le_niveau(&r, b)?;
    // 7. Les touches enfoncées.
    exiger_les_modificateurs_relaches(b)?;
    // 8. Verr. Maj, pour `taper --comme-au-clavier` seulement.
    let par_la_disposition = matches!(
        frappe,
        Some(Frappe::Texte {
            par_la_disposition: true,
            ..
        })
    );
    if par_la_disposition && b.verr_maj() {
        return Err(Arret::Refus(VERR_MAJ.into()));
    }
    // 9. L'activation.
    let pp = activer(&r, s, b)?;
    // 10. Le refus et la règle de KeePassXC, rejugés sur W relue. `activer` s'arrête là, avec la
    // ligne de W : rien ne se frappe après lui.
    let (w, tous) = rejuger(cible, &r, frappe.is_some(), b)?;
    let Some(frappe) = frappe else {
        return Ok(Issue::fait(vec![formater(&ligne_de(
            &w,
            &tous,
            pp.id == w.id,
        ))]));
    };
    // 11. Les touches enfoncées, de nouveau.
    exiger_les_modificateurs_relaches(b)?;
    // 12. Tous les lots, avant le premier envoi : un caractère refusé n'en laisse partir aucun.
    let lots = calculer_les_lots(frappe, b)?;
    // 13. Pour chaque unité : le premier plan accepté, l'envoi, la cadence. Un envoi que Windows
    // n'insère pas du tout se juge au premier plan relu : UIPI bloque sans erreur l'envoi vers une
    // fenêtre plus élevée qui vient de prendre le premier plan, un refus prévisible de Windows, et
    // non un échec de l'outil. Le premier plan encore accepté, l'échec reste 3.
    for (parties, lot) in lots.iter().enumerate() {
        let change = || {
            Arret::Refus(format!(
                "le premier plan a changé : {parties} unités sur {} sont parties",
                lots.len()
            ))
        };
        if premier_plan_accepte(&r, b)?.is_none() {
            return Err(change());
        }
        let inseres = s.envoyer(b, lot);
        if inseres == 0 && premier_plan_accepte(&r, b)?.is_none() {
            return Err(change());
        }
        insertion_complete(b, lot, inseres)?;
        b.attendre(CADENCE);
    }
    Ok(Issue::fait(Vec::new()))
}

/// Le refus de KeePassXC (ADR-0005, « KeePassXC »), pour `touche` et `taper` (`frappe`) : quand le
/// processus de `w`, ou celui d'une de ses fenêtres enfants, est KeePassXC. `activer` vers lui
/// reste permis.
fn refuser_keepassxc(frappe: bool, w: &FenetreLue, tous: &[Processus]) -> Result<(), Arret> {
    if frappe && keepassxc_dans(w, tous) {
        return Err(Arret::Refus(KEEPASSXC.into()));
    }
    Ok(())
}

/// Vrai si le poste est verrouillé (ADR-0005, « Le poste verrouillé ») : le bureau d'entrée n'est
/// pas `Default` ou ne se lit pas ; les drapeaux lus disent « verrouillé » ; l'état lu n'est pas
/// « actif ». Des drapeaux inconnus ou illisibles, et un état illisible, ne verrouillent pas à eux
/// seuls.
fn poste_verrouille(s: &EtatSession) -> bool {
    s.bureau_d_entree.as_deref() != Some(BUREAU_PAR_DEFAUT)
        || s.drapeaux == Some(WTS_VERROUILLE)
        || s.etat.is_some_and(|etat| etat != WTS_ACTIF)
}

/// Le niveau d'intégrité (ADR-0005, « Le niveau d'intégrité se juge par comparaison ») : refus si
/// celui de P dépasse celui de `pilote`, ou si l'un des deux ne se lit pas.
fn juger_le_niveau(r: &Retenue, b: &mut dyn Bureau) -> Result<(), Arret> {
    let cible = b.niveau_integrite(Some(r.processus.pid));
    let pilote = b.niveau_integrite(None);
    match (cible, pilote) {
        (Some(cible), Some(pilote)) if cible > pilote => Err(Arret::Refus(NIVEAU_PLUS_HAUT.into())),
        (Some(_), Some(_)) => Ok(()),
        _ => Err(Arret::Refus(NIVEAU_ILLISIBLE.into())),
    }
}

/// Refus si une touche de modification est enfoncée : elle s'ajouterait à chaque touche frappée.
fn exiger_les_modificateurs_relaches(b: &mut dyn Bureau) -> Result<(), Arret> {
    if b.touches_enfoncees().is_empty() {
        Ok(())
    } else {
        Err(Arret::Refus(MODIFICATEUR_ENFONCE.into()))
    }
}

/// L'activation de W (ADR-0005, « L'activation »), qui rend le premier plan accepté. Une fenêtre
/// réduite se restaure, puis se sonde jusqu'à ne plus l'être : la restauration part sans attendre
/// la fenêtre (`ShowWindowAsync`), dont le retour ne dit pas qu'elle est faite. Rien de plus si le
/// premier plan est accepté. Sinon, un mouvement de souris nul, la demande, une sonde ; puis, en un
/// seul envoi, Alt gauche entourant 0xE8, que Windows n'attribue à rien, sans jamais frapper Alt
/// seule, qui ouvrirait le mode menu d'une fenêtre Win32 ; une sonde du relâchement des
/// modificateurs, refus sans demande s'ils restent enfoncés ; la demande, une sonde ; sinon, refus.
/// Un de ces deux envois que `SendInput` n'insère pas du tout ne fait pas sortir : l'activation
/// suit son cours et se juge au premier plan relu ; inséré en partie, il sort en 3, après le
/// relâchement tenté (`Suivi::envoyer_pour_activer`).
fn activer(r: &Retenue, s: &mut Suivi, b: &mut dyn Bureau) -> Result<PremierPlan, Arret> {
    let w = r.fenetre.id;
    let refus = || Arret::Refus(PREMIER_PLAN_REFUSE.into());
    if b.est_reduite(w) {
        s.restaurer(b, w);
        let restauree = sonder(b, PAS_DES_SONDES, RESTAURATION_MAX, |b: &mut dyn Bureau| {
            Ok((!b.est_reduite(w)).then_some(()))
        })?;
        if restauree.is_none() {
            return Err(refus());
        }
    }
    if let Some(pp) = premier_plan_accepte(r, b)? {
        return Ok(pp);
    }
    s.envoyer_pour_activer(b, &[Evenement::SourisNulle])?;
    s.demander(b, w);
    if let Some(pp) = attendre_le_premier_plan(r, b)? {
        return Ok(pp);
    }
    let alt = lot_touches(&[vk::ALT_G, vk::SANS_EFFET], &mut |v| b.balayage(v));
    s.envoyer_pour_activer(b, &alt)?;
    let relaches = sonder(
        b,
        PAS_DES_MODIFICATEURS,
        MODIFICATEURS_MAX,
        |b: &mut dyn Bureau| Ok(b.touches_enfoncees().is_empty().then_some(())),
    )?;
    if relaches.is_none() {
        return Err(Arret::Refus(MODIFICATEUR_ENFONCE.into()));
    }
    s.demander(b, w);
    attendre_le_premier_plan(r, b)?.ok_or_else(refus)
}

/// Le premier plan, s'il est accepté pour la retenue `r` (ADR-0005, « Le premier plan accepté »),
/// jugé sur l'instantané des processus relu juste après lui. L'instantané prend du temps sur le
/// vrai poste : le premier plan se relit après lui, et doit être le même qu'avant. C'est cette
/// dernière lecture qui se juge et se rend, pour que l'envoi qui suit parte juste après elle : une
/// fenêtre qui prend le premier plan pendant l'instantané ne reçoit rien.
fn premier_plan_accepte(r: &Retenue, b: &mut dyn Bureau) -> Result<Option<PremierPlan>, Arret> {
    let avant = b.premier_plan();
    let tous = b.processus().map_err(echec_systeme)?;
    let apres = b.premier_plan();
    Ok(apres.filter(|pp| avant.as_ref() == Some(pp) && accepte(Some(pp), r, &tous)))
}

/// La sonde du premier plan après une demande : toutes les 100 ms, 1 s au plus.
fn attendre_le_premier_plan(r: &Retenue, b: &mut dyn Bureau) -> Result<Option<PremierPlan>, Arret> {
    sonder(b, PAS_DES_SONDES, PREMIER_PLAN_MAX, |b: &mut dyn Bureau| {
        premier_plan_accepte(r, b)
    })
}

/// Une sonde bornée : `lire` tout de suite, puis après chaque `pas`, tant que moins de `borne` s'est
/// écoulé depuis le début ; la première valeur lue, ou `None` à la borne. Le temps est celui de
/// `maintenant`, si bien qu'une lecture lente compte dans la borne.
fn sonder<T>(
    b: &mut dyn Bureau,
    pas: Duree,
    borne: Duree,
    mut lire: impl FnMut(&mut dyn Bureau) -> Result<Option<T>, Arret>,
) -> Result<Option<T>, Arret> {
    let debut = b.maintenant();
    loop {
        if let Some(valeur) = lire(b)? {
            return Ok(Some(valeur));
        }
        if b.maintenant().0.saturating_sub(debut.0) >= borne.0 {
            return Ok(None);
        }
        b.attendre(pas);
    }
}

/// W relue après l'activation, et l'instantané des processus relu après elle : la fenêtre de même
/// HWND et de même processus que W, absente sinon (2). Le refus s'y rejuge, puis, pour `touche` et
/// `taper` (`frappe`), la règle de KeePassXC : une restauration montre des enfants qu'une fenêtre
/// réduite cachait. Les messages sont ceux de la retenue (`aucune_fenetre`, `refus_du_processus`).
fn rejuger(
    cible: &Cible,
    r: &Retenue,
    frappe: bool,
    b: &mut dyn Bureau,
) -> Result<(FenetreLue, Vec<Processus>), Arret> {
    let (fenetres, tous) = lire_fenetres_et_processus(b)?;
    let w = fenetres
        .into_iter()
        .find(|f| f.id == r.fenetre.id && f.pid == r.fenetre.pid)
        .ok_or_else(|| aucune_fenetre(cible))?;
    if let Some(nom) = fenetre_refusee(&w, &tous) {
        return Err(refus_du_processus(&nom));
    }
    refuser_keepassxc(frappe, &w, &tous)?;
    Ok((w, tous))
}

/// Les lots de la frappe, un par unité, dans l'ordre : pour `touche`, la suite des combinaisons,
/// `fois` fois ; pour `taper`, chaque unité, par la disposition du clavier ou en UTF-16.
fn calculer_les_lots(frappe: &Frappe, b: &mut dyn Bureau) -> Result<Vec<Vec<Evenement>>, Arret> {
    match frappe {
        Frappe::Touches { combinaisons, fois } => {
            let suite = combinaisons
                .iter()
                .map(|c| lot_de_combinaison(c, b))
                .collect::<Result<Vec<_>, Arret>>()?;
            let mut lots = Vec::new();
            for _ in 0..*fois {
                lots.extend(suite.iter().cloned());
            }
            Ok(lots)
        }
        Frappe::Texte {
            unites,
            par_la_disposition: true,
        } => unites
            .iter()
            .map(|&u| lot_clavier(caractere_de(u), b))
            .collect(),
        Frappe::Texte {
            unites,
            par_la_disposition: false,
        } => Ok(unites
            .iter()
            .map(|&u| lot_unicode(u, &mut |v| b.balayage(v)))
            .collect()),
    }
}

/// Le lot d'une combinaison : par sa touche virtuelle ; `barre` et `interrogation`, qui n'en ont
/// pas, par la disposition du clavier. Un tel caractère nommé s'écrit seul : la disposition choisit
/// elle-même les modificateurs qui le produisent, et Ctrl, Maj ou Alt demandés en plus en feraient
/// un autre caractère ou un raccourci. Accompagné de l'un d'eux, il rend l'arrêt d'`analyser_touche`,
/// qui le refuse déjà : 64, « touche inconnue : <combinaison> ».
fn lot_de_combinaison(c: &Combinaison, b: &mut dyn Bureau) -> Result<Vec<Evenement>, Arret> {
    match (vk(c.touche), caractere(c.touche)) {
        (Some(v), _) => Ok(lot_combinaison(c, v, &mut |x| b.balayage(x))),
        (None, Some(car)) if c.ctrl || c.maj || c.alt => Err(Arret::Usage(format!(
            "touche inconnue : {}",
            nom_du_caractere_nomme(c, car)
        ))),
        (None, Some(car)) => lot_clavier(car, b),
        // Aucune touche nommée n'est sans l'une et l'autre : ce bras ferme le `match`.
        (None, None) => Err(echec("touche sans touche virtuelle ni caractère")),
    }
}

/// La combinaison `c`, dont la touche est le caractère nommé `car`, écrite comme sur la ligne de
/// commande : ses modificateurs dans l'ordre Ctrl, Maj, Alt, puis `barre` pour `/`, `interrogation`
/// pour `?`.
fn nom_du_caractere_nomme(c: &Combinaison, car: char) -> String {
    let mut morceaux: Vec<&str> = [(c.ctrl, "Ctrl"), (c.maj, "Maj"), (c.alt, "Alt")]
        .into_iter()
        .filter(|&(demande, _)| demande)
        .map(|(_, nom)| nom)
        .collect();
    morceaux.push(if car == '/' { "barre" } else { "interrogation" });
    morceaux.join("+")
}

/// Le caractère qu'une unité frappe par la disposition : `\n` pour Entrée, `\t` pour Tab.
fn caractere_de(u: Unite) -> char {
    match u {
        Unite::Entree => '\n',
        Unite::Tab => '\t',
        Unite::Caractere(c) => c,
    }
}

/// La ligne du journal de la commande, si le journal s'ouvre ; vrai si elle y est parvenue. Un
/// journal exigé et inaccessible ne se rouvre pas ; un journal que l'analyse n'a pas exigé s'ouvre
/// ici.
fn journaliser(sous: &'static str, code: Code, s: &Suivi, b: &mut dyn Bureau) -> bool {
    let ouvert = match s.journal {
        Journal::Inaccessible => false,
        Journal::Ouvert => true,
        Journal::PasOuvert => b.ouvrir_journal().is_ok(),
    };
    if !ouvert {
        return false;
    }
    let session = b.session();
    let appel = Appel {
        date: b.date_locale(),
        session: session.as_deref(),
        sous_commande: sous,
        processus: s.processus.as_deref(),
        code,
        longueur: s.longueur,
    };
    b.ajouter_au_journal(&ligne(&appel)).is_ok()
}

/// L'échec du système : « échec du système : <quoi> ».
fn echec(quoi: &str) -> Arret {
    Arret::Echec(format!("échec du système : {quoi}"))
}

/// L'échec d'une lecture ou d'une action du système, son texte échappé : il tient en une ligne.
fn echec_systeme(e: ErreurSysteme) -> Arret {
    echec(&echapper(&e.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::bureau::WTS_DEVERROUILLE;
    use crate::domaine::bureau::WTS_INCONNU;
    use crate::domaine::cli::VERSION;
    use crate::domaine::cli::essai::a;
    use crate::domaine::faux::Accord;
    use crate::domaine::faux::AppelFaux;
    use crate::domaine::faux::DEBUT;
    use crate::domaine::faux::FauxBureau;
    use crate::domaine::faux::Reduction;
    use crate::domaine::faux::TOUJOURS;
    use crate::domaine::fenetre::essai::f;
    use crate::domaine::fenetre::essai::p;
    use crate::domaine::fenetre::essai::pp;
    use crate::domaine::touches::NomTouche;
    use crate::domaine::touches::essai::touches;
    use std::collections::VecDeque;
    use std::path::PathBuf;

    /// Le début de chaque ligne du journal du faux bureau : sa date, puis sa session.
    const TETE: &str = "2026-11-22T10:20:30\tsession-essai\t";

    /// `executer` sur les arguments de `s`, `pilote` en tête.
    fn x(b: &mut FauxBureau, s: &str) -> Issue {
        executer(&a(s), b)
    }

    /// Vrai si le faux bureau n'a reçu ni `Restaurer`, ni `Demande`, ni `Envoi`.
    fn aucun_geste(b: &FauxBureau) -> bool {
        gestes(b).is_empty()
    }

    /// Les `Restaurer`, `Demande` et `Envoi` reçus, dans l'ordre.
    fn gestes(b: &FauxBureau) -> Vec<AppelFaux> {
        b.appels
            .iter()
            .filter(|a| {
                matches!(
                    a,
                    AppelFaux::Restaurer(_) | AppelFaux::Demande(_) | AppelFaux::Envoi(_)
                )
            })
            .cloned()
            .collect()
    }

    /// Vrai si le faux bureau a reçu un appel que `quoi` reconnaît.
    fn appele(b: &FauxBureau, quoi: fn(&AppelFaux) -> bool) -> bool {
        b.appels.iter().any(quoi)
    }

    /// Le nombre d'appels égaux à `appel`.
    fn compte(b: &FauxBureau, appel: &AppelFaux) -> usize {
        b.appels.iter().filter(|a| *a == appel).count()
    }

    /// Le temps simulé écoulé depuis le début.
    fn ecoule(b: &FauxBureau) -> u64 {
        b.temps - DEBUT
    }

    /// Vrai si un lot envoyé porte un événement de touche.
    fn une_touche_envoyee(b: &FauxBureau) -> bool {
        b.envois()
            .iter()
            .flatten()
            .any(|e| matches!(e, Evenement::Touche { .. }))
    }

    /// L'événement d'une touche non étendue.
    fn t(vk: u8, scan: u8, relachee: bool) -> Evenement {
        Evenement::Touche {
            vk,
            scan,
            etendue: false,
            relachee,
        }
    }

    /// Le lot de l'activation : Alt gauche enfoncée, 0xE8 enfoncée puis relâchée, Alt relâchée.
    fn alt_entourant() -> Vec<Evenement> {
        vec![
            t(0xA4, 0x38, false),
            t(0xE8, 0, false),
            t(0xE8, 0, true),
            t(0xA4, 0x38, true),
        ]
    }

    /// Les appels reçus juste après l'envoi d'Alt entourant.
    fn apres_alt(b: &FauxBureau) -> Vec<AppelFaux> {
        let alt = AppelFaux::Envoi(alt_entourant());
        b.appels
            .iter()
            .skip_while(|a| **a != alt)
            .skip(1)
            .cloned()
            .collect()
    }

    /// Le refus de KeePassXC.
    fn refus_keepassxc() -> Issue {
        Issue::arret(Arret::Refus(
            "KeePassXC : touche et taper sont refusés dans cette version".into(),
        ))
    }

    /// Le poste avec explorer (50), qui a lancé KeePassXC (60), dont la fenêtre 7 est titrée.
    fn avec_keepassxc_par_explorer() -> FauxBureau {
        let mut b = FauxBureau::avec_appli();
        b.processus.push(p(50, 4, "explorer.exe", Some(1)));
        b.processus.push(p(60, 50, "KeePassXC.exe", Some(2)));
        b.fenetres.push(f(7, 60, "Base.kdbx - KeePassXC"));
        b
    }

    /// Une session verrouillée par son bureau d'entrée.
    fn session_verrouillee() -> EtatSession {
        EtatSession {
            bureau_d_entree: Some("Winlogon".into()),
            drapeaux: Some(WTS_DEVERROUILLE),
            etat: Some(WTS_ACTIF),
        }
    }

    // L'ordre des gardes.

    #[test]
    fn lecture_n_active_rien() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(x(&mut b, "fenetres").code, Code::Fait);
        assert!(aucun_geste(&b));
        assert!(
            !b.appels
                .iter()
                .any(|a| matches!(a, AppelFaux::PrendreVerrou(_)))
        );
    }
    #[test]
    fn journal_inaccessible_refuse_avant_tout_geste() {
        let mut b = FauxBureau::avec_appli();
        b.journal_ferme = true;
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(i.code, Code::Refus);
        assert_eq!(
            i.erreur.as_deref(),
            Some("journal inaccessible : ouvert dans une autre application ?")
        );
        assert!(
            !b.appels
                .iter()
                .any(|a| matches!(a, AppelFaux::Fenetres | AppelFaux::Processus))
        );
        assert!(aucun_geste(&b));
        assert_eq!(compte(&b, &AppelFaux::OuvrirJournal), 1);
        assert!(!appele(&b, |a| matches!(a, AppelFaux::Ajout(_))));
    }
    #[test]
    fn touche_interdite_avant_le_journal() {
        let mut b = FauxBureau::avec_appli();
        b.journal_ferme = true;
        let i = x(&mut b, "touche --fenetre appli-essai Alt+Tab");
        assert_eq!(i.code, Code::Refus);
        assert_eq!(i.erreur.as_deref(), Some("touche interdite : Alt+Tab"));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn touche_interdite_a_sa_ligne() {
        let mut b = FauxBureau::avec_appli();
        let i = x(&mut b, "touche --fenetre appli-essai Alt+Tab");
        assert_eq!(i.code, Code::Refus);
        assert!(!appele(&b, |a| matches!(a, AppelFaux::Fenetres)));
        assert_eq!(b.journal, vec![format!("{TETE}touche\t\t1\t\n")]);
    }
    #[test]
    fn usage_avant_la_touche_interdite() {
        let mut b = FauxBureau::avec_appli();
        let i = x(&mut b, "touche --fenetre appli-essai Alt+Tab --bidule");
        assert_eq!(i.code, Code::Usage);
        let erreur = i.erreur.unwrap_or_default();
        assert!(erreur.starts_with("option inconnue : --bidule"), "{erreur}");
        assert!(aucun_geste(&b));
    }
    #[test]
    fn texte_trop_long_avant_le_journal() {
        let mut b = FauxBureau::avec_appli();
        b.journal_ferme = true;
        let i = x(
            &mut b,
            &format!("taper --fenetre appli-essai --texte {}", "x".repeat(10_001)),
        );
        assert_eq!(
            i,
            Issue::arret(Arret::Usage(
                "texte trop long : 10001 caractères, 10000 au plus".into()
            ))
        );
        assert!(!appele(&b, |a| matches!(
            a,
            AppelFaux::Fenetres | AppelFaux::PrendreVerrou(_)
        )));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn taper_texte_de_10001_caracteres_sort_en_64() {
        let mut b = FauxBureau::avec_appli();
        let i = x(
            &mut b,
            &format!("taper --fenetre appli-essai --texte {}", "x".repeat(10_001)),
        );
        assert_eq!(
            i,
            Issue::arret(Arret::Usage(
                "texte trop long : 10001 caractères, 10000 au plus".into()
            ))
        );
        assert!(!appele(&b, |a| matches!(
            a,
            AppelFaux::Fenetres | AppelFaux::PrendreVerrou(_)
        )));
        assert!(aucun_geste(&b));
        assert_eq!(b.journal, vec![format!("{TETE}taper\t\t64\t\n")]);
    }
    #[test]
    fn taper_un_fichier_illisible_sort_en_64() {
        let mut b = FauxBureau::avec_appli();
        let i = x(&mut b, "taper --fenetre appli-essai --fichier f.txt");
        assert_eq!(
            i,
            Issue::arret(Arret::Usage("fichier illisible : f.txt".into()))
        );
        assert!(appele(&b, |a| *a == AppelFaux::LireFichier(40_007)));
        assert!(!appele(&b, |a| matches!(a, AppelFaux::Fenetres)));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn taper_une_entree_illisible_sort_en_64() {
        let mut b = FauxBureau::avec_appli();
        b.entree = None;
        let i = x(&mut b, "taper --fenetre appli-essai -");
        assert_eq!(
            i,
            Issue::arret(Arret::Usage("entrée standard illisible".into()))
        );
        assert!(appele(&b, |a| *a == AppelFaux::LireEntree(40_007)));
        assert!(!appele(&b, |a| matches!(a, AppelFaux::Fenetres)));
        assert!(aucun_geste(&b));
    }
    /// `taper --fenetre appli-essai`, puis `source`, des arguments passés tels quels : un texte peut
    /// porter une espace, un caractère de contrôle, ou être vide.
    fn taper_avec(source: &[&str]) -> Vec<OsString> {
        ["pilote", "taper", "--fenetre", "appli-essai"]
            .iter()
            .chain(source)
            .map(OsString::from)
            .collect()
    }
    /// Vrai si l'analyse a refusé avant le journal et toute lecture de fenêtre : la seule ligne du
    /// journal est celle de `taper` au code 64, et rien n'est lu ni fait du poste.
    fn refuse_a_l_analyse(b: &FauxBureau) -> bool {
        aucun_geste(b)
            && !appele(b, |a| {
                matches!(a, AppelFaux::Fenetres | AppelFaux::PrendreVerrou(_))
            })
            && b.journal == vec![format!("{TETE}taper\t\t64\t\n")]
    }
    // Un caractère de contrôle autre que la tabulation, CR et LF refuse en 64 à l'analyse, quelle
    // que soit la source du texte : `--texte`, un fichier, l'entrée standard.
    #[test]
    fn caractere_de_controle_refuse_pour_chaque_source() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            executer(&taper_avec(&["--texte", "a\u{1b}b"]), &mut b),
            Issue::arret(Arret::Usage("caractère de contrôle : U+001B".into()))
        );
        assert!(refuse_a_l_analyse(&b));
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(PathBuf::from("c.txt"), b"a\x08b\r\n".to_vec())];
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --fichier c.txt"),
            Issue::arret(Arret::Usage("caractère de contrôle : U+0008".into()))
        );
        assert!(refuse_a_l_analyse(&b));
        let mut b = FauxBureau::avec_appli();
        b.entree = Some("a\u{85}b".as_bytes().to_vec());
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai -"),
            Issue::arret(Arret::Usage("caractère de contrôle : U+0085".into()))
        );
        assert!(refuse_a_l_analyse(&b));
        // Avant le journal : un journal fermé ne change ni le code ni le message.
        let mut b = FauxBureau::avec_appli();
        b.journal_ferme = true;
        b.entree = Some(b"\x7f".to_vec());
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai -"),
            Issue::arret(Arret::Usage("caractère de contrôle : U+007F".into()))
        );
        assert!(aucun_geste(&b));
    }
    // Un fichier ASCII en UTF-16LE sans BOM, ses NUL compris, est illisible.
    #[test]
    fn utf16_sans_bom_refuse_comme_illisible() {
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(
            PathBuf::from("u.txt"),
            vec![b'a', 0, b'b', 0, b'\r', 0, b'\n', 0],
        )];
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --fichier u.txt"),
            Issue::arret(Arret::Usage(
                "texte illisible : ni UTF-8 ni UTF-16 à BOM".into()
            ))
        );
        assert!(refuse_a_l_analyse(&b));
    }
    // Un texte vide refuse en 64 à l'analyse, pour chaque source : `--texte ""`, un fichier qui ne
    // porte qu'un saut de ligne, une entrée standard vide. Rien n'est activé.
    #[test]
    fn texte_vide_refuse_pour_chaque_source() {
        let vide = Issue::arret(Arret::Usage("texte vide".into()));
        let mut b = FauxBureau::avec_appli();
        assert_eq!(executer(&taper_avec(&["--texte", ""]), &mut b), vide);
        assert!(refuse_a_l_analyse(&b));
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(PathBuf::from("v.txt"), b"\r\n".to_vec())];
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --fichier v.txt"),
            vide
        );
        assert!(refuse_a_l_analyse(&b));
        let mut b = FauxBureau::avec_appli();
        b.entree = Some(Vec::new());
        assert_eq!(x(&mut b, "taper --fenetre appli-essai -"), vide);
        assert!(refuse_a_l_analyse(&b));
    }
    // Un fichier ou une entrée standard se lisent au plus 40 007 octets : au-delà de 40 006, le texte
    // dépasse forcément 10 000 caractères, et refuse en 64 à l'analyse. À 40 006 octets, il passe la
    // borne des octets et se juge en caractères.
    #[test]
    fn lecture_bornee_en_octets_refuse_en_64() {
        let trop = Issue::arret(Arret::Usage(
            "texte trop long : plus de 40006 octets, 10000 caractères au plus".into(),
        ));
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(PathBuf::from("g.txt"), vec![b'x'; 50_000])];
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --fichier g.txt"),
            trop
        );
        assert!(appele(&b, |a| *a == AppelFaux::LireFichier(40_007)));
        assert!(refuse_a_l_analyse(&b));
        let mut b = FauxBureau::avec_appli();
        b.entree = Some(vec![b'y'; 40_007]);
        assert_eq!(x(&mut b, "taper --fenetre appli-essai -"), trop);
        assert!(appele(&b, |a| *a == AppelFaux::LireEntree(40_007)));
        assert!(refuse_a_l_analyse(&b));
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(PathBuf::from("b.txt"), vec![b'x'; 40_006])];
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --fichier b.txt"),
            Issue::arret(Arret::Usage(
                "texte trop long : 40006 caractères, 10000 au plus".into()
            ))
        );
        assert!(refuse_a_l_analyse(&b));
    }
    // Les touches interdites se jugent sur tous les arguments avant qu'aucun nom ne s'analyse : une
    // touche inconnue placée avant une interdite ne change ni le code ni le message.
    #[test]
    fn interdite_avant_l_inconnue_d_un_autre_argument() {
        for s in [
            "touche --fenetre appli-essai Bidule Alt+Tab",
            "touche --fenetre appli-essai Tab Bidule Win+R",
            "touche --fenetre appli-essai F13 Alt+F6",
        ] {
            let mut b = FauxBureau::avec_appli();
            let interdite = s.rsplit(' ').next().unwrap_or_default();
            assert_eq!(
                x(&mut b, s),
                Issue::arret(Arret::Refus(format!("touche interdite : {interdite}"))),
                "{s}"
            );
            assert!(aucun_geste(&b), "{s}");
        }
    }
    #[test]
    fn fenetre_refusee_avant_le_verrou() {
        let mut b = FauxBureau::avec_appli();
        b.processus.push(p(7, 4, "SystemSettings.exe", Some(1)));
        b.fenetres.push(f(2, 7, "Paramètres"));
        let i = x(&mut b, "touche --fenetre SystemSettings Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus(
                "fenêtre refusée : processus SystemSettings".into()
            ))
        );
        assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_))));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn keepassxc_par_explorer_refuse_taper() {
        let mut b = avec_keepassxc_par_explorer();
        let i = x(&mut b, "taper --fenetre explorer --texte a");
        assert_eq!(i, refus_keepassxc());
        assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_))));
        assert!(aucun_geste(&b));
        // Le journal nomme le processus de la fenêtre retenue, P, non le nom demandé.
        assert_eq!(b.journal, vec![format!("{TETE}taper\tKeePassXC\t1\t1\n")]);
    }
    #[test]
    fn keepassxc_par_explorer_refuse_touche() {
        let mut b = avec_keepassxc_par_explorer();
        let i = x(&mut b, "touche --fenetre explorer Tab");
        assert_eq!(i, refus_keepassxc());
        assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_))));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn keepassxc_enfant_de_w_refuse_taper() {
        let mut b = FauxBureau::avec_appli();
        b.processus.push(p(300, 4, "KeePassXC.exe", Some(1)));
        b.fenetres[0].pids_enfants = vec![300];
        let i = x(&mut b, "taper --fenetre appli-essai --texte a");
        assert_eq!(i, refus_keepassxc());
        assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_))));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn activer_vers_keepassxc_est_permis() {
        let mut b = avec_keepassxc_par_explorer();
        let i = x(&mut b, "activer --fenetre KeePassXC");
        assert_eq!(
            i,
            Issue::fait(vec![
                "[Fenetre] <Base.kdbx - KeePassXC> processus=KeePassXC pid=60 premier-plan=oui"
                    .into()
            ])
        );
    }
    #[test]
    fn verrou_demande_5_s() {
        let mut b = FauxBureau::avec_appli();
        x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(compte(&b, &AppelFaux::PrendreVerrou(Duree(5000))), 1);
    }
    #[test]
    fn activer_prend_le_verrou() {
        let mut b = FauxBureau::avec_appli();
        x(&mut b, "activer --fenetre appli-essai");
        assert_eq!(compte(&b, &AppelFaux::PrendreVerrou(Duree(5000))), 1);
    }
    #[test]
    fn verrou_occupe_refuse_en_1_sans_rien_envoyer() {
        let mut b = FauxBureau::avec_appli();
        b.verrou = Ok(EtatVerrou::Occupe);
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus(
                "un autre geste est en cours depuis 5 s".into()
            ))
        );
        assert!(!appele(&b, |a| matches!(a, AppelFaux::EtatSession)));
        assert!(aucun_geste(&b));
    }
    #[test]
    fn verrou_abandonne_compte_comme_pris() {
        let mut b = FauxBureau::avec_appli();
        b.verrou = Ok(EtatVerrou::Repris);
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab").code,
            Code::Fait
        );
        assert_eq!(b.envois().len(), 1);
    }
    #[test]
    fn poste_verrouille_par_cas() {
        for (bureau, drapeaux, etat, verrouille) in [
            (
                Some("Default"),
                Some(WTS_DEVERROUILLE),
                Some(WTS_ACTIF),
                false,
            ),
            (
                Some("Winlogon"),
                Some(WTS_DEVERROUILLE),
                Some(WTS_ACTIF),
                true,
            ),
            (None, Some(WTS_DEVERROUILLE), Some(WTS_ACTIF), true),
            (Some("Default"), Some(WTS_VERROUILLE), Some(WTS_ACTIF), true),
            (Some("Default"), Some(WTS_INCONNU), Some(WTS_ACTIF), false),
            (Some("Default"), None, None, false),
            (None, None, None, true),
            (Some("Default"), Some(WTS_DEVERROUILLE), Some(4), true),
            (Some("Default"), Some(WTS_INCONNU), Some(4), true),
        ] {
            let mut b = FauxBureau::avec_appli();
            b.session = EtatSession {
                bureau_d_entree: bureau.map(Into::into),
                drapeaux,
                etat,
            };
            let i = x(&mut b, "touche --fenetre appli-essai Tab");
            let cas = format!("{bureau:?} {drapeaux:?} {etat:?}");
            if verrouille {
                assert_eq!(i.erreur.as_deref(), Some("poste verrouillé"), "{cas}");
                assert!(aucun_geste(&b), "{cas}");
            } else {
                assert_eq!(i.code, Code::Fait, "{cas}");
            }
        }
    }
    #[test]
    fn niveau_plus_haut_refuse_en_1() {
        let mut b = FauxBureau::avec_appli();
        b.niveau_cible = Some(0x3000);
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus("fenêtre plus élevée que pilote".into()))
        );
        assert!(aucun_geste(&b));
        // Le niveau lu est celui de P, le processus de la fenêtre retenue, puis celui de pilote.
        assert_eq!(compte(&b, &AppelFaux::Niveau(Some(100))), 1);
        assert_eq!(compte(&b, &AppelFaux::Niveau(None)), 1);
    }
    #[test]
    fn niveau_egal_ou_plus_bas_passe() {
        for cible in [0x2000, 0x1000] {
            let mut b = FauxBureau::avec_appli();
            b.niveau_cible = Some(cible);
            assert_eq!(
                x(&mut b, "touche --fenetre appli-essai Tab").code,
                Code::Fait,
                "{cible:#X}"
            );
        }
    }
    #[test]
    fn niveau_illisible_refuse_en_1() {
        let illisible = Issue::arret(Arret::Refus("niveau d'intégrité illisible".into()));
        let mut b = FauxBureau::avec_appli();
        b.niveau_cible = None;
        assert_eq!(x(&mut b, "touche --fenetre appli-essai Tab"), illisible);
        assert!(aucun_geste(&b));
        let mut b = FauxBureau::avec_appli();
        b.niveau_pilote = None;
        assert_eq!(x(&mut b, "touche --fenetre appli-essai Tab"), illisible);
        assert!(aucun_geste(&b));
    }
    #[test]
    fn refuse_si_un_modificateur_est_enfonce() {
        let mut b = FauxBureau::avec_appli();
        b.touches_avant = vec![0xA0];
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus("touche de modification enfoncée".into()))
        );
        assert!(aucun_geste(&b));
    }
    #[test]
    fn modificateurs_revus_apres_l_activation() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = None;
        b.touches_apres = vec![vk::MAJ_G];
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus("touche de modification enfoncée".into()))
        );
        assert!(appele(&b, |a| matches!(a, AppelFaux::Demande(_))));
        assert!(!une_touche_envoyee(&b));
    }
    #[test]
    fn verrou_avant_le_poste_verrouille() {
        let mut b = FauxBureau::avec_appli();
        b.verrou = Ok(EtatVerrou::Occupe);
        b.session = session_verrouillee();
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab")
                .erreur
                .as_deref(),
            Some("un autre geste est en cours depuis 5 s")
        );
    }
    #[test]
    fn poste_verrouille_avant_le_niveau() {
        let mut b = FauxBureau::avec_appli();
        b.session = session_verrouillee();
        b.niveau_cible = Some(0x3000);
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab")
                .erreur
                .as_deref(),
            Some("poste verrouillé")
        );
    }
    #[test]
    fn niveau_avant_les_modificateurs() {
        let mut b = FauxBureau::avec_appli();
        b.niveau_cible = Some(0x3000);
        b.touches_avant = vec![vk::MAJ_G];
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab")
                .erreur
                .as_deref(),
            Some("fenêtre plus élevée que pilote")
        );
    }
    #[test]
    fn modificateurs_avant_l_activation() {
        let mut b = FauxBureau::avec_appli();
        b.touches_avant = vec![vk::MAJ_G];
        b.reduction = Reduction::JusquARestaurer;
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab")
                .erreur
                .as_deref(),
            Some("touche de modification enfoncée")
        );
        assert!(!appele(&b, |a| matches!(a, AppelFaux::Restaurer(_))));
    }
    // Verr. Maj se lit après les touches enfoncées.
    #[test]
    fn modificateurs_avant_verr_maj() {
        let mut b = FauxBureau::avec_appli();
        b.touches_avant = vec![vk::MAJ_G];
        b.verr_maj = true;
        assert_eq!(
            x(
                &mut b,
                "taper --fenetre appli-essai --comme-au-clavier --texte a"
            )
            .erreur
            .as_deref(),
            Some("touche de modification enfoncée")
        );
    }
    // Chaque garde à sa place, et rien d'autre : l'ordre entier des appels d'un `taper
    // --comme-au-clavier` qui passe, d'un seul caractère.
    #[test]
    fn ordre_complet_des_appels() {
        let mut b = FauxBureau::avec_appli();
        let i = x(
            &mut b,
            "taper --fenetre appli-essai --comme-au-clavier --texte a",
        );
        assert_eq!(i, Issue::fait(vec![]));
        assert_eq!(
            b.appels,
            vec![
                AppelFaux::OuvrirJournal,
                AppelFaux::Fenetres,
                AppelFaux::Processus,
                AppelFaux::PrendreVerrou(Duree(5000)),
                AppelFaux::EtatSession,
                AppelFaux::Niveau(Some(100)),
                AppelFaux::Niveau(None),
                AppelFaux::TouchesEnfoncees,
                AppelFaux::VerrMaj,
                AppelFaux::EstReduite,
                AppelFaux::PremierPlan,
                AppelFaux::Processus,
                AppelFaux::PremierPlan,
                AppelFaux::Fenetres,
                AppelFaux::Processus,
                AppelFaux::TouchesEnfoncees,
                AppelFaux::PremierPlan,
                AppelFaux::Processus,
                AppelFaux::PremierPlan,
                AppelFaux::Envoi(vec![t(0x41, 0x10, false), t(0x41, 0x10, true)]),
                AppelFaux::Attendre(Duree(5)),
                AppelFaux::Ajout(format!("{TETE}taper\tappli-essai\t0\t1\n")),
            ]
        );
    }
    // `fenetres` lit les fenêtres, puis l'instantané des processus, puis le premier plan, et rien
    // d'autre ; ses lignes sont celles d'ADR-0006, « Les sorties de la tranche 1 ».
    #[test]
    fn fenetres_ecrit_ses_lignes_sans_les_gardes() {
        let mut b = FauxBureau::avec_appli();
        b.processus.push(p(50, 4, "explorer.exe", Some(1)));
        b.fenetres.push(f(2, 50, "Documents"));
        assert_eq!(
            x(&mut b, "fenetres"),
            Issue::fait(vec![
                "[Fenetre] <Appli d'essai> processus=appli-essai pid=100 premier-plan=oui".into(),
                "[Fenetre] <Documents> processus=explorer pid=50".into(),
            ])
        );
        assert_eq!(
            b.appels,
            vec![
                AppelFaux::OuvrirJournal,
                AppelFaux::Fenetres,
                AppelFaux::Processus,
                AppelFaux::PremierPlan,
                AppelFaux::Ajout(format!("{TETE}fenetres\t\t0\t\n")),
            ]
        );
    }
    // Aucune fenêtre : 2, et le journal ne nomme aucun processus, faute de fenêtre retenue.
    #[test]
    fn aucune_fenetre_sort_en_2_sans_processus_au_journal() {
        let mut b = FauxBureau::avec_appli();
        let i = x(&mut b, "touche --fenetre absent Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Introuvable("aucune fenêtre de « absent »".into()))
        );
        assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_))));
        assert!(aucun_geste(&b));
        assert_eq!(b.journal, vec![format!("{TETE}touche\t\t2\t\n")]);
    }
    #[test]
    fn constantes_par_leur_valeur() {
        assert_eq!(ATTENTE_DU_VERROU, Duree(5_000));
        assert_eq!(CADENCE, Duree(5));
        assert_eq!(
            [WTS_VERROUILLE, WTS_DEVERROUILLE, WTS_INCONNU, WTS_ACTIF],
            [0, 1, -1, 0]
        );
    }

    // L'activation et l'envoi.

    #[test]
    fn activer_ne_fait_rien_si_le_premier_plan_est_accepte() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
        assert!(!appele(&b, |a| matches!(
            a,
            AppelFaux::Demande(_) | AppelFaux::Envoi(_)
        )));
    }
    #[test]
    fn activer_quand_une_autre_fenetre_de_p_a_le_premier_plan() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = Some(pp(2, 100, vec![]));
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1))
            ]
        );
    }
    #[test]
    fn activer_essaie_le_mouvement_nul_puis_alt_entourant() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = Some(pp(2, 100, vec![]));
        b.accord = Accord::ApresAlt;
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1)),
                AppelFaux::Envoi(alt_entourant()),
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
        // La première demande se sonde toutes les 100 ms, 1 s au plus, avant Alt.
        assert_eq!(compte(&b, &AppelFaux::Attendre(Duree(100))), 10);
        assert_eq!(ecoule(&b), 1_000);
    }
    #[test]
    fn activer_attend_que_les_modificateurs_soient_relaches() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = Some(pp(2, 100, vec![]));
        b.accord = Accord::ApresAlt;
        b.alt_tenue = Duree(30);
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
        assert_eq!(
            apres_alt(&b)[..8],
            [
                AppelFaux::TouchesEnfoncees,
                AppelFaux::Attendre(Duree(10)),
                AppelFaux::TouchesEnfoncees,
                AppelFaux::Attendre(Duree(10)),
                AppelFaux::TouchesEnfoncees,
                AppelFaux::Attendre(Duree(10)),
                AppelFaux::TouchesEnfoncees,
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = Some(pp(2, 100, vec![]));
        b.accord = Accord::ApresAlt;
        b.alt_tenue = TOUJOURS;
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai"),
            Issue::arret(Arret::Refus("touche de modification enfoncée".into()))
        );
        let apres = apres_alt(&b);
        assert!(!apres.iter().any(|a| matches!(a, AppelFaux::Demande(_))));
        // La sonde des modificateurs : toutes les 10 ms, 500 ms au plus.
        assert_eq!(
            apres
                .iter()
                .filter(|a| **a == AppelFaux::Attendre(Duree(10)))
                .count(),
            50
        );
    }
    #[test]
    fn activer_ne_frappe_jamais_alt_seule() {
        let scenarios: [fn(&mut FauxBureau); 8] = [
            |_| {},
            |b| b.premier_plan = Some(pp(2, 100, vec![])),
            |b| {
                b.premier_plan = None;
                b.accord = Accord::ApresAlt;
            },
            |b| {
                b.premier_plan = None;
                b.accord = Accord::Jamais;
            },
            |b| {
                b.premier_plan = None;
                b.accord = Accord::ApresAlt;
                b.alt_tenue = Duree(30);
            },
            |b| {
                b.premier_plan = None;
                b.accord = Accord::ApresAlt;
                b.alt_tenue = TOUJOURS;
            },
            |b| {
                b.premier_plan = None;
                b.reduction = Reduction::JusquARestaurer;
            },
            |b| b.reduction = Reduction::Toujours,
        ];
        let mut alt_envoyee = false;
        for (n, regler) in scenarios.iter().enumerate() {
            let mut b = FauxBureau::avec_appli();
            regler(&mut b);
            x(&mut b, "activer --fenetre appli-essai");
            for lot in b.envois() {
                let vks: Vec<u8> = touches(&lot).iter().map(|&(v, _)| v).collect();
                assert!(
                    vks.is_empty() || !vks.iter().all(|v| vk::MODIFICATEURS.contains(v)),
                    "scénario {n} : {lot:?}"
                );
                alt_envoyee |= vks.contains(&vk::ALT_G);
            }
        }
        assert!(alt_envoyee);
    }
    #[test]
    fn activer_restaure_une_fenetre_reduite() {
        let mut b = FauxBureau::avec_appli();
        b.reduction = Reduction::JusquARestaurer;
        b.premier_plan = None;
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Restaurer(IdFenetre(1)),
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
    }
    #[test]
    fn activer_attend_une_restauration_tardive() {
        // La restauration part sans attendre la fenêtre visée : celle-ci reste réduite un temps
        // après `restaurer`, et la sonde l'attend au lieu de refuser.
        let mut b = FauxBureau::avec_appli();
        b.reduction = Reduction::Tardive(Duree(250));
        b.premier_plan = None;
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Restaurer(IdFenetre(1)),
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
        // Réduite à 0, 100 et 200 ms après la restauration, restaurée à 300 ms.
        assert_eq!(compte(&b, &AppelFaux::Attendre(Duree(100))), 3);
    }
    #[test]
    fn activer_refuse_une_fenetre_qui_reste_reduite() {
        let mut b = FauxBureau::avec_appli();
        b.reduction = Reduction::Toujours;
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai"),
            Issue::arret(Arret::Refus("Windows a refusé le premier plan".into()))
        );
        assert_eq!(gestes(&b), vec![AppelFaux::Restaurer(IdFenetre(1))]);
        // La restauration se sonde toutes les 100 ms, 2 s au plus.
        assert_eq!(compte(&b, &AppelFaux::Attendre(Duree(100))), 20);
        assert_eq!(ecoule(&b), 2_000);
    }
    #[test]
    fn activer_refuse_quand_windows_refuse() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = None;
        b.accord = Accord::Jamais;
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai"),
            Issue::arret(Arret::Refus("Windows a refusé le premier plan".into()))
        );
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1)),
                AppelFaux::Envoi(alt_entourant()),
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
        assert_eq!(compte(&b, &AppelFaux::Attendre(Duree(100))), 20);
        assert_eq!(ecoule(&b), 2_000);
    }
    #[test]
    fn refus_rejuge_apres_l_activation() {
        let mut b = FauxBureau::avec_appli();
        b.enfant_tardif = Some(p(7, 4, "SystemSettings.exe", Some(1)));
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus(
                "fenêtre refusée : processus SystemSettings".into()
            ))
        );
        assert_eq!(compte(&b, &AppelFaux::Fenetres), 2);
        assert!(!une_touche_envoyee(&b));
    }
    #[test]
    fn keepassxc_rejuge_apres_l_activation() {
        let mut b = FauxBureau::avec_appli();
        b.reduction = Reduction::JusquARestaurer;
        b.enfant_tardif = Some(p(300, 4, "KeePassXC.exe", Some(1)));
        let i = x(&mut b, "touche --fenetre appli-essai Tab");
        assert_eq!(i, refus_keepassxc());
        assert!(appele(&b, |a| matches!(a, AppelFaux::Restaurer(_))));
        assert!(!une_touche_envoyee(&b));
    }
    #[test]
    fn refus_rejuge_avant_les_modificateurs_revus() {
        let mut b = FauxBureau::avec_appli();
        b.reduction = Reduction::JusquARestaurer;
        b.enfant_tardif = Some(p(7, 4, "SystemSettings.exe", Some(1)));
        b.touches_apres = vec![vk::MAJ_G];
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab")
                .erreur
                .as_deref(),
            Some("fenêtre refusée : processus SystemSettings")
        );
        assert!(!une_touche_envoyee(&b));
    }
    // `activer` rejuge la liste de refus, mais pas la règle de KeePassXC, qui ne vise que
    // `touche` et `taper`.
    #[test]
    fn activer_rejuge_le_refus_sans_la_regle_de_keepassxc() {
        let mut b = FauxBureau::avec_appli();
        b.enfant_tardif = Some(p(7, 4, "SystemSettings.exe", Some(1)));
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai").erreur.as_deref(),
            Some("fenêtre refusée : processus SystemSettings")
        );
        let mut b = FauxBureau::avec_appli();
        b.enfant_tardif = Some(p(300, 4, "KeePassXC.exe", Some(1)));
        assert_eq!(x(&mut b, "activer --fenetre appli-essai").code, Code::Fait);
    }
    // W relue est la fenêtre de même HWND et de même processus : absente, ou son HWND repris par un
    // autre processus, la frappe ne part pas.
    #[test]
    fn w_disparue_apres_l_activation_sort_en_2() {
        for relue in [
            vec![],
            vec![f(1, 555, "Appli d'essai")],
            vec![f(3, 100, "Appli d'essai")],
        ] {
            let mut b = FauxBureau::avec_appli();
            b.relecture = Some(relue.clone());
            assert_eq!(
                x(&mut b, "touche --fenetre appli-essai Tab"),
                Issue::arret(Arret::Introuvable(
                    "aucune fenêtre de « appli-essai »".into()
                )),
                "{relue:?}"
            );
            assert!(!une_touche_envoyee(&b), "{relue:?}");
        }
    }
    #[test]
    fn activer_ecrit_la_ligne_de_la_fenetre() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai").sortie,
            vec!["[Fenetre] <Appli d'essai> processus=appli-essai pid=100 premier-plan=oui"]
        );
    }
    // Un dialogue possédé par W au premier plan est accepté sans demande ; W n'a pas le premier
    // plan, et sa ligne ne le dit pas.
    #[test]
    fn activer_vers_un_dialogue_de_w_au_premier_plan() {
        let mut b = FauxBureau::avec_appli();
        b.processus.push(p(101, 100, "fils.exe", Some(6)));
        b.premier_plan = Some(pp(9, 101, vec![IdFenetre(1)]));
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai"),
            Issue::fait(vec![
                "[Fenetre] <Appli d'essai> processus=appli-essai pid=100".into()
            ])
        );
        assert!(aucun_geste(&b));
    }
    // Chaque unité part juste après la dernière lecture du premier plan, faite après l'instantané
    // des processus, lui-même lu après une première lecture du premier plan : rien d'autre ne
    // s'intercale entre le dernier contrôle et l'envoi.
    #[test]
    fn premier_plan_verifie_avant_chaque_unite() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --texte abc").code,
            Code::Fait
        );
        let envois: Vec<usize> = (0..b.appels.len())
            .filter(|&i| matches!(b.appels[i], AppelFaux::Envoi(_)))
            .collect();
        assert_eq!(envois.len(), 3);
        for &i in &envois {
            assert!(
                b.appels[..i].ends_with(&[
                    AppelFaux::PremierPlan,
                    AppelFaux::Processus,
                    AppelFaux::PremierPlan
                ]),
                "appel {i} : {:?}",
                &b.appels[..i]
            );
        }
    }
    #[test]
    fn la_frappe_attend_5_ms_entre_deux_unites() {
        let mut b = FauxBureau::avec_appli();
        x(&mut b, "taper --fenetre appli-essai --texte abc");
        let envois: Vec<usize> = (0..b.appels.len())
            .filter(|&i| matches!(b.appels[i], AppelFaux::Envoi(_)))
            .collect();
        assert_eq!(envois.len(), 3);
        for paire in envois.windows(2) {
            assert!(b.appels[paire[0]..paire[1]].contains(&AppelFaux::Attendre(Duree(5))));
        }
    }
    #[test]
    fn taper_s_arrete_quand_le_premier_plan_change() {
        let mut b = FauxBureau::avec_appli();
        b.vol = Some((3, pp(50, 555, vec![])));
        let i = x(&mut b, "taper --fenetre appli-essai --texte abcdefghij");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus(
                "le premier plan a changé : 3 unités sur 10 sont parties".into()
            ))
        );
        assert_eq!(b.envois().len(), 3);
    }
    // Une fenêtre qui prend le premier plan pendant la lecture de l'instantané des processus, après
    // le contrôle du premier plan, ne reçoit rien : le premier plan se relit après l'instantané, et
    // l'unité ne part que s'il n'a pas changé.
    #[test]
    fn vol_pendant_l_instantane_n_envoie_rien_au_voleur() {
        let mut b = FauxBureau::avec_appli();
        b.vol_pendant_l_instantane = Some((1, pp(50, 555, vec![])));
        let i = x(&mut b, "taper --fenetre appli-essai --texte abc");
        assert_eq!(
            i,
            Issue::arret(Arret::Refus(
                "le premier plan a changé : 1 unités sur 3 sont parties".into()
            ))
        );
        assert_eq!(b.envois().len(), 1);
    }
    #[test]
    fn une_combinaison_part_en_un_seul_envoi() {
        let mut b = FauxBureau::avec_appli();
        x(&mut b, "touche --fenetre appli-essai Ctrl+Maj+Tab");
        let envois = b.envois();
        assert_eq!(envois.len(), 1);
        assert_eq!(envois[0].len(), 6);
        assert_eq!(
            touches(&envois[0]),
            [
                (0xA2, false),
                (0xA0, false),
                (0x09, false),
                (0x09, true),
                (0xA0, true),
                (0xA2, true)
            ]
        );
    }
    #[test]
    fn touche_fois_repete_la_suite() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab Entree --fois 2").code,
            Code::Fait
        );
        let tab = vec![t(0x09, 0x0F, false), t(0x09, 0x0F, true)];
        let entree = vec![t(0x0D, 0x1C, false), t(0x0D, 0x1C, true)];
        assert_eq!(b.envois(), vec![tab.clone(), entree.clone(), tab, entree]);
    }
    #[test]
    fn envoi_incomplet_sort_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.insere_au_plus = Some(0);
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec(
                "échec du système : SendInput a inséré 0 événements sur 2".into()
            ))
        );
    }
    // Un envoi de l'activation que `SendInput` n'insère pas du tout (UIPI, quand une fenêtre plus
    // élevée a le premier plan) ne sort pas en 3 : l'activation suit son cours, et se juge au
    // premier plan relu, qui ne vient pas.
    #[test]
    fn activation_non_inseree_se_juge_au_premier_plan() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = None;
        b.insere_au_plus = Some(0);
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai"),
            Issue::arret(Arret::Refus("Windows a refusé le premier plan".into()))
        );
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1)),
                AppelFaux::Envoi(alt_entourant()),
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
    }
    // Un mouvement nul que `SendInput` n'insère pas, puis une demande que Windows accorde : la
    // commande réussit.
    #[test]
    fn mouvement_nul_non_insere_puis_premier_plan_accorde() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = None;
        b.accord = Accord::ATouteDemande;
        b.insere_au_plus = Some(0);
        assert_eq!(
            x(&mut b, "activer --fenetre appli-essai"),
            Issue::fait(vec![
                "[Fenetre] <Appli d'essai> processus=appli-essai pid=100 premier-plan=oui".into()
            ])
        );
        assert_eq!(
            gestes(&b),
            vec![
                AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                AppelFaux::Demande(IdFenetre(1)),
            ]
        );
    }
    // Une insertion partielle de l'Alt entourant reste un échec du système, après le relâchement
    // d'Alt gauche, que précède 0xE8 enfoncée puis relâchée : jamais Alt relâchée seule. Que le
    // début inséré ait aussi enfoncé 0xE8 ou non, le relâchement est le même : la frappe de 0xE8 qui
    // le commence relâche aussi 0xE8 restée enfoncée. Aucune demande ne suit.
    #[test]
    fn alt_entourant_insere_en_partie_relache_puis_sort_en_3() {
        for k in 1..=3 {
            let mut b = FauxBureau::avec_appli();
            b.premier_plan = None;
            b.accord = Accord::ApresAlt;
            b.insertions = VecDeque::from([1, k]);
            assert_eq!(
                x(&mut b, "activer --fenetre appli-essai"),
                Issue::arret(Arret::Echec(format!(
                    "échec du système : SendInput a inséré {k} événements sur 4 ; relâchement : 3 \
                     sur 3"
                ))),
                "{k}"
            );
            assert_eq!(
                gestes(&b),
                vec![
                    AppelFaux::Envoi(vec![Evenement::SourisNulle]),
                    AppelFaux::Demande(IdFenetre(1)),
                    AppelFaux::Envoi(alt_entourant()),
                    AppelFaux::Envoi(vec![
                        t(0xE8, 0, false),
                        t(0xE8, 0, true),
                        t(0xA4, 0x38, true)
                    ]),
                ],
                "{k}"
            );
        }
    }
    // Une combinaison insérée en partie : ce que son début a enfoncé, et pas relâché, se relâche
    // d'un seul envoi, dans l'ordre inverse de l'enfoncement, sans garde du premier plan, puis la
    // commande sort en 3 en disant les deux insertions.
    #[test]
    fn combinaison_inseree_en_partie_relache_dans_l_ordre_inverse() {
        let ctrl = |relachee| t(0xA2, 0x1D, relachee);
        let maj = |relachee| t(0xA0, 0x2A, relachee);
        let tab = |relachee| t(0x09, 0x0F, relachee);
        for (k, relachement) in [
            (1, vec![ctrl(true)]),
            (2, vec![maj(true), ctrl(true)]),
            (3, vec![tab(true), maj(true), ctrl(true)]),
            (4, vec![maj(true), ctrl(true)]),
            (5, vec![ctrl(true)]),
        ] {
            let mut b = FauxBureau::avec_appli();
            b.insertions = VecDeque::from([k]);
            let p = relachement.len();
            assert_eq!(
                x(&mut b, "touche --fenetre appli-essai Ctrl+Maj+Tab"),
                Issue::arret(Arret::Echec(format!(
                    "échec du système : SendInput a inséré {k} événements sur 6 ; relâchement : \
                     {p} sur {p}"
                ))),
                "{k}"
            );
            let combinaison = vec![
                ctrl(false),
                maj(false),
                tab(false),
                tab(true),
                maj(true),
                ctrl(true),
            ];
            assert_eq!(b.envois(), vec![combinaison, relachement], "{k}");
            // Le relâchement part juste après l'envoi partiel, sans lecture du premier plan.
            let apres: Vec<&AppelFaux> = b
                .appels
                .iter()
                .skip_while(|a| !matches!(a, AppelFaux::Envoi(_)))
                .collect();
            assert!(
                matches!(
                    apres[..],
                    [
                        AppelFaux::Envoi(_),
                        AppelFaux::Envoi(_),
                        AppelFaux::Ajout(_)
                    ]
                ),
                "{k} : {apres:?}"
            );
        }
    }
    // Un caractère d'AltGr tapé par la disposition, inséré en partie : 0xE8 enfoncée puis relâchée
    // précède l'Alt droite relâchée, qui garde son balayage étendu, puis Ctrl gauche.
    #[test]
    fn altgr_insere_en_partie_frappe_0xe8_avant_l_alt_droite() {
        let mut b = FauxBureau::avec_appli();
        b.insertions = VecDeque::from([2]);
        assert_eq!(
            x(
                &mut b,
                "taper --fenetre appli-essai --comme-au-clavier --texte @"
            ),
            Issue::arret(Arret::Echec(
                "échec du système : SendInput a inséré 2 événements sur 6 ; relâchement : 4 sur 4"
                    .into()
            ))
        );
        let alt_droite_relachee = Evenement::Touche {
            vk: 0xA5,
            scan: 0x38,
            etendue: true,
            relachee: true,
        };
        assert_eq!(
            b.envois()[1],
            vec![
                t(0xE8, 0, false),
                t(0xE8, 0, true),
                alt_droite_relachee,
                t(0xA2, 0x1D, true)
            ]
        );
    }
    // Une unité UTF-16 insérée en partie se relâche par son unité ; quand le début inséré n'a rien
    // laissé enfoncé, aucun relâchement ne part, et le message est celui de l'envoi seul.
    #[test]
    fn unicode_insere_en_partie_relache_son_unite() {
        let unicode = |unite, relachee| Evenement::Unicode { unite, relachee };
        let mut b = FauxBureau::avec_appli();
        b.insertions = VecDeque::from([3]);
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --texte 😀"),
            Issue::arret(Arret::Echec(
                "échec du système : SendInput a inséré 3 événements sur 4 ; relâchement : 1 sur 1"
                    .into()
            ))
        );
        assert_eq!(b.envois()[1], vec![unicode(0xDE00, true)]);
        let mut b = FauxBureau::avec_appli();
        b.insertions = VecDeque::from([2]);
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --texte 😀"),
            Issue::arret(Arret::Echec(
                "échec du système : SendInput a inséré 2 événements sur 4".into()
            ))
        );
        assert_eq!(b.envois().len(), 1);
    }
    // Le relâchement lui-même inséré en partie : la commande sort toujours en 3, et le message dit
    // combien de relâchements sont partis.
    #[test]
    fn relachement_insere_en_partie_sort_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.insertions = VecDeque::from([2, 1]);
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Ctrl+Maj+Tab"),
            Issue::arret(Arret::Echec(
                "échec du système : SendInput a inséré 2 événements sur 6 ; relâchement : 1 sur 2"
                    .into()
            ))
        );
        assert_eq!(b.envois().len(), 2);
    }
    // Pendant la frappe, un envoi que Windows n'insère pas du tout se juge au premier plan relu :
    // une fenêtre qui l'a pris fait sortir en 1, avec le compte des unités parties avant lui.
    #[test]
    fn zero_insere_pendant_la_frappe_quand_le_premier_plan_change_sort_en_1() {
        let mut b = FauxBureau::avec_appli();
        b.insertions = VecDeque::from([2, 2, 0]);
        b.vol = Some((3, pp(50, 555, vec![])));
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --texte abcde"),
            Issue::arret(Arret::Refus(
                "le premier plan a changé : 2 unités sur 5 sont parties".into()
            ))
        );
        assert_eq!(b.envois().len(), 3);
    }
    // Le premier plan relu encore accepté, un envoi que Windows n'insère pas du tout reste un échec
    // du système : rien n'explique le refus.
    #[test]
    fn zero_insere_pendant_la_frappe_premier_plan_accepte_sort_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.insertions = VecDeque::from([2, 0]);
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --texte abc"),
            Issue::arret(Arret::Echec(
                "échec du système : SendInput a inséré 0 événements sur 2".into()
            ))
        );
        assert_eq!(b.envois().len(), 2);
        assert!(
            b.appels.ends_with(&[
                AppelFaux::Envoi(vec![
                    Evenement::Unicode {
                        unite: 0x62,
                        relachee: false
                    },
                    Evenement::Unicode {
                        unite: 0x62,
                        relachee: true
                    }
                ]),
                AppelFaux::PremierPlan,
                AppelFaux::Processus,
                AppelFaux::PremierPlan,
                AppelFaux::Ajout(format!("{TETE}taper\tappli-essai\t3\t3\n")),
            ]),
            "{:?}",
            b.appels
        );
    }
    #[test]
    fn touche_barre_passe_par_la_disposition() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai barre").code,
            Code::Fait
        );
        assert_eq!(
            b.envois()
                .iter()
                .map(|lot| touches(lot))
                .collect::<Vec<_>>(),
            vec![vec![
                (0xA0, false),
                (0xBF, false),
                (0xBF, true),
                (0xA0, true)
            ]]
        );
    }
    // Un caractère nommé s'écrit seul : accompagné de Ctrl, Maj ou Alt, son lot n'est pas formé, et
    // l'arrêt est celui d'`analyser_touche`, qui le refuse avant.
    #[test]
    fn caractere_nomme_avec_un_modificateur_est_inconnu() {
        let seul = Combinaison {
            ctrl: false,
            maj: false,
            alt: false,
            touche: NomTouche::Barre,
        };
        for (c, nom) in [
            (Combinaison { ctrl: true, ..seul }, "Ctrl+barre"),
            (
                Combinaison {
                    maj: true,
                    touche: NomTouche::Interrogation,
                    ..seul
                },
                "Maj+interrogation",
            ),
            (Combinaison { alt: true, ..seul }, "Alt+barre"),
            (
                Combinaison {
                    ctrl: true,
                    maj: true,
                    alt: true,
                    touche: NomTouche::Interrogation,
                },
                "Ctrl+Maj+Alt+interrogation",
            ),
        ] {
            let mut b = FauxBureau::avec_appli();
            assert_eq!(
                lot_de_combinaison(&c, &mut b),
                Err(Arret::Usage(format!("touche inconnue : {nom}"))),
                "{nom}"
            );
        }
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            touches(&lot_de_combinaison(&seul, &mut b).unwrap_or_default()),
            [(0xA0, false), (0xBF, false), (0xBF, true), (0xA0, true)]
        );
    }
    #[test]
    fn comme_au_clavier_refuse_avant_tout_envoi() {
        let mut b = FauxBureau::avec_appli();
        b.premier_plan = None;
        let i = x(
            &mut b,
            "taper --fenetre appli-essai --comme-au-clavier --texte añ",
        );
        assert_eq!(
            i,
            Issue::arret(Arret::Refus("caractère absent du clavier : « ñ »".into()))
        );
        assert_eq!(b.envois(), vec![vec![Evenement::SourisNulle]]);
    }
    #[test]
    fn comme_au_clavier_refuse_quand_verr_maj_est_allumee() {
        let mut b = FauxBureau::avec_appli();
        b.verr_maj = true;
        assert_eq!(
            x(
                &mut b,
                "taper --fenetre appli-essai --comme-au-clavier --texte a"
            ),
            Issue::arret(Arret::Refus("Verr. Maj est allumée".into()))
        );
        assert!(aucun_geste(&b));
        for s in [
            "taper --fenetre appli-essai --texte a",
            "touche --fenetre appli-essai barre",
        ] {
            let mut b = FauxBureau::avec_appli();
            b.verr_maj = true;
            assert_eq!(x(&mut b, s).code, Code::Fait, "{s}");
        }
    }
    // Par la disposition, chaque unité passe par `lot_clavier` : Entrée et Tab sans modificateur,
    // un CRLF en une seule Entrée.
    #[test]
    fn comme_au_clavier_frappe_chaque_unite() {
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(PathBuf::from("t.txt"), b"A\r\n\ta".to_vec())];
        assert_eq!(
            x(
                &mut b,
                "taper --fenetre appli-essai --comme-au-clavier --fichier t.txt"
            )
            .code,
            Code::Fait
        );
        assert_eq!(
            b.envois(),
            vec![
                vec![
                    t(0xA0, 0x2A, false),
                    t(0x41, 0x10, false),
                    t(0x41, 0x10, true),
                    t(0xA0, 0x2A, true)
                ],
                vec![t(0x0D, 0x1C, false), t(0x0D, 0x1C, true)],
                vec![t(0x09, 0x0F, false), t(0x09, 0x0F, true)],
                vec![t(0x41, 0x10, false), t(0x41, 0x10, true)],
            ]
        );
    }
    // Sans la disposition, un caractère part par son unité UTF-16, et la longueur au journal compte
    // les caractères, non les octets.
    #[test]
    fn taper_sans_la_disposition_part_en_unicode() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --texte été").code,
            Code::Fait
        );
        let unicode = |unite, relachee| Evenement::Unicode { unite, relachee };
        assert_eq!(
            b.envois(),
            vec![
                vec![unicode(0xE9, false), unicode(0xE9, true)],
                vec![unicode(0x74, false), unicode(0x74, true)],
                vec![unicode(0xE9, false), unicode(0xE9, true)],
            ]
        );
        assert_eq!(b.journal, vec![format!("{TETE}taper\tappli-essai\t0\t3\n")]);
    }
    // Un fichier lisible se décode, son saut de ligne final retiré, et un CRLF part en une Entrée ;
    // l'entrée standard se lit de même.
    #[test]
    fn taper_lit_un_fichier_et_l_entree() {
        let mut b = FauxBureau::avec_appli();
        b.fichiers = vec![(PathBuf::from("f.txt"), b"a\r\nb\n".to_vec())];
        assert_eq!(
            x(&mut b, "taper --fenetre appli-essai --fichier f.txt").code,
            Code::Fait
        );
        let unicode = |unite, relachee| Evenement::Unicode { unite, relachee };
        assert_eq!(
            b.envois(),
            vec![
                vec![unicode(0x61, false), unicode(0x61, true)],
                vec![t(0x0D, 0x1C, false), t(0x0D, 0x1C, true)],
                vec![unicode(0x62, false), unicode(0x62, true)],
            ]
        );
        assert_eq!(b.journal, vec![format!("{TETE}taper\tappli-essai\t0\t4\n")]);
        let mut b = FauxBureau::avec_appli();
        b.entree = Some(b"xy".to_vec());
        assert_eq!(x(&mut b, "taper --fenetre appli-essai -").code, Code::Fait);
        assert_eq!(b.envois().len(), 2);
    }
    // Le chemin d'un fichier illisible s'échappe : le message tient en une ligne.
    #[test]
    fn fichier_illisible_echappe_son_chemin() {
        let mut b = FauxBureau::avec_appli();
        let args: Vec<OsString> = [
            "pilote",
            "taper",
            "--fenetre",
            "appli-essai",
            "--fichier",
            "a\nb>c",
        ]
        .map(Into::into)
        .into();
        assert_eq!(
            executer(&args, &mut b),
            Issue::arret(Arret::Usage(r"fichier illisible : a\nb\>c".into()))
        );
    }
    #[test]
    fn sous_commande_pas_encore_livree_sort_en_64() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "focus --fenetre x"),
            Issue::arret(Arret::Usage("pas encore livrée : focus".into()))
        );
        assert_eq!(b.journal, vec![format!("{TETE}focus\t\t64\t\n")]);
        // Le journal s'ouvre, puis reçoit la ligne : rien ne lit une fenêtre.
        assert_eq!(
            b.appels,
            vec![
                AppelFaux::OuvrirJournal,
                AppelFaux::Ajout(format!("{TETE}focus\t\t64\t\n")),
            ]
        );
    }
    // Une sous-commande que clap accepte est une commande valide : elle exige le journal, et sort
    // en 1 s'il ne s'ouvre pas, avant de dire qu'elle n'est pas encore livrée.
    #[test]
    fn pas_encore_livree_exige_le_journal() {
        let mut b = FauxBureau::avec_appli();
        b.journal_ferme = true;
        assert_eq!(
            x(&mut b, "focus --fenetre x"),
            Issue::arret(Arret::Refus(
                "journal inaccessible : ouvert dans une autre application ?".into()
            ))
        );
        assert_eq!(b.appels, vec![AppelFaux::OuvrirJournal]);
        assert!(b.journal.is_empty());
    }
    #[test]
    fn lecture_ratee_sort_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.erreur_fenetres = Some(ErreurSysteme("EnumWindows".into()));
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec("échec du système : EnumWindows".into()))
        );
        assert!(aucun_geste(&b));
    }
    // Une lecture des processus ou une prise du verrou qui échouent sortent de même en 3, leur
    // texte échappé.
    #[test]
    fn processus_et_verrou_illisibles_sortent_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.erreur_processus = Some(ErreurSysteme("CreateToolhelp32Snapshot\n5".into()));
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec(
                r"échec du système : CreateToolhelp32Snapshot\n5".into()
            ))
        );
        assert!(aucun_geste(&b));
        let mut b = FauxBureau::avec_appli();
        b.verrou = Err(ErreurSysteme("CreateMutexW".into()));
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec("échec du système : CreateMutexW".into()))
        );
        assert!(aucun_geste(&b));
    }

    // Le journal dans `executer`.

    #[test]
    fn chaque_appel_a_sa_ligne_meme_refuse() {
        let mut b = FauxBureau::avec_appli();
        b.session = session_verrouillee();
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab").code,
            Code::Refus
        );
        assert_eq!(b.journal, vec![format!("{TETE}touche\tappli-essai\t1\t\n")]);
    }
    #[test]
    fn appel_mal_forme_a_sa_ligne_sans_ses_arguments() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "taper --fenetre xyz --txt secret").code,
            Code::Usage
        );
        assert_eq!(b.journal.len(), 1);
        assert!(b.journal[0].contains("\ttaper\t"));
        assert!(!b.journal[0].contains("secret"));
        assert!(!b.journal[0].contains("xyz"));
    }
    #[test]
    fn aide_a_sa_ligne_si_le_journal_s_ouvre() {
        let mut b = FauxBureau::avec_appli();
        let i = x(&mut b, "--help");
        assert_eq!(i.code, Code::Fait);
        assert!(!i.sortie.is_empty());
        assert_eq!(b.journal, vec![format!("{TETE}aide\t\t0\t\n")]);
        let mut b = FauxBureau::avec_appli();
        b.journal_ferme = true;
        assert_eq!(x(&mut b, "--help").code, Code::Fait);
        assert!(b.journal.is_empty());
    }
    // La version s'écrit en une ligne sur la sortie standard ; l'aide d'une sous-commande ne
    // remplit ni le processus ni la longueur.
    #[test]
    fn version_et_aide_de_taper() {
        let mut b = FauxBureau::avec_appli();
        assert_eq!(
            x(&mut b, "--version"),
            Issue::fait(vec![format!("pilote {VERSION}")])
        );
        assert_eq!(b.journal, vec![format!("{TETE}version\t\t0\t\n")]);
        let mut b = FauxBureau::avec_appli();
        let i = x(&mut b, "taper --help");
        assert_eq!(i.code, Code::Fait);
        assert!(i.sortie.len() > 1, "{:?}", i.sortie);
        assert_eq!(b.journal, vec![format!("{TETE}taper\t\t0\t\n")]);
    }
    #[test]
    fn aucune_ligne_ne_porte_un_texte_tape() {
        let mut b = FauxBureau::avec_appli();
        x(&mut b, "taper --fenetre appli-essai --texte secret-d-essai");
        assert_eq!(b.journal.len(), 1);
        assert!(
            b.journal[0].ends_with("\tappli-essai\t0\t14\n"),
            "{:?}",
            b.journal
        );
        assert!(!b.journal[0].contains("secret"));
    }
    #[test]
    fn ajout_au_journal_rate_apres_un_geste_sort_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.ajout_rate = true;
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec(
                "échec du système : le journal n'a pas reçu sa ligne".into()
            ))
        );
        assert_eq!(b.envois().len(), 1);
    }
    // Sans geste, une ligne que le journal refuse ne change pas le code : un refus, une lecture
    // ou une activation qui n'a rien eu à faire gardent le leur.
    #[test]
    fn ajout_rate_sans_geste_garde_le_code() {
        for (s, code) in [
            ("touche --fenetre appli-essai Alt+Tab", Code::Refus),
            ("fenetres", Code::Fait),
            ("activer --fenetre appli-essai", Code::Fait),
        ] {
            let mut b = FauxBureau::avec_appli();
            b.ajout_rate = true;
            assert_eq!(x(&mut b, s).code, code, "{s}");
            assert!(appele(&b, |a| matches!(a, AppelFaux::Ajout(_))), "{s}");
        }
        let mut b = FauxBureau::avec_appli();
        b.ajout_rate = true;
        b.session = session_verrouillee();
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab")
                .erreur
                .as_deref(),
            Some("poste verrouillé")
        );
    }
    // Un échec du système survenu après un geste garde son propre message quand le journal perd
    // aussi sa ligne, qu'il la refuse ou que son écriture panique.
    #[test]
    fn echec_garde_son_message_quand_le_journal_rate() {
        let regler: [fn(&mut FauxBureau); 2] =
            [|b| b.ajout_rate = true, |b| b.ajout_panique = true];
        for (n, perdre_la_ligne) in regler.iter().enumerate() {
            let mut b = FauxBureau::avec_appli();
            perdre_la_ligne(&mut b);
            b.insere_au_plus = Some(1);
            assert_eq!(
                x(&mut b, "touche --fenetre appli-essai Tab"),
                Issue::arret(Arret::Echec(
                    "échec du système : SendInput a inséré 1 événements sur 2 ; relâchement : 1 \
                     sur 1"
                        .into()
                )),
                "réglage {n}"
            );
            assert_eq!(b.envois().len(), 2, "réglage {n}");
            assert!(b.journal.is_empty(), "réglage {n}");
        }
    }
    #[test]
    fn panique_sort_en_3_avec_sa_ligne() {
        let mut b = FauxBureau::avec_appli();
        b.fenetres_panique = true;
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec("échec du système : panique".into()))
        );
        assert_eq!(b.journal, vec![format!("{TETE}touche\t\t3\t\n")]);
    }
    // Une panique pendant l'écriture de la ligne finale ne sort pas d'`executer` : la ligne est
    // perdue, comme un ajout refusé, et après un geste le code est 3.
    #[test]
    fn panique_du_journal_apres_un_geste_sort_en_3() {
        let mut b = FauxBureau::avec_appli();
        b.ajout_panique = true;
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec(
                "échec du système : le journal n'a pas reçu sa ligne".into()
            ))
        );
        assert_eq!(b.envois().len(), 1);
        assert_eq!(
            b.appels
                .iter()
                .filter(|a| matches!(a, AppelFaux::Ajout(_)))
                .count(),
            1
        );
        assert!(b.journal.is_empty());
    }
    // Sans geste, la panique du journal ne change pas l'issue : l'aide, dont le journal s'ouvre à
    // la ligne finale, un refus, une lecture et une activation qui n'a rien eu à faire rendent ce
    // qu'ils rendent quand le journal reçoit sa ligne.
    #[test]
    fn panique_du_journal_sans_geste_garde_l_issue() {
        for s in [
            "--help",
            "touche --fenetre appli-essai Alt+Tab",
            "fenetres",
            "activer --fenetre appli-essai",
        ] {
            let attendue = x(&mut FauxBureau::avec_appli(), s);
            let mut b = FauxBureau::avec_appli();
            b.ajout_panique = true;
            assert_eq!(x(&mut b, s), attendue, "{s}");
            assert!(appele(&b, |a| matches!(a, AppelFaux::Ajout(_))), "{s}");
        }
    }
    // Une panique du corps, puis celle du journal : l'issue reste la seule erreur de la panique.
    #[test]
    fn panique_du_corps_puis_du_journal_garde_son_message() {
        let mut b = FauxBureau::avec_appli();
        b.fenetres_panique = true;
        b.ajout_panique = true;
        assert_eq!(
            x(&mut b, "touche --fenetre appli-essai Tab"),
            Issue::arret(Arret::Echec("échec du système : panique".into()))
        );
    }
}
