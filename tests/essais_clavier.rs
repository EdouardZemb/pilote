//! Les essais de bout en bout du clavier et des shells, contre l'application d'essai, sur le runner
//! Windows de la CI : les accents, un caractère hors du plan de base et un CRLF, en UTF-16 ; AltGr
//! et la touche `barre`, par la disposition française ; Tab ; les refus de `--comme-au-clavier` ;
//! puis `pilote` appelé depuis PowerShell 5.1 et depuis Git Bash. Ils pilotent le bureau, et ne
//! tournent jamais sur le poste (décision 13 du plan maître).

#![cfg(windows)]

mod commun;

use commun::Appli;
use commun::CALME;
use commun::DELAI;
use commun::Evenement;
use commun::Profil;
use commun::Shell;
use commun::VerrMaj;
use commun::exiger_la_disposition_francaise;
use commun::lancer_appli;
use commun::pilote;
use commun::shell;
use commun::sonder;
use std::fs;
use std::thread;

/// Lance l'application, le focus sur `champ`, y tape `texte` par `taper --texte`, et vérifie que la
/// dernière saisie de `champ` le porte, lue après le calme : une unité de trop arriverait après la
/// saisie guettée. `texte` ne porte rien que `JSON.stringify` échappe : sa ligne se guette telle
/// quelle.
fn taper_dans_le_champ(nom: &str, texte: &str) {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf(nom);
    let s = pilote(
        &profil,
        &["taper", "--fenetre", "appli-essai", "--texte", texte],
        None,
    );
    assert_eq!(s.code, 0, "{}", s.message());
    let motif = format!("\"valeur\":\"{texte}\"");
    assert!(appli.attendre(&motif, DELAI), "{:?}", appli.lignes());
    thread::sleep(CALME);
    assert_eq!(appli.derniere_saisie("champ").as_deref(), Some(texte));
}

/// Le rang du premier événement de `evenements`, à partir du rang `debut`, qui répond à `critere`.
fn rang(
    evenements: &[Evenement],
    debut: usize,
    critere: impl Fn(&Evenement) -> bool,
) -> Option<usize> {
    evenements
        .iter()
        .enumerate()
        .skip(debut)
        .find(|(_, e)| critere(e))
        .map(|(i, _)| i)
}

/// Vrai si, à partir du rang `debut`, la `touche` `key` paraît, puis, après elle, un `focus` de
/// l'élément `id`, avant 5 s. La page dit le `keydown` à la capture, avant que son raccourci ou
/// l'action par défaut déplacent le focus : un focus qui suit la touche en vient.
fn focus_apres_la_touche(appli: &Appli, debut: usize, key: &str, id: &str) -> bool {
    sonder(DELAI, || {
        let evenements = appli.lire_evenements();
        rang(&evenements, debut, |e| e.genre == "touche" && e.key == key)
            .and_then(|touche| {
                rang(&evenements, touche + 1, |e| {
                    e.genre == "focus" && e.id == id
                })
            })
            .is_some()
    })
}

/// Les lettres accentuées, la ligature, `€` et l'apostrophe partent en UTF-16, telles quelles.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_des_accents_en_unicode() {
    taper_dans_le_champ("accents", "Élève à l'œuvre : ça coûte 5 €");
}

/// Un caractère hors du plan de base part en sa paire de substitution, et arrive entier.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_hors_du_plan_de_base_envoie_la_paire() {
    taper_dans_le_champ("paire", "😀");
}

/// Un CRLF lu d'un fichier donne une seule Entrée : la zone de texte reçoit `a\nb`.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_crlf_donne_un_seul_entree() {
    let appli = lancer_appli("appli-essai", &["--focus", "zone"]);
    let profil = Profil::neuf("crlf");
    let fichier = profil.racine.join("crlf.txt");
    fs::write(&fichier, b"a\r\nb").expect("l'écriture du texte");
    let chemin = fichier.to_str().expect("un chemin en Unicode");
    let s = pilote(
        &profil,
        &["taper", "--fenetre", "appli-essai", "--fichier", chemin],
        None,
    );
    assert_eq!(s.code, 0, "{}", s.message());
    assert!(
        appli.attendre(r#""valeur":"a\nb""#, DELAI),
        "{:?}",
        appli.lignes()
    );
    thread::sleep(CALME);
    assert_eq!(appli.derniere_saisie("zone").as_deref(), Some("a\nb"));
}

/// `@`, tapé par la disposition française, part sur AltGr et la touche 0.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_arobase_par_altgr() {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf("arobase");
    exiger_la_disposition_francaise(&appli, &profil);
    let s = pilote(
        &profil,
        &[
            "taper",
            "--fenetre",
            "appli-essai",
            "--comme-au-clavier",
            "--texte",
            "@",
        ],
        None,
    );
    assert_eq!(s.code, 0, "{}", s.message());
    assert!(
        appli.attendre("\"valeur\":\"@\"", DELAI),
        "{:?}",
        appli.lignes()
    );
    thread::sleep(CALME);
    assert_eq!(appli.derniere_saisie("champ").as_deref(), Some("@"));
    assert!(
        appli
            .lire_evenements()
            .iter()
            .any(|e| e.genre == "touche" && e.code == "Digit0"),
        "{:?}",
        appli.lignes()
    );
}

/// `touche barre`, hors d'un champ de texte, frappe `/` par la disposition, et le raccourci de la
/// page porte le focus à `recherche`.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn touche_barre_declenche_le_raccourci() {
    let appli = lancer_appli("appli-essai", &[]);
    let profil = Profil::neuf("barre");
    exiger_la_disposition_francaise(&appli, &profil);
    let avant = appli.lire_evenements().len();
    let s = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "barre"],
        None,
    );
    assert_eq!(s.code, 0, "{}", s.message());
    assert!(
        focus_apres_la_touche(&appli, avant, "/", "recherche"),
        "aucun focus de recherche après la touche « / » : {:?}",
        appli.lignes()
    );
}

/// Tab, depuis `depart`, porte le focus à `champ`, le suivant dans l'ordre de tabulation.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn touche_tab_deplace_le_focus() {
    let appli = lancer_appli("appli-essai", &[]);
    let profil = Profil::neuf("tab");
    let avant = appli.lire_evenements().len();
    let s = pilote(
        &profil,
        &["touche", "--fenetre", "appli-essai", "Tab"],
        None,
    );
    assert_eq!(s.code, 0, "{}", s.message());
    assert!(
        focus_apres_la_touche(&appli, avant, "Tab", "champ"),
        "aucun focus de champ après la touche Tab : {:?}",
        appli.lignes()
    );
}

/// `ñ`, absent du clavier, refuse en 1 : rien ne part, ni vers la page, ni vers la fenêtre.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn comme_au_clavier_refuse_un_caractere_absent() {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf("absent");
    let s = pilote(
        &profil,
        &[
            "taper",
            "--fenetre",
            "appli-essai",
            "--comme-au-clavier",
            "--texte",
            "ñ",
        ],
        None,
    );
    assert_eq!(
        (s.code, s.message()),
        (1, "caractère absent du clavier : « ñ »".to_string())
    );
    thread::sleep(CALME);
    assert!(
        !appli.lire_evenements().iter().any(|e| e.est_une_frappe()),
        "{:?}",
        appli.lignes()
    );
}

/// Verr. Maj allumée refuse `taper --comme-au-clavier` en 1 : rien ne part. La frappe de Verr. Maj
/// elle-même arrive à l'application, qui a le premier plan : seuls les événements d'après elle se
/// jugent, une fois le calme passé.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn comme_au_clavier_refuse_quand_verr_maj_est_allumee() {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf("verr-maj");
    let _verr_maj = VerrMaj::allumer();
    thread::sleep(CALME);
    let avant = appli.lire_evenements().len();
    let s = pilote(
        &profil,
        &[
            "taper",
            "--fenetre",
            "appli-essai",
            "--comme-au-clavier",
            "--texte",
            "a",
        ],
        None,
    );
    assert_eq!(
        (s.code, s.message()),
        (1, "Verr. Maj est allumée".to_string())
    );
    thread::sleep(CALME);
    assert!(
        !appli
            .lire_evenements()
            .iter()
            .skip(avant)
            .any(|e| e.est_une_frappe()),
        "{:?}",
        appli.lignes()
    );
}

/// Depuis PowerShell 5.1 : un fichier qu'écrit `Out-File`, en UTF-16 à BOM et fini par le CRLF
/// qu'il ajoute, se tape sans ce saut final : aucune Entrée ne suit la saisie. Puis, la console
/// réglée en UTF-8, l'erreur de `pilote` se lit telle quelle. Chaque échec du script sort sur son
/// propre code : 99 pour un autre PowerShell, le code de `pilote` pour `taper`, 97 et 98 pour le
/// code et le message d'`activer`.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_depuis_powershell_5_1_un_fichier_utf16() {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf("powershell");
    let script = [
        "if ($PSVersionTable.PSVersion.Major -ne 5) { exit 99 }",
        "$f = Join-Path $env:TEMP 'ete.txt'",
        "'été' | Out-File -FilePath $f",
        "& pilote taper --fenetre appli-essai --fichier $f",
        "if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }",
        "[Console]::OutputEncoding = [Text.Encoding]::UTF8",
        "$e = (& pilote activer --fenetre absent 2>&1 | ForEach-Object { \"$_\" }) -join ''",
        "if ($LASTEXITCODE -ne 2) { exit 97 }",
        "if ($e -ne 'aucune fenêtre de « absent »') { exit 98 }",
        "exit 0",
    ]
    .join("; ");
    let s = shell(Shell::Powershell51, &script, &profil);
    assert_eq!(
        s.code,
        0,
        "sortie : {:?} ; erreur : {}",
        s.lignes(),
        s.message()
    );
    assert!(
        appli.attendre("\"valeur\":\"été\"", DELAI),
        "{:?}",
        appli.lignes()
    );
    thread::sleep(CALME);
    assert_eq!(appli.derniere_saisie("champ").as_deref(), Some("été"));
    let evenements = appli.lire_evenements();
    let saisie = rang(&evenements, 0, |e| {
        e.genre == "saisie" && e.id == "champ" && e.valeur == "été"
    })
    .unwrap_or_else(|| panic!("aucune saisie « été » de champ : {evenements:?}"));
    assert!(
        rang(&evenements, saisie + 1, |e| {
            (e.genre == "touche" || e.genre == "touche-fenetre") && e.key == "Enter"
        })
        .is_none(),
        "une Entrée suit la saisie « été » : {evenements:?}"
    );
}

/// Depuis Git Bash : un texte sur l'entrée standard, Maj+Tab, puis `barre`, qui évite un argument
/// commençant par `/`, que Git Bash réécrirait ; l'erreur d'`activer`, que Git Bash écrit dans un
/// fichier, est en UTF-8 sans BOM. Les accolades gardent la chaîne de `&&` : le code 2 attendu
/// d'`activer` y devient 0.
#[ignore = "il pilote le bureau : la CI le lance"]
#[test]
fn taper_depuis_git_bash_l_entree_standard() {
    let appli = lancer_appli("appli-essai", &["--focus", "champ"]);
    let profil = Profil::neuf("git-bash");
    exiger_la_disposition_francaise(&appli, &profil);
    let erreur = profil.racine.join("activer.txt");
    let f = erreur
        .to_str()
        .expect("un chemin en Unicode")
        .replace('\\', "/");
    let script = format!(
        "case $(uname -s) in MINGW*) ;; *) exit 99 ;; esac && printf 'été' | pilote taper \
         --fenetre appli-essai - && pilote touche --fenetre appli-essai Maj+Tab && pilote touche \
         --fenetre appli-essai barre && {{ pilote activer --fenetre absent 2>'{f}'; test $? -eq 2; \
         }}"
    );
    let avant = appli.lire_evenements().len();
    let s = shell(Shell::GitBash, &script, &profil);
    assert_eq!(
        s.code,
        0,
        "sortie : {:?} ; erreur : {}",
        s.lignes(),
        s.message()
    );
    assert!(
        appli.attendre("\"genre\":\"focus\",\"id\":\"recherche\"", DELAI),
        "{:?}",
        appli.lignes()
    );
    let evenements = appli.lire_evenements();
    let saisie = rang(&evenements, avant, |e| {
        e.genre == "saisie" && e.id == "champ" && e.valeur == "été"
    })
    .unwrap_or_else(|| panic!("aucune saisie « été » de champ : {evenements:?}"));
    let depart = rang(&evenements, saisie + 1, |e| {
        e.genre == "focus" && e.id == "depart"
    })
    .unwrap_or_else(|| panic!("aucun focus de depart après la saisie : {evenements:?}"));
    assert!(
        rang(&evenements, depart + 1, |e| e.genre == "focus"
            && e.id == "recherche")
        .is_some(),
        "aucun focus de recherche après celui de depart : {evenements:?}"
    );
    assert!(
        rang(&evenements, depart + 1, |e| e.genre == "touche"
            && e.key == "/")
        .is_some(),
        "aucune touche « / » après le focus de depart : {evenements:?}"
    );
    assert_eq!(
        fs::read(&erreur).expect("le fichier de l'erreur"),
        "aucune fenêtre de « absent »\n".as_bytes()
    );
}
