//! Les fenêtres : celles qui comptent (les candidates), la fenêtre retenue W et son processus P,
//! le premier plan accepté avant chaque unité (décision 11), et les lignes de `fenetres`
//! (décision 7). `--fenetre` nomme les racines, `--pid` en garde une seule ; les candidates de leur
//! arbre comptent, les refusées écartées ; `--titre` départage par égalité exacte. Un nom de
//! processus s'écrit sans `.exe`, et tout texte venu du système passe par `echapper` avant d'entrer
//! dans un message, un titre entre chevrons.

use crate::domaine::cli::Cible;
use crate::domaine::processus::Processus;
use crate::domaine::processus::arbre;
use crate::domaine::processus::meme_nom;
use crate::domaine::processus::processus_de;
use crate::domaine::processus::sans_exe;
use crate::domaine::refus::est_keepassxc;
use crate::domaine::refus::fenetre_refusee;
use crate::domaine::refus::processus_refuse;
use crate::domaine::sortie::Arret;
use crate::domaine::sortie::Ligne;
use crate::domaine::sortie::echapper;
use std::collections::BTreeSet;

/// L'identifiant d'une fenêtre (son `HWND`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdFenetre(pub u64);

/// Une fenêtre de premier niveau, telle que la couche Windows la lit : `masquee` dit le masquage
/// de DWM, vrai aussi quand il ne se lit pas, `outil` le style de fenêtre d'outil, `aumid`
/// l'identifiant d'application lu sur la fenêtre, et `pids_enfants` les processus de ses fenêtres
/// enfants, le sien compris s'il en porte, chacun lu : une fille dont le processus ne se lit pas
/// n'y paraît pas.
#[derive(Clone, Debug, PartialEq)]
pub struct FenetreLue {
    pub id: IdFenetre,
    pub pid: u32,
    pub titre: String,
    pub classe: String,
    pub visible: bool,
    pub masquee: bool,
    pub outil: bool,
    pub proprietaire: Option<IdFenetre>,
    pub reduite: bool,
    pub aumid: Option<String>,
    pub pids_enfants: Vec<u32>,
}

/// La fenêtre au premier plan, son processus, et la chaîne de ses propriétaires, du plus proche au
/// plus lointain, 16 au plus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PremierPlan {
    pub id: IdFenetre,
    pub pid: u32,
    pub proprietaires: Vec<IdFenetre>,
}

/// La fenêtre retenue W, son processus P, et l'arbre de P : P et ses descendants.
#[derive(Clone, Debug, PartialEq)]
pub struct Retenue {
    pub fenetre: FenetreLue,
    pub processus: Processus,
    pub arbre: BTreeSet<u32>,
}

/// Vrai si `f` compte : visible, ni masquée, ni fenêtre d'outil, et titrée. Une fenêtre réduite ou
/// possédée compte.
pub fn candidate(f: &FenetreLue) -> bool {
    f.visible && !f.masquee && !f.outil && !f.titre.is_empty()
}

/// La fenêtre que vise `cible` : les racines que nomme `--fenetre`, restreintes à `--pid` s'il est
/// donné, leur arbre, les candidates de cet arbre, les refusées écartées (refus 1 s'il n'en restait
/// que de refusées), `--titre` par égalité exacte, puis le compte : aucune ou plusieurs sortent
/// en 2. P est le processus de W, et l'arbre retenu, P et ses descendants.
pub fn retenir(
    cible: &Cible,
    fenetres: &[FenetreLue],
    tous: &[Processus],
) -> Result<Retenue, Arret> {
    let racines: Vec<u32> = tous
        .iter()
        .filter(|p| meme_nom(&p.nom, &cible.fenetre) && cible.pid.is_none_or(|pid| pid == p.pid))
        .map(|p| p.pid)
        .collect();
    let arbre_des_racines = arbre(&racines, tous);
    let mut candidates: Vec<(&FenetreLue, &Processus)> = fenetres
        .iter()
        .filter(|f| candidate(f) && arbre_des_racines.contains(&f.pid))
        .filter_map(|f| processus_de(f.pid, tous).map(|p| (f, p)))
        .collect();
    candidates.sort_by_cached_key(|&(f, _)| cle(f, tous));
    let mut permises: Vec<(&FenetreLue, &Processus)> = candidates
        .iter()
        .copied()
        .filter(|&(f, _)| fenetre_refusee(f, tous).is_none())
        .collect();
    if permises.is_empty()
        && let Some(nom) = candidates
            .iter()
            .find_map(|&(f, _)| fenetre_refusee(f, tous))
    {
        return Err(refus_du_processus(&nom));
    }
    if let Some(titre) = &cible.titre {
        permises.retain(|&(f, _)| f.titre == *titre);
    }
    match permises[..] {
        [] => Err(aucune_fenetre(cible)),
        [(w, p)] => Ok(Retenue {
            fenetre: w.clone(),
            processus: p.clone(),
            arbre: arbre(&[p.pid], tous),
        }),
        _ => {
            let titres: Vec<String> = permises
                .iter()
                .map(|&(f, _)| format!("<{}>", echapper(&f.titre)))
                .collect();
            Err(Arret::Introuvable(format!(
                "plusieurs fenêtres de « {} » : {} ; --titre ou --pid départage",
                echapper(&cible.fenetre),
                titres.join(" ")
            )))
        }
    }
}

/// Le refus d'une fenêtre que la liste de refus écarte pour son processus `nom` (1), à la retenue
/// comme après l'activation : « fenêtre refusée : processus <nom> », le nom échappé.
pub fn refus_du_processus(nom: &str) -> Arret {
    Arret::Refus(format!("fenêtre refusée : processus {}", echapper(nom)))
}

/// Aucune fenêtre pour `cible` (2), à la retenue comme après l'activation : « aucune fenêtre de
/// « <nom> » », le nom demandé échappé.
pub fn aucune_fenetre(cible: &Cible) -> Arret {
    Arret::Introuvable(format!(
        "aucune fenêtre de « {} »",
        echapper(&cible.fenetre)
    ))
}

/// Vrai si le premier plan `pp` est accepté pour la retenue `r` : c'est W, son HWND et son
/// processus ; ou la chaîne de ses propriétaires mène à W, et son processus, lu dans `tous`, est de
/// l'arbre de P sans que la liste de refus ni la règle de KeePassXC le refusent. L'arbre de P se
/// juge à la retenue et sur la liste relue `tous` : rien de né après la retenue, rien dont le PID
/// a été repris depuis. Le HWND de W sous un autre processus, un premier plan nul, une autre
/// fenêtre de P que W ne possède pas, ou un processus qui ne se lit pas, ne sont jamais acceptés.
pub fn accepte(pp: Option<&PremierPlan>, r: &Retenue, tous: &[Processus]) -> bool {
    let Some(pp) = pp else {
        return false;
    };
    if pp.id == r.fenetre.id {
        return pp.pid == r.fenetre.pid;
    }
    pp.proprietaires.contains(&r.fenetre.id)
        && r.arbre.contains(&pp.pid)
        && arbre(&[r.processus.pid], tous).contains(&pp.pid)
        && processus_de(pp.pid, tous)
            .is_some_and(|p| !processus_refuse(&p.nom) && !est_keepassxc(&p.nom))
}

/// La ligne de `fenetres` pour `f` : `[Fenetre] <titre> processus=<nom> pid=<pid>`, le nom sans
/// `.exe` (vide si le processus ne se lit pas), plus `premier-plan=oui` si `premier_plan`. Le titre
/// et le nom y sont tels quels : `formater` les échappe.
pub fn ligne_de(f: &FenetreLue, tous: &[Processus], premier_plan: bool) -> Ligne {
    let mut champs = vec![
        ("processus", nom_du_processus(f.pid, tous).to_string()),
        ("pid", f.pid.to_string()),
    ];
    if premier_plan {
        champs.push(("premier-plan", "oui".to_string()));
    }
    Ligne {
        genre: "Fenetre",
        nom: Some(f.titre.clone()),
        champs,
    }
}

/// Les lignes de `fenetres` : les candidates de tous les processus, sauf les refusées, triées par
/// nom de processus, puis PID, puis titre ; celle qui est au premier plan porte `premier-plan=oui`.
pub fn lister(fenetres: &[FenetreLue], tous: &[Processus], pp: Option<&PremierPlan>) -> Vec<Ligne> {
    let mut listees: Vec<&FenetreLue> = fenetres
        .iter()
        .filter(|f| candidate(f) && fenetre_refusee(f, tous).is_none())
        .collect();
    listees.sort_by_cached_key(|f| cle(f, tous));
    listees
        .into_iter()
        .map(|f| ligne_de(f, tous, pp.is_some_and(|pp| pp.id == f.id)))
        .collect()
}

/// Le nom du processus `pid`, sans `.exe` ; vide s'il ne se lit pas.
fn nom_du_processus(pid: u32, tous: &[Processus]) -> &str {
    processus_de(pid, tous).map_or("", |p| sans_exe(&p.nom))
}

/// La clé de l'ordre des fenêtres (décision 7) : le nom du processus, sans égard à la casse puis
/// tel quel, puis le PID, puis le titre.
fn cle<'a>(f: &'a FenetreLue, tous: &'a [Processus]) -> (String, &'a str, u32, &'a str) {
    let nom = nom_du_processus(f.pid, tous);
    (nom.to_lowercase(), nom, f.pid, &f.titre)
}

/// La fenêtre dont le fil donne la disposition du clavier (décision 2) : la fenêtre focalisée, à
/// défaut l'active, à défaut celle du premier plan ; `None` si les trois manquent.
pub fn fil_focalise(
    focus: Option<IdFenetre>,
    active: Option<IdFenetre>,
    premier_plan: Option<IdFenetre>,
) -> Option<IdFenetre> {
    focus.or(active).or(premier_plan)
}

/// Les constructeurs des tests, que `processus.rs`, `refus.rs` et `faux.rs` réemploient.
#[cfg(test)]
pub(crate) mod essai {
    use super::FenetreLue;
    use super::IdFenetre;
    use super::PremierPlan;
    use super::Retenue;
    use crate::domaine::processus::Processus;

    /// Un processus.
    pub(crate) fn p(pid: u32, parent: u32, nom: &str, creation: Option<u64>) -> Processus {
        Processus {
            pid,
            parent,
            nom: nom.to_string(),
            creation,
        }
    }

    /// Une fenêtre candidate : visible, ni masquée, ni outil, ni réduite, de classe `Fenetre`, sans
    /// propriétaire, AUMID ni enfants.
    pub(crate) fn f(id: u64, pid: u32, titre: &str) -> FenetreLue {
        FenetreLue {
            id: IdFenetre(id),
            pid,
            titre: titre.to_string(),
            classe: "Fenetre".to_string(),
            visible: true,
            masquee: false,
            outil: false,
            proprietaire: None,
            reduite: false,
            aumid: None,
            pids_enfants: Vec::new(),
        }
    }

    /// Un premier plan.
    pub(crate) fn pp(id: u64, pid: u32, proprietaires: Vec<IdFenetre>) -> PremierPlan {
        PremierPlan {
            id: IdFenetre(id),
            pid,
            proprietaires,
        }
    }

    /// Une retenue et les processus du poste : W, la fenêtre 1 « Appli d'essai » du processus 100
    /// « appli-essai.exe » (P), son fils 101 « fils.exe », d'un nom permis et né après lui, et
    /// l'arbre {100, 101}.
    pub(crate) fn retenue_avec_arbre() -> (Retenue, Vec<Processus>) {
        let tous = vec![
            p(100, 1, "appli-essai.exe", Some(5)),
            p(101, 100, "fils.exe", Some(6)),
        ];
        let retenue = Retenue {
            fenetre: f(1, 100, "Appli d'essai"),
            processus: tous[0].clone(),
            arbre: [100, 101].into(),
        };
        (retenue, tous)
    }
}

#[cfg(test)]
mod tests {
    use super::essai::f;
    use super::essai::p;
    use super::essai::pp;
    use super::essai::retenue_avec_arbre;
    use super::*;
    use crate::domaine::cli::essai::cible;
    use crate::domaine::refus::AUMID_PARAMETRES;
    use crate::domaine::refus::CLASSE_CADRE;
    use crate::domaine::sortie::formater;

    /// Une cible qui départage par le titre.
    fn cible_titre(fenetre: &str, titre: &str) -> Cible {
        Cible {
            titre: Some(titre.into()),
            ..cible(fenetre)
        }
    }

    /// Une cible qui départage par le PID.
    fn cible_pid(fenetre: &str, pid: u32) -> Cible {
        Cible {
            pid: Some(pid),
            ..cible(fenetre)
        }
    }

    /// Les lignes de `lister`, formées.
    fn lignes(
        fenetres: &[FenetreLue],
        tous: &[Processus],
        pp: Option<&PremierPlan>,
    ) -> Vec<String> {
        lister(fenetres, tous, pp).iter().map(formater).collect()
    }

    #[test]
    fn fenetre_candidate_exclut_masquee_outil_sans_titre_invisible() {
        let base = f(1, 10, "A");
        assert!(candidate(&base));
        let gestes: [fn(&mut FenetreLue); 4] = [
            |w| w.masquee = true,
            |w| w.outil = true,
            |w| w.titre.clear(),
            |w| w.visible = false,
        ];
        for g in gestes {
            let mut w = base.clone();
            g(&mut w);
            assert!(!candidate(&w));
        }
    }
    #[test]
    fn fil_focalise_se_replie_sur_l_active_puis_le_premier_plan() {
        let (f, a, p) = (Some(IdFenetre(1)), Some(IdFenetre(2)), Some(IdFenetre(3)));
        assert_eq!(fil_focalise(f, a, p), f);
        assert_eq!(fil_focalise(f, None, None), f);
        assert_eq!(fil_focalise(None, a, p), a);
        assert_eq!(fil_focalise(None, a, None), a);
        assert_eq!(fil_focalise(None, None, p), p);
        assert_eq!(fil_focalise(None, None, None), None);
    }
    #[test]
    fn retient_la_seule_fenetre_de_l_arbre() {
        let t = [
            p(100, 1, "appli-essai.exe", Some(5)),
            p(101, 100, "x.exe", Some(6)),
        ];
        let r = retenir(&cible("appli-essai"), &[f(1, 100, "Appli d'essai")], &t).unwrap();
        assert_eq!(
            (r.fenetre.id, r.processus.pid, r.arbre.clone()),
            (IdFenetre(1), 100, [100, 101].into())
        );
    }
    #[test]
    fn fenetre_d_un_descendant_est_candidate() {
        let t = [
            p(100, 1, "appli.exe", Some(5)),
            p(101, 100, "fils.exe", Some(6)),
        ];
        let r = retenir(&cible("appli"), &[f(1, 101, "Fils")], &t).unwrap();
        assert_eq!(
            (r.fenetre.id, r.processus.clone(), r.arbre.clone()),
            (IdFenetre(1), t[1].clone(), [101].into())
        );
    }
    #[test]
    fn aucune_fenetre_sort_en_2() {
        assert_eq!(
            retenir(&cible("absent"), &[], &[]).unwrap_err(),
            Arret::Introuvable("aucune fenêtre de « absent »".into())
        );
    }
    #[test]
    fn deux_fenetres_titrees_sans_option_titre_sortent_en_2() {
        assert_eq!(
            retenir(
                &cible("appli-essai"),
                &[f(1, 100, "Appli d'essai"), f(2, 100, "Dialogue d'essai")],
                &[p(100, 0, "appli-essai.exe", Some(1))]
            )
            .unwrap_err(),
            Arret::Introuvable(
                "plusieurs fenêtres de « appli-essai » : <Appli d'essai> <Dialogue d'essai> ; --titre ou --pid départage"
                    .into()
            )
        );
    }
    #[test]
    fn titre_departage_par_egalite_exacte() {
        let fenetres = [f(1, 100, "Appli d'essai"), f(2, 100, "Dialogue d'essai")];
        let t = [p(100, 0, "appli-essai.exe", Some(1))];
        let r = retenir(
            &cible_titre("appli-essai", "Dialogue d'essai"),
            &fenetres,
            &t,
        )
        .unwrap();
        assert_eq!(r.fenetre.id, IdFenetre(2));
        assert_eq!(
            retenir(&cible_titre("appli-essai", "Dialogue"), &fenetres, &t).unwrap_err(),
            Arret::Introuvable("aucune fenêtre de « appli-essai »".into())
        );
    }
    #[test]
    fn pid_garde_la_seule_racine_choisie() {
        let fenetres = [f(1, 100, "Etabli"), f(2, 200, "Etabli")];
        let t = [
            p(100, 0, "etabli.exe", Some(1)),
            p(200, 0, "etabli.exe", Some(2)),
        ];
        let r = retenir(&cible_pid("etabli", 200), &fenetres, &t).unwrap();
        assert_eq!((r.fenetre.id, r.processus.pid), (IdFenetre(2), 200));
        assert!(!r.arbre.contains(&100));
        assert!(matches!(
            retenir(&cible("etabli"), &fenetres, &t),
            Err(Arret::Introuvable(m)) if m.starts_with("plusieurs fenêtres de « etabli » : ")
        ));
    }
    #[test]
    fn candidates_refusees_s_ecartent_avant_le_compte() {
        let t = [
            p(5, 0, "ApplicationFrameHost.exe", Some(1)),
            p(7, 0, "SystemSettings.exe", Some(2)),
            p(30, 0, "CalculatorApp.exe", Some(2)),
        ];
        let mut parametres = f(1, 5, "Paramètres");
        parametres.classe = CLASSE_CADRE.into();
        parametres.aumid = Some(AUMID_PARAMETRES.into());
        parametres.pids_enfants = vec![5, 7];
        let mut calculatrice = f(2, 5, "Calculatrice");
        calculatrice.classe = CLASSE_CADRE.into();
        calculatrice.aumid = Some("Microsoft.WindowsCalculator_8wekyb3d8bbwe!App".into());
        calculatrice.pids_enfants = vec![5, 30];
        let r = retenir(
            &cible("ApplicationFrameHost"),
            &[parametres, calculatrice],
            &t,
        )
        .unwrap();
        assert_eq!(r.fenetre.id, IdFenetre(2));
    }
    #[test]
    fn seules_des_refusees_refusent_en_1() {
        assert_eq!(
            retenir(
                &cible("SystemSettings"),
                &[f(1, 7, "Paramètres")],
                &[p(7, 0, "SystemSettings.exe", Some(1))]
            )
            .unwrap_err(),
            Arret::Refus("fenêtre refusée : processus SystemSettings".into())
        );
    }
    #[test]
    fn plusieurs_sans_citer_une_refusee() {
        let t = [
            p(100, 0, "appli.exe", Some(1)),
            p(101, 100, "SystemSettings.exe", Some(2)),
        ];
        let fenetres = [f(1, 100, "Un"), f(3, 101, "Paramètres"), f(2, 100, "Deux")];
        assert_eq!(
            retenir(&cible("appli"), &fenetres, &t).unwrap_err(),
            Arret::Introuvable(
                "plusieurs fenêtres de « appli » : <Deux> <Un> ; --titre ou --pid départage".into()
            )
        );
    }
    #[test]
    fn premier_plan_accepte_w_ou_ce_que_w_possede() {
        let (r, t) = retenue_avec_arbre(); // W = 1, P = 100, fils 101
        assert!(accepte(Some(&pp(1, 100, vec![])), &r, &t));
        assert!(accepte(Some(&pp(9, 101, vec![IdFenetre(1)])), &r, &t));
        assert!(accepte(
            Some(&pp(8, 100, vec![IdFenetre(9), IdFenetre(1)])),
            &r,
            &t
        ));
    }
    #[test]
    fn premier_plan_refuse_une_autre_fenetre_de_p() {
        let (r, t) = retenue_avec_arbre();
        assert!(!accepte(Some(&pp(2, 100, vec![])), &r, &t));
    }
    #[test]
    fn premier_plan_refuse_un_dialogue_d_un_autre_arbre() {
        let (r, t) = retenue_avec_arbre();
        assert!(!accepte(Some(&pp(9, 555, vec![IdFenetre(1)])), &r, &t));
    }
    #[test]
    fn premier_plan_refuse_un_dialogue_refuse() {
        let (r, mut t) = retenue_avec_arbre();
        t[1].nom = "CredentialUIBroker.exe".into();
        assert!(!accepte(Some(&pp(9, 101, vec![IdFenetre(1)])), &r, &t));
    }
    #[test]
    fn premier_plan_refuse_un_dialogue_de_keepassxc() {
        let (r, mut t) = retenue_avec_arbre();
        t[1].nom = "KeePassXC.exe".into();
        assert!(!accepte(Some(&pp(9, 101, vec![IdFenetre(1)])), &r, &t));
    }
    #[test]
    fn premier_plan_nul_refuse() {
        let (r, t) = retenue_avec_arbre();
        assert!(!accepte(None, &r, &t));
    }
    #[test]
    fn deux_racines_de_meme_nom_le_premier_plan_sur_l_autre_refuse() {
        let fenetres = [f(1, 100, "Etabli"), f(2, 200, "Etabli")];
        let t = [
            p(100, 0, "etabli.exe", Some(1)),
            p(200, 0, "etabli.exe", Some(2)),
        ];
        let r = retenir(&cible_pid("etabli", 100), &fenetres, &t).unwrap();
        assert!(accepte(Some(&pp(1, 100, vec![])), &r, &t));
        assert!(!accepte(Some(&pp(2, 200, vec![])), &r, &t));
    }
    #[test]
    fn fenetres_liste_sans_les_refusees_avec_le_premier_plan() {
        let t = [
            p(2, 0, "zeta.exe", Some(1)),
            p(3, 0, "alpha.exe", Some(1)),
            p(7, 0, "SystemSettings.exe", Some(1)),
        ];
        let fenetres = [f(10, 2, "Z"), f(11, 7, "Paramètres"), f(12, 3, "A")];
        assert_eq!(
            lignes(&fenetres, &t, Some(&pp(10, 2, vec![]))),
            [
                "[Fenetre] <A> processus=alpha pid=3",
                "[Fenetre] <Z> processus=zeta pid=2 premier-plan=oui"
            ]
        );
    }

    // L'ordre de `fenetres` (décision 7), lu par ses valeurs : le nom de processus d'abord, sans
    // égard à la casse (`Gamma` après `beta`), puis tel quel (`Beta` avant `beta`), puis le PID à
    // nom égal, puis le titre à PID égal ; les non-candidates n'y paraissent pas, et sans premier
    // plan aucune ligne ne porte `premier-plan=oui`.
    #[test]
    fn fenetres_triees_par_nom_puis_pid_puis_titre() {
        let t = [
            p(9, 0, "beta.exe", Some(1)),
            p(6, 0, "Gamma.exe", Some(1)),
            p(4, 0, "beta.exe", Some(1)),
            p(20, 0, "Beta.exe", Some(1)),
            p(5, 0, "alpha.exe", Some(1)),
        ];
        let mut cachee = f(15, 5, "Tao Thread Event Target");
        cachee.visible = false;
        let fenetres = [
            f(10, 9, "a"),
            f(14, 6, "g"),
            f(11, 4, "b"),
            f(12, 4, "a"),
            f(16, 20, "c"),
            f(13, 5, "z"),
            cachee,
        ];
        assert_eq!(
            lignes(&fenetres, &t, None),
            [
                "[Fenetre] <z> processus=alpha pid=5",
                "[Fenetre] <c> processus=Beta pid=20",
                "[Fenetre] <a> processus=beta pid=4",
                "[Fenetre] <b> processus=beta pid=4",
                "[Fenetre] <a> processus=beta pid=9",
                "[Fenetre] <g> processus=Gamma pid=6",
            ]
        );
    }
    // La ligne d'une fenêtre : le titre et le nom tels quels (`formater` les échappe), le nom sans
    // `.exe`, vide si le processus ne se lit pas.
    #[test]
    fn ligne_de_forme_exacte() {
        let t = [p(100, 0, "Appli-Essai.EXE", Some(1))];
        assert_eq!(
            ligne_de(&f(1, 100, "a>b"), &t, true),
            Ligne {
                genre: "Fenetre",
                nom: Some("a>b".into()),
                champs: vec![
                    ("processus", "Appli-Essai".into()),
                    ("pid", "100".into()),
                    ("premier-plan", "oui".into()),
                ],
            }
        );
        assert_eq!(
            formater(&ligne_de(&f(2, 300, "Sans processus"), &t, false)),
            r#"[Fenetre] <Sans processus> processus="" pid=300"#
        );
    }
    // Une fenêtre hors de l'arbre, ou qui n'est pas candidate, ne compte pas : seule la fenêtre
    // titrée et visible de l'arbre est retenue.
    #[test]
    fn hors_de_l_arbre_ou_non_candidate_ne_compte_pas() {
        let t = [
            p(100, 0, "appli-essai.exe", Some(1)),
            p(300, 0, "autre.exe", Some(1)),
        ];
        let mut cachee = f(2, 100, "Tao Thread Event Target");
        cachee.visible = false;
        let fenetres = [f(1, 100, "Appli d'essai"), cachee, f(3, 300, "Autre")];
        let r = retenir(&cible("appli-essai"), &fenetres, &t).unwrap();
        assert_eq!(r.fenetre.id, IdFenetre(1));
    }
    // `--pid` ne garde qu'une racine nommée par `--fenetre` : un PID d'un autre nom ne retient rien.
    #[test]
    fn pid_d_un_autre_nom_ne_retient_rien() {
        let t = [
            p(100, 0, "etabli.exe", Some(1)),
            p(300, 0, "autre.exe", Some(1)),
        ];
        assert_eq!(
            retenir(&cible_pid("etabli", 300), &[f(3, 300, "Autre")], &t).unwrap_err(),
            Arret::Introuvable("aucune fenêtre de « etabli »".into())
        );
    }
    // Le refus se juge avant `--titre` : quand seules des refusées restent, `--titre` n'y change
    // rien, et la sortie est 1.
    #[test]
    fn refus_avant_le_titre() {
        assert_eq!(
            retenir(
                &cible_titre("SystemSettings", "Autre"),
                &[f(1, 7, "Paramètres")],
                &[p(7, 0, "SystemSettings.exe", Some(1))]
            )
            .unwrap_err(),
            Arret::Refus("fenêtre refusée : processus SystemSettings".into())
        );
    }
    // Le nom demandé et les titres cités s'échappent : un message tient en une ligne, et un `>`
    // d'un titre ne ferme pas ses chevrons.
    #[test]
    fn noms_et_titres_echappes_dans_les_messages() {
        let t = [p(100, 0, "appli.exe", Some(1))];
        assert_eq!(
            retenir(&cible(r"a>b\c"), &[], &t).unwrap_err(),
            Arret::Introuvable(r"aucune fenêtre de « a\>b\\c »".into())
        );
        assert_eq!(
            retenir(&cible("appli"), &[f(1, 100, "x>y"), f(2, 100, "z\nw")], &t).unwrap_err(),
            Arret::Introuvable(
                r"plusieurs fenêtres de « appli » : <x\>y> <z\nw> ; --titre ou --pid départage"
                    .into()
            )
        );
    }
    // Un premier plan possédé par W, dont le processus de l'arbre ne se lit plus, n'est pas
    // accepté.
    #[test]
    fn premier_plan_d_un_processus_illisible_refuse() {
        let (r, t) = retenue_avec_arbre();
        assert!(!accepte(Some(&pp(9, 101, vec![IdFenetre(1)])), &r, &t[..1]));
    }
    // Windows peut redonner la valeur d'un HWND détruit : le premier plan qui porte celui de W
    // n'est W que si son processus est P. Repris par un autre processus, même un descendant de P,
    // il est refusé, sans retomber sur la règle des fenêtres possédées.
    #[test]
    fn hwnd_de_w_repris_par_un_autre_processus_refuse() {
        let (r, t) = retenue_avec_arbre();
        assert!(!accepte(Some(&pp(1, 555, vec![])), &r, &t));
        assert!(!accepte(Some(&pp(1, 101, vec![])), &r, &t));
        assert!(!accepte(Some(&pp(1, 101, vec![IdFenetre(1)])), &r, &t));
    }
    // Un PID se réutilise pendant une frappe : une fenêtre possédée par W n'est acceptée que si son
    // processus est encore de l'arbre de P sur la liste relue. Le PID 101 repris par un étranger,
    // né après P d'un père hors de l'arbre, ou d'un père qui porte le PID de P mais né avant P,
    // est refusé.
    #[test]
    fn pid_de_l_arbre_repris_par_un_etranger_refuse() {
        let (r, mut t) = retenue_avec_arbre();
        let possedee = pp(9, 101, vec![IdFenetre(1)]);
        assert!(accepte(Some(&possedee), &r, &t));
        let naissance_de_p = r.processus.creation.unwrap();
        let i = t.iter().position(|q| q.pid == 101).unwrap();
        t[i] = p(101, 999, "etranger.exe", Some(naissance_de_p + 2));
        assert!(!accepte(Some(&possedee), &r, &t));
        t[i] = p(101, 100, "etranger.exe", Some(naissance_de_p - 1));
        assert!(!accepte(Some(&possedee), &r, &t));
    }
    // L'arbre de P se juge aussi à la retenue : un processus né ensuite, même un vrai fils de P,
    // n'en est pas, et sa fenêtre possédée par W est refusée.
    #[test]
    fn processus_ne_apres_la_retenue_refuse() {
        let (r, mut t) = retenue_avec_arbre();
        let naissance_de_p = r.processus.creation.unwrap();
        t.push(p(102, 100, "nouveau.exe", Some(naissance_de_p + 3)));
        assert!(!accepte(Some(&pp(9, 102, vec![IdFenetre(1)])), &r, &t));
    }
}
