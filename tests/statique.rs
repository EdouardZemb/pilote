//! Le contrôle statique des sources (ADR-0003). Hors de `src/systeme.rs`, aucune source de `src/`
//! ne lit l'horloge ni l'environnement, ne devine un chemin ni ne tire le hasard ; le domaine,
//! `src/domaine/`, ne nomme de plus ni le système de fichiers, ni l'environnement, ni les
//! processus, ni la crate `windows` ; et aucune source, sauf celle-ci, ne nomme `SetValue`. Le test
//! lit sur le disque chaque `.rs` de `src/`, `tests/` et `essai/appli/src/`. Les règles « dans
//! `src/` » visent les chemins qui commencent par `src/`, jamais `essai/appli/src/` ni `tests/`.

use std::fs;
use std::path::{Path, PathBuf};

/// `SetValue`, refusé partout sauf ici : un champ se remplit en tapant (ADR-0002).
const SET_VALUE: &str = "SetValue";

/// Les mots entiers : un mot ne compte que s'il ne colle à aucune lettre, aucun chiffre ni aucun
/// `_`, juste avant ni juste après lui. `Instant` ne compte pas dans `Instantane`, ni `rand` dans
/// `grand`. La borne ne se demande que du côté où le mot commence ou finit par une lettre, un
/// chiffre ou `_` : `"TEMP"` compte aussi dans `r"TEMP"`.
const MOTS_ENTIERS: &[&str] = &[
    "Instant",
    "SystemTime",
    "UNIX_EPOCH",
    "GetLocalTime",
    "GetTickCount64",
    "QueryPerformanceCounter",
    "sleep",
    "temp_dir",
    "home_dir",
    "LOCALAPPDATA",
    "APPDATA",
    "USERPROFILE",
    "AppData",
    "rand",
    SET_VALUE,
    "\"TEMP\"",
    "\"TMP\"",
];

/// Les préfixes : ils ne demandent la borne qu'avant eux, et voient ainsi leurs variantes
/// (`GetTempPathW`, `env::vars_os`).
const PREFIXES: &[&str] = &[
    "std::time",
    "env::var",
    "GetEnvironmentVariable",
    "GetEnvironmentStrings",
    "ExpandEnvironmentStrings",
    "GetTempPath",
    "SHGetKnownFolder",
    "SHGetFolderPath",
    "CSIDL_",
    "FOLDERID_",
    "dirs::",
];

/// Les préfixes refusés de plus dans le domaine.
const MOTS_DU_DOMAINE: &[&str] = &["std::fs", "std::env", "std::process", "windows::"];

/// Le seul fichier de `src/` qui lit l'horloge et l'environnement, et résout les chemins.
const SYSTEME: &str = "src/systeme.rs";

/// Le domaine, pur.
const DOMAINE: &str = "src/domaine/";

/// Ce fichier, le seul qui nomme `SetValue`.
const STATIQUE: &str = "tests/statique.rs";

/// Les racines lues, depuis la racine du dépôt.
const RACINES: &[&str] = &["src", "tests", "essai/appli/src"];

/// Les mots que `chemin` ne porte jamais, chacun avec sa sorte (vrai pour un mot entier, faux pour
/// un préfixe) : dans `src/`, hors de `SYSTEME`, tous les mots entiers et les préfixes sauf
/// `SetValue` ; dans le domaine, de plus `MOTS_DU_DOMAINE` ; partout sauf ici, `SetValue`.
fn mots_refuses(chemin: &str) -> Vec<(&'static str, bool)> {
    let mut mots = Vec::new();
    if chemin.starts_with("src/") && chemin != SYSTEME {
        mots.extend(
            MOTS_ENTIERS
                .iter()
                .filter(|mot| **mot != SET_VALUE)
                .map(|mot| (*mot, true)),
        );
        mots.extend(PREFIXES.iter().map(|mot| (*mot, false)));
    }
    if chemin.starts_with(DOMAINE) {
        mots.extend(MOTS_DU_DOMAINE.iter().map(|mot| (*mot, false)));
    }
    if chemin != STATIQUE {
        mots.push((SET_VALUE, true));
    }
    mots
}

/// Vrai si `c` colle à un nom : une lettre, un chiffre ou `_`.
fn colle(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Vrai si `ligne` porte `mot` à au moins une place, sans rien qui colle juste avant lui, ni, pour
/// un mot entier, juste après ; chaque place se lit, même quand deux places se chevauchent.
fn porte(ligne: &str, mot: &str, entier: bool) -> bool {
    let borne_avant = mot.chars().next().is_some_and(colle);
    let borne_apres = entier && mot.chars().next_back().is_some_and(colle);
    ligne.char_indices().any(|(debut, _)| {
        ligne[debut..].starts_with(mot)
            && !(borne_avant && ligne[..debut].chars().next_back().is_some_and(colle))
            && !(borne_apres && ligne[debut + mot.len()..].chars().next().is_some_and(colle))
    })
}

/// Les fautes de `sources`, des paires (chemin, texte) nommées depuis la racine du dépôt, les
/// chemins en `/`. Une ligne dont le début, blancs ôtés, est `//` (commentaire, documentation) ne
/// compte pas. Chaque faute nomme le fichier, la ligne et le mot, une fois par ligne :
/// « <chemin>:<ligne> contient « <mot> » ».
fn controler(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    for (chemin, texte) in sources {
        let mots = mots_refuses(chemin);
        for (indice, ligne) in texte.lines().enumerate() {
            if ligne.trim_start().starts_with("//") {
                continue;
            }
            for (mot, entier) in &mots {
                if porte(ligne, mot, *entier) {
                    fautes.push(format!("{chemin}:{} contient « {mot} »", indice + 1));
                }
            }
        }
    }
    fautes
}

fn lire(chemin: &Path) -> String {
    fs::read_to_string(chemin).unwrap_or_else(|e| panic!("lecture de {} : {e}", chemin.display()))
}

/// Tous les fichiers `*.rs` sous `dossier`, en descendant dans les sous-dossiers.
fn fichiers_rs(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    for entree in
        fs::read_dir(dossier).unwrap_or_else(|e| panic!("lecture de {} : {e}", dossier.display()))
    {
        let chemin = entree.expect("entrée").path();
        if chemin.is_dir() {
            fichiers_rs(&chemin, sortie);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            sortie.push(chemin);
        }
    }
}

/// `chemin`, relatif à `racine`, ses composants joints par `/`, sous Windows comme sous Linux.
fn relatif(racine: &Path, chemin: &Path) -> String {
    let reste = chemin
        .strip_prefix(racine)
        .expect("un chemin sous la racine");
    reste
        .components()
        .map(|composant| composant.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Les sources du dépôt, lues sur le disque : chaque `.rs` des racines, en descendant, nommé depuis
/// la racine du dépôt.
fn lire_les_sources() -> Vec<(String, String)> {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut fichiers = Vec::new();
    for dossier in RACINES {
        fichiers_rs(&racine.join(dossier), &mut fichiers);
    }
    fichiers.sort();
    fichiers
        .iter()
        .map(|f| (relatif(racine, f), lire(f)))
        .collect()
}

/// Une source inventée, nommée depuis la racine du dépôt.
fn source(chemin: &str, texte: &str) -> (String, String) {
    (chemin.to_string(), texte.to_string())
}

/// `controler`, sur la seule source plantée, rend exactement `faute`.
fn refuse(chemin: &str, texte: &str, faute: &str) {
    assert_eq!(controler(&[source(chemin, texte)]), vec![faute.to_string()]);
}

/// `controler`, sur la seule source plantée, ne rend aucune faute.
fn admet(chemin: &str, texte: &str) {
    assert_eq!(controler(&[source(chemin, texte)]), Vec::<String>::new());
}

#[test]
fn sources_respectent_les_regles() {
    assert_eq!(controler(&lire_les_sources()), Vec::<String>::new());
}

#[test]
fn racines_lues() {
    let s = lire_les_sources();
    for r in ["src/", "tests/", "essai/appli/src/"] {
        assert!(s.iter().any(|(c, _)| c.starts_with(r)), "{r}");
    }
}

#[test]
fn refuse_l_horloge_hors_de_systeme() {
    refuse(
        "src/domaine/gestes.rs",
        "let t = Instant::now();",
        "src/domaine/gestes.rs:1 contient « Instant »",
    );
}

#[test]
fn admet_l_horloge_dans_systeme() {
    admet("src/systeme.rs", "let t = std::time::Instant::now();");
}

#[test]
fn refuse_une_variante_par_son_prefixe() {
    refuse(
        "src/bureau/mod.rs",
        "GetTempPathW(&mut b)",
        "src/bureau/mod.rs:1 contient « GetTempPath »",
    );
    refuse(
        "src/bureau/mod.rs",
        "std::env::vars_os()",
        "src/bureau/mod.rs:1 contient « env::var »",
    );
}

#[test]
fn refuse_le_systeme_de_fichiers_dans_le_domaine() {
    refuse(
        "src/domaine/texte.rs",
        "use std::fs;",
        "src/domaine/texte.rs:1 contient « std::fs »",
    );
}

#[test]
fn refuse_set_value_partout() {
    refuse(
        "tests/essais_clavier.rs",
        "x.SetValue(v)",
        "tests/essais_clavier.rs:1 contient « SetValue »",
    );
}

#[test]
fn cherche_des_noms_bornes() {
    admet("src/domaine/bureau.rs", "struct Instantane;");
    admet("src/domaine/bureau.rs", "let grand = 1;");
}

#[test]
fn ignore_une_ligne_de_commentaire() {
    admet("src/domaine/bureau.rs", "// Instant::now() est refusé ici");
}

#[test]
fn chaque_mot_refuse_a_sa_plantation() {
    for mot in MOTS_ENTIERS.iter().chain(PREFIXES) {
        if *mot == "SetValue" {
            continue;
        }
        refuse(
            "src/bureau/mod.rs",
            &format!("let x = {mot};"),
            &format!("src/bureau/mod.rs:1 contient « {mot} »"),
        );
    }
}

#[test]
fn chaque_mot_du_domaine_a_sa_plantation() {
    for mot in MOTS_DU_DOMAINE {
        refuse(
            "src/domaine/gestes.rs",
            &format!("let x = {mot};"),
            &format!("src/domaine/gestes.rs:1 contient « {mot} »"),
        );
    }
}

#[test]
fn admet_le_sommeil_dans_l_application_d_essai() {
    admet("essai/appli/src/main.rs", "std::thread::sleep(d);");
}

#[test]
fn refuse_une_variable_en_chaine_brute() {
    refuse(
        "src/bureau/mod.rs",
        "let n = r\"TEMP\";",
        "src/bureau/mod.rs:1 contient « \"TEMP\" »",
    );
}

#[test]
fn la_borne_compte_les_chiffres_et_le_souligne() {
    admet("src/domaine/bureau.rs", "let x2rand = 1;");
    admet("src/domaine/bureau.rs", "let rand_x = 1;");
}
