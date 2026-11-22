//! Les essais de bout en bout des gardes, contre l'application d'essai, sur le runner Windows de la
//! CI : l'activation, le premier plan volé pendant une frappe, la fenêtre retenue parmi plusieurs,
//! la liste de refus et la règle de KeePassXC sur des copies renommées et sur les vrais Paramètres,
//! le verrou des gestes, le journal dans le profil redirigé, la sortie en UTF-8 et la version. Ils
//! pilotent le bureau, et ne tournent jamais sur le poste (décision 13 du plan maître).

#![cfg(windows)]

mod commun;

use commun::Appli;
use commun::CALME;
use commun::DELAI;
use commun::Profil;
use commun::Shell;
use commun::Sortie;
use commun::est_reduite;
use commun::fenetre_sans_titre_de;
use commun::lancer_appli;
use commun::pilote;
use commun::reduire_les_parametres;
use commun::shell;
use commun::sonder;
use commun::tenir_le_verrou;
use pilote::domaine::sortie::echapper;
use std::fs;
use std::fs::OpenOptions;
use std::os::windows::fs::OpenOptionsExt;
use std::thread;
use std::time::Duration;

/// Le plus long que les Paramètres mettent à ouvrir leur fenêtre.
const PARAMETRES_MAX: Duration = Duration::from_secs(15);

/// La saisie de l'application à laquelle le voleur prend le premier plan : avant elle, au moins
/// autant d'unités sont parties.
const SAISIES_AVANT_LE_VOL: usize = 100;

/// Le message du refus de KeePassXC (décision 3).
const KEEPASSXC: &str = "KeePassXC : touche et taper sont refusés dans cette version";

/// Le message du refus des Paramètres, le nom gardant la casse de l'exécutable.
const SYSTEMSETTINGS: &str = "fenêtre refusée : processus SystemSettings";

/// Vrai si la sortie de `fenetres` donne `premier-plan=oui` à la ligne du processus `processus`.
fn premier_plan_a(fenetres: &Sortie, processus: &str) -> bool {
    let marque = format!(" processus={processus} ");
    fenetres
        .lignes()
        .iter()
        .any(|l| l.contains(&marque) && l.ends_with(" premier-plan=oui"))
}

/// Vrai si l'application a reçu une frappe : un `keydown` ou une saisie de sa page, ou une touche
/// reçue par l'une de ses fenêtres hors de sa page.
fn a_recu_une_frappe(appli: &Appli) -> bool {
    appli.lire_evenements().iter().any(|e| e.est_une_frappe())
}

/// `activer` porte au premier plan une fenêtre d'arrière-plan, celle d'une copie puis celle de
/// l'application, que `fenetres` marque alors chacune à son tour.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn activer_une_fenetre_d_arriere_plan() {
    let _appli = lancer_appli("appli-essai", &[]);
    let _autre = lancer_appli("autre", &[]);
    let profil = Profil::neuf("arriere-plan");
    for processus in ["autre", "appli-essai"] {
        let a = pilote(&profil, &["activer", "--fenetre", processus], None);
        assert_eq!(a.code, 0, "{processus} : {}", a.message());
        let f = pilote(&profil, &["fenetres"], None);
        assert!(
            premier_plan_a(&f, processus),
            "{processus} : {:?}",
            f.lignes()
        );
    }
}

/// `activer` restaure une fenêtre réduite, puis lui donne le premier plan.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn activer_restaure_une_fenetre_reduite() {
    let appli = lancer_appli("appli-essai", &["--reduite"]);
    assert!(
        sonder(DELAI, || est_reduite(appli.pid)),
        "la fenêtre n'est pas réduite au départ"
    );
    let profil = Profil::neuf("reduite");
    let a = pilote(&profil, &["activer", "--fenetre", "appli-essai"], None);
    assert_eq!(a.code, 0, "{}", a.message());
    assert!(!est_reduite(appli.pid), "la fenêtre est restée réduite");
    let f = pilote(&profil, &["fenetres"], None);
    assert!(premier_plan_a(&f, "appli-essai"), "{:?}", f.lignes());
}

/// Une frappe qui active elle-même sa fenêtre ne perd aucun caractère : l'application, derrière une
/// copie lancée après elle, reçoit « bonjour » entier d'un seul `taper`, sans `activer` avant lui,
/// et la copie ne reçoit rien. Le focus n'entre dans la WebView2 qu'un instant après l'activation :
/// un caractère perdu là dirait un défaut de l'outil, jamais un essai à assouplir. Puis, la fenêtre
/// déjà au premier plan, deux `activer` qui n'ont rien à faire et un `taper` ne perdent rien non
/// plus. L'Alt entourant ne se force pas sur le runner : les tests du domaine le couvrent.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn activer_puis_taper_ne_perd_aucun_caractere() {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let autre = lancer_appli("autre", &[]);
    let profil = Profil::neuf("activer-taper");
    let f = pilote(&profil, &["fenetres"], None);
    assert_eq!(f.code, 0, "{}", f.message());
    assert!(
        !premier_plan_a(&f, "appli-essai"),
        "l'appli tient déjà le premier plan : l'essai ne prouverait rien de l'activation ; {:?}",
        f.lignes()
    );
    let t = pilote(
        &profil,
        &["taper", "--fenetre", "appli-essai", "--texte", "bonjour"],
        None,
    );
    assert_eq!(t.code, 0, "{}", t.message());
    assert!(
        appli.attendre("\"valeur\":\"bonjour\"", DELAI),
        "{:?}",
        appli.lignes()
    );
    thread::sleep(CALME);
    assert_eq!(appli.derniere_saisie("champ").as_deref(), Some("bonjour"));
    assert!(!a_recu_une_frappe(&autre), "{:?}", autre.lignes());
    let a = pilote(&profil, &["activer", "--fenetre", "appli-essai"], None);
    assert_eq!(a.code, 0, "{}", a.message());
    let f = pilote(&profil, &["fenetres"], None);
    assert!(premier_plan_a(&f, "appli-essai"), "{:?}", f.lignes());
    let a = pilote(&profil, &["activer", "--fenetre", "appli-essai"], None);
    assert_eq!(a.code, 0, "{}", a.message());
    let t = pilote(
        &profil,
        &["taper", "--fenetre", "appli-essai", "--texte", "salut"],
        None,
    );
    assert_eq!(t.code, 0, "{}", t.message());
    assert!(
        appli.attendre("\"valeur\":\"bonjoursalut\"", DELAI),
        "{:?}",
        appli.lignes()
    );
    thread::sleep(CALME);
    assert_eq!(
        appli.derniere_saisie("champ").as_deref(),
        Some("bonjoursalut")
    );
    assert!(!a_recu_une_frappe(&autre), "{:?}", autre.lignes());
}

/// Une autre fenêtre prend le premier plan au milieu d'une frappe de 10 000 caractères : la frappe
/// s'arrête en 1, le message compte les unités parties, et rien ne va au voleur. Le voleur part de
/// la 100e saisie que reçoit l'application, qui renouvelle son droit de céder le premier plan à
/// chaque `keydown` : 100 unités au moins sont donc parties, et moins de 10 000. La durée du
/// `taper` et le nombre d'unités parties s'écrivent sur la sortie d'erreur.
///
/// Ce qui reste hors de portée de l'outil : une unité partie juste après le dernier contrôle du
/// premier plan, au moment même du vol, va au voleur, sa page ou sa fenêtre (la fenêtre résiduelle
/// d'une unité, ADR-0005). Un rouge rare sur une frappe reçue par le voleur montre ce reste : il se
/// rapporte, il ne se relance pas.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_refuse_quand_une_autre_fenetre_vole_le_premier_plan() {
    let appli = lancer_appli(
        "appli-essai",
        &["--permettre-premier-plan", "--focus", "champ"],
    );
    let cible = appli
        .evenements
        .to_str()
        .expect("un chemin en Unicode")
        .to_string();
    let voleur = lancer_appli(
        "voleur",
        &[
            "--voler-premier-plan",
            &cible,
            &SAISIES_AVANT_LE_VOL.to_string(),
        ],
    );
    let profil = Profil::neuf("vol");
    let a = pilote(&profil, &["activer", "--fenetre", "appli-essai"], None);
    assert_eq!(a.code, 0, "{}", a.message());
    let f = pilote(&profil, &["fenetres"], None);
    assert!(
        f.lignes().iter().any(|l| {
            l.starts_with("[Fenetre] <Appli d'essai> processus=appli-essai ")
                && l.ends_with(" premier-plan=oui")
        }),
        "{:?}",
        f.lignes()
    );
    let texte = "a".repeat(10_000);
    let t = pilote(
        &profil,
        &["taper", "--fenetre", "appli-essai", "--texte", &texte],
        None,
    );
    let message = t.message();
    let parties: Option<usize> = message
        .strip_prefix("le premier plan a changé : ")
        .and_then(|reste| reste.strip_suffix(" unités sur 10000 sont parties"))
        .and_then(|k| k.parse().ok());
    eprintln!(
        "durée du taper : {} ms pour {} unités",
        t.duree.as_millis(),
        parties.map_or_else(|| "?".to_string(), |k| k.to_string())
    );
    // La dernière ligne du voleur, `vole` ou `vol manqué`, se guette. `vole` s'écrit dans les 20 ms
    // du vol, au pas du voleur ; une touche partie vers lui juste après le dernier contrôle du
    // premier plan ne s'écrit qu'après son `keydown`, l'IPC de la WebView2 et la boucle de sa
    // fenêtre, ou après la boucle seule si sa fenêtre la reçoit avant que le focus soit dans sa
    // WebView2. Le calme la laisse arriver avant que les lignes du voleur se jugent ; la valeur
    // reçue par l'application se lit ensuite.
    voleur.attendre("vol", DELAI);
    thread::sleep(CALME);
    let du_voleur = voleur.lignes();
    assert!(
        du_voleur.iter().any(|l| l == "vole") && !du_voleur.iter().any(|l| l == "vol manqué"),
        "le voleur n'a pas pris le premier plan : {du_voleur:?}"
    );
    assert!(
        !a_recu_une_frappe(&voleur),
        "le voleur a reçu une frappe : {du_voleur:?}"
    );
    assert_eq!(t.code, 1, "{message}");
    let parties = parties.unwrap_or_else(|| panic!("message inattendu : {message}"));
    assert!(parties < 10_000, "{message}");
    assert!(
        parties >= SAISIES_AVANT_LE_VOL,
        "{parties} unités parties, moins que les {SAISIES_AVANT_LE_VOL} saisies reçues avant le \
         vol : {message}"
    );
    let recue = appli.derniere_saisie("champ").unwrap_or_default();
    assert!(
        recue.chars().count() <= parties,
        "{} caractères reçus, {parties} unités parties",
        recue.chars().count()
    );
}

/// Deux fenêtres titrées du même processus : sans `--titre`, 2, qui les nomme ; avec `--titre`, la
/// frappe va à la fenêtre nommée, jamais au dialogue, ni à sa page ni à sa fenêtre. Les deux pages
/// disent `pret` avant toute frappe : `lancer_appli` rend la main au premier `pret`, qui peut être
/// celui du dialogue.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn deux_fenetres_titrees_sans_option_titre_sortent_en_2() {
    let appli = lancer_appli("appli-essai", &["--dialogue"]);
    for titre in ["Dialogue d'essai", "Appli d'essai"] {
        assert!(
            appli.attendre(&format!("{titre}\t{{\"genre\":\"pret\"}}"), DELAI),
            "{titre} : {:?}",
            appli.lignes()
        );
    }
    let profil = Profil::neuf("deux-fenetres");
    let s = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Tab"],
        None,
    );
    assert_eq!(
        (s.code, s.message()),
        (
            2,
            "plusieurs fenêtres de « appli-essai » : <Appli d'essai> <Dialogue d'essai> ; --titre \
             ou --pid départage"
                .to_string()
        )
    );
    let t = pilote(
        &profil,
        &[
            "touche",
            "--fenetre",
            "appli-essai",
            "--titre",
            "Appli d'essai",
            "Tab",
        ],
        None,
    );
    assert_eq!(t.code, 0, "{}", t.message());
    assert!(
        appli.attendre("\"key\":\"Tab\"", DELAI),
        "{:?}",
        appli.lignes()
    );
    thread::sleep(CALME);
    let e = appli.lire_evenements();
    assert!(
        e.iter()
            .any(|e| e.titre == "Appli d'essai" && e.genre == "touche" && e.key == "Tab"),
        "{e:?}"
    );
    assert!(
        !e.iter()
            .any(|e| e.titre == "Dialogue d'essai" && e.est_une_frappe()),
        "{e:?}"
    );
}

/// Une liste ouverte paraît comme une fenêtre sans titre de la WebView2 : elle ne compte pas parmi
/// les fenêtres de l'application, et Échap, qui la ferme, va à la fenêtre principale. Seuls les
/// `msedgewebview2` de l'arbre de l'application comptent, et aucune telle fenêtre ne paraît avant
/// `Alt+Bas` : c'est bien la liste que l'essai voit s'ouvrir.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn touche_avec_une_liste_ouverte_vise_la_fenetre_principale() {
    let appli = lancer_appli("appli-essai", &["--focus", "liste"]);
    let profil = Profil::neuf("liste");
    let liste_ouverte = || fenetre_sans_titre_de("msedgewebview2", appli.pid);
    assert!(
        sonder(DELAI, || !liste_ouverte()),
        "une fenêtre sans titre de msedgewebview2 paraît dans l'appli avant Alt+Bas"
    );
    let o = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Alt+Bas"],
        None,
    );
    assert_eq!(o.code, 0, "{}", o.message());
    assert!(
        sonder(DELAI, liste_ouverte),
        "la liste ouverte ne paraît pas comme une fenêtre de msedgewebview2"
    );
    let e = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Echap"],
        None,
    );
    assert_eq!(e.code, 0, "{}", e.message());
    assert!(
        sonder(DELAI, || !liste_ouverte()),
        "la liste est restée ouverte"
    );
}

/// Une copie nommée `SystemSettings` est refusée et absente de `fenetres`, tandis qu'une
/// application permise, à côté, reste visée et listée, et reçoit le Tab qui lui est destiné : le
/// refus ne lit que les enfants de chaque fenêtre, jamais tout le bureau. La copie ne reçoit rien.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn copie_systemsettings_refusee() {
    let permise = lancer_appli("appli-essai", &[]);
    let copie = lancer_appli("SystemSettings", &[]);
    let profil = Profil::neuf("systemsettings");
    let a = pilote(&profil, &["activer", "--fenetre", "SystemSettings"], None);
    assert_eq!((a.code, a.message()), (1, SYSTEMSETTINGS.to_string()));
    let t = pilote(
        &profil,
        &["taper", "--fenetre", "SystemSettings", "--texte", "x"],
        None,
    );
    assert_eq!((t.code, t.message()), (1, SYSTEMSETTINGS.to_string()));
    let f = pilote(&profil, &["fenetres"], None);
    assert_eq!(f.code, 0, "{}", f.message());
    let lignes = f.lignes();
    assert!(
        !lignes
            .iter()
            .any(|l| l.contains(" processus=SystemSettings ")),
        "{lignes:?}"
    );
    assert!(
        lignes.iter().any(|l| l.contains(" processus=appli-essai ")),
        "{lignes:?}"
    );
    let tab = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Tab"],
        None,
    );
    assert_eq!(tab.code, 0, "{}", tab.message());
    assert!(
        permise.attendre("\"key\":\"Tab\"", DELAI),
        "{:?}",
        permise.lignes()
    );
    thread::sleep(CALME);
    assert!(!a_recu_une_frappe(&copie), "{:?}", copie.lignes());
}

/// Une copie nommée `KeePassXC` refuse `taper` et `touche` en 1 (décision 3) : rien ne part.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn copie_keepassxc_refuse_taper_et_touche() {
    let copie = lancer_appli("KeePassXC", &["--focus", "champ"]);
    let profil = Profil::neuf("keepassxc");
    let t = pilote(
        &profil,
        &["taper", "--fenetre", "KeePassXC", "--texte", "x"],
        None,
    );
    assert_eq!((t.code, t.message()), (1, KEEPASSXC.to_string()));
    let k = pilote(&profil, &["touche", "--fenetre", "KeePassXC", "Tab"], None);
    assert_eq!((k.code, k.message()), (1, KEEPASSXC.to_string()));
    thread::sleep(CALME);
    assert!(!a_recu_une_frappe(&copie), "{:?}", copie.lignes());
}

/// Les vrais Paramètres, réduits, restent refusés : leur cadre réduit ne garde que ses propres
/// filles, et son AUMID ou l'absence d'enfant lisible le refuse. Le cadre doit être encore réduit
/// juste avant `activer` : sinon l'enfant `SystemSettings` refuserait seul, et l'essai ne
/// prouverait rien du cadre réduit. Cet état, le code reçu et la sortie de `fenetres` s'écrivent
/// sur la sortie d'erreur. La garde des Paramètres les ferme à la fin, essai en échec compris.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn parametres_reduits_restent_refuses() {
    let profil = Profil::neuf("parametres");
    let s = shell(Shell::Powershell51, "start ms-settings:", &profil);
    assert_eq!(s.code, 0, "{}", s.message());
    let mut lus = None;
    sonder(PARAMETRES_MAX, || {
        lus = reduire_les_parametres();
        lus.is_some()
    });
    let parametres = lus.expect("le runner n'a pas ouvert les Paramètres en 15 s");
    let chevrons = format!("<{}>", echapper(&parametres.titre));
    let reduit = parametres.est_reduit();
    let a = pilote(
        &profil,
        &["activer", "--fenetre", "ApplicationFrameHost"],
        None,
    );
    let f = pilote(&profil, &["fenetres"], None);
    eprintln!(
        "cadre réduit avant activer : {reduit} ; activer --fenetre ApplicationFrameHost : {} ; \
         fenetres : {:?}",
        a.code,
        f.lignes()
    );
    assert!(
        reduit,
        "le cadre des Paramètres n'était plus réduit avant activer : l'essai ne prouverait rien \
         du cadre réduit"
    );
    match a.code {
        1 => assert_eq!(a.message(), SYSTEMSETTINGS),
        2 => assert!(!a.message().contains(&chevrons), "{}", a.message()),
        autre => panic!("activer a rendu {autre} : {}", a.message()),
    }
    assert_eq!(f.code, 0, "{}", f.message());
    let debut = format!("[Fenetre] {chevrons} processus=ApplicationFrameHost");
    assert!(
        !f.lignes().iter().any(|l| l.starts_with(&debut)),
        "{:?}",
        f.lignes()
    );
}

/// Un geste qui trouve le verrou tenu l'attend 5 s, puis refuse en 1.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn deux_gestes_en_meme_temps_le_second_attend_5_s_puis_refuse() {
    let _appli = lancer_appli("appli-essai", &[]);
    let profil = Profil::neuf("verrou-tenu");
    let _teneur = tenir_le_verrou();
    let s = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Tab"],
        None,
    );
    assert_eq!(
        (s.code, s.message()),
        (1, "un autre geste est en cours depuis 5 s".to_string())
    );
    assert!(
        s.duree >= Duration::from_secs(5) && s.duree < Duration::from_secs(9),
        "{:?}",
        s.duree
    );
}

/// Un verrou que son détenteur, mort, a abandonné se reprend : le geste part, et le Tab arrive. Le
/// teneur vit, mort, jusqu'au retour de `pilote` : son handle garde le mutex abandonné en vie, et
/// `pilote` le reçoit en `WAIT_ABANDONED`. Un code 1 ou 3 dirait cet abandon mal traduit.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn un_verrou_abandonne_est_repris() {
    let appli = lancer_appli("appli-essai", &[]);
    let profil = Profil::neuf("verrou-abandonne");
    let mut teneur = tenir_le_verrou();
    teneur.tuer();
    let s = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Tab"],
        None,
    );
    drop(teneur);
    assert_eq!(s.code, 0, "{}", s.message());
    assert!(
        appli.attendre("\"key\":\"Tab\"", DELAI),
        "{:?}",
        appli.lignes()
    );
}

/// Le journal tombe dans le profil que reçoit `pilote`, une ligne de six champs par appel, sans
/// jamais le texte tapé.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn journal_tombe_dans_le_profil_redirige() {
    let _appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf("journal");
    let f = pilote(&profil, &["fenetres"], None);
    assert_eq!(f.code, 0, "{}", f.message());
    let t = pilote(
        &profil,
        &[
            "taper",
            "--fenetre",
            "appli-essai",
            "--texte",
            "secret-d-essai",
        ],
        None,
    );
    assert_eq!(t.code, 0, "{}", t.message());
    let chemin = profil.racine.join(r"AppData\Local\pilote\journal.tsv");
    let journal =
        fs::read_to_string(&chemin).unwrap_or_else(|e| panic!("{} : {e}", chemin.display()));
    let lignes: Vec<&str> = journal.lines().collect();
    assert_eq!(lignes.len(), 2, "{lignes:?}");
    for (ligne, sous_commande) in lignes.iter().zip(["fenetres", "taper"]) {
        let champs: Vec<&str> = ligne.split('\t').collect();
        assert_eq!(champs.len(), 6, "{ligne:?}");
        assert_eq!(champs[2], sous_commande, "{ligne:?}");
        assert!(!ligne.contains("secret"), "{ligne:?}");
    }
    assert!(lignes[1].ends_with("\t0\t14"), "{:?}", lignes[1]);
}

/// Un journal qu'une autre application tient ouvert sans partage refuse le geste avant qu'il parte.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn journal_verrouille_refuse_avant_tout_geste() {
    let appli = lancer_appli("appli-essai", &[]);
    let profil = Profil::neuf("journal-verrouille");
    let dossier = profil.racine.join(r"AppData\Local\pilote");
    fs::create_dir_all(&dossier).expect("le dossier du journal");
    let _tenu = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(dossier.join("journal.tsv"))
        .expect("le journal, ouvert sans partage");
    let s = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Tab"],
        None,
    );
    assert_eq!(
        (s.code, s.message()),
        (
            1,
            "journal inaccessible : ouvert dans une autre application ?".to_string()
        )
    );
    thread::sleep(CALME);
    assert!(!a_recu_une_frappe(&appli), "{:?}", appli.lignes());
}

/// La sortie d'erreur et la sortie standard de `pilote` sont en UTF-8, sans BOM.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn sortie_et_erreurs_en_utf8_sans_bom() {
    let _appli = lancer_appli("appli-essai", &[]);
    let profil = Profil::neuf("utf8");
    let a = pilote(&profil, &["activer", "--fenetre", "absent"], None);
    assert_eq!(a.code, 2, "{}", a.message());
    assert_eq!(a.erreur, "aucune fenêtre de « absent »\n".as_bytes());
    let f = pilote(&profil, &["fenetres"], None);
    assert_eq!(f.code, 0, "{}", f.message());
    assert!(!f.sortie.starts_with(&[0xEF, 0xBB, 0xBF]), "{:?}", f.sortie);
    let texte = String::from_utf8(f.sortie.clone()).expect("une sortie en UTF-8");
    assert!(texte.contains(" processus=appli-essai "), "{texte}");
}

/// La version porte le commit que la CI construit.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn version_porte_le_commit() {
    let profil = Profil::neuf("version");
    let commit = std::env::var("GITHUB_SHA").expect("GITHUB_SHA manque");
    let court = commit.get(..12).expect("un commit de 12 chiffres au moins");
    let s = pilote(&profil, &["--version"], None);
    assert_eq!(s.code, 0, "{}", s.message());
    assert_eq!(
        String::from_utf8_lossy(&s.sortie),
        format!("pilote 0.1.0 ({court})\n")
    );
}
