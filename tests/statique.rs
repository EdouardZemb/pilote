//! Le contrôle statique des sources (ADR-0003). Hors de `src/systeme.rs`, aucune source de `src/`
//! ne lit l'horloge ni l'environnement, ne devine un chemin ni ne tire le hasard ; le domaine,
//! `src/domaine/`, ne nomme de plus ni le système de fichiers, ni l'environnement, ni les
//! processus, ni la crate `windows` ; et aucune source, sauf celle-ci, ne nomme `SetValue`. Le test
//! lit sur le disque chaque `.rs` de `src/`, `tests/` et `essai/appli/src/`. Les règles « dans
//! `src/` » visent les chemins qui commencent par `src/`, jamais `essai/appli/src/` ni `tests/`.
//!
//! Les essais de bout en bout ne tournent jamais sur le poste (`CLAUDE.md` : la CI seule les
//! lance) : chaque `#[test]` des fichiers d'essai, `tests/essais_*.rs`, porte sur la ligne d'avant
//! l'attribut `IGNORE_DES_ESSAIS` ; chaque autre `#[test]` lu, hors du domaine et de ce fichier, un
//! `#[ignore = …]`, l'application d'essai comprise : seuls les tests du domaine et du contrôle
//! tournent par un simple `cargo test`, et un test qui toucherait Windows ne part qu'en CI ; un
//! `#[test]` qui n'est pas au début de sa ligne, blancs ôtés, est une faute hors de ce fichier,
//! puisque l'attribut exigé se lit sur la ligne d'avant ; aucun fichier de `tests/`, hors du module
//! commun des essais et de ce fichier, ne nomme un mot de lancement, de Rust (`Command::new`) comme
//! de Win32 (`CreateProcess`, `ShellExecute`) ; la couche Windows, `src/bureau/`, n'a aucun test ;
//! et seuls elle, `src/lib.rs` et ce fichier nomment le poste réel, `BureauWindows`.
//!
//! Aucune source de `src/`, `src/systeme.rs` compris, ne nomme le jeton restreint
//! (`CreateRestrictedToken`, `CreateProcessAsUser`, `SetTokenInformation`), qui vit dans les
//! essais, ni la propriété de la valeur, ni les motifs LegacyIAccessible et Text, qui la portent
//! aussi : une valeur ne passe jamais par le cache de l'automatisation (ADR-0005, « Les valeurs
//! secrètes ne sortent jamais »). Et `ToUnicodeEx(` ne s'écrit que dans `src/bureau/clavier.rs`,
//! avec `OCTET_BAS` sur la ligne de l'appel ou sur l'une des trois suivantes : seul l'octet bas du
//! balayage va à `ToUnicodeEx` (ADR-0005, « Ce qui précise ou change le plan de la tranche » ; LE-2
//! du registre).
//!
//! Le contrôle est une recherche dans le texte, pas une analyse de Rust. Il ferme les formes
//! courantes : les chemins complets (`std::fs::read`), les imports groupés (`use std::{fs, io};`)
//! et les préfixes nus qu'ils laissent employer (`fs::read`). Il ne ferme ni les alias (`use std
//! as s;`, puis `s::fs`), ni les blancs dans un chemin (`std :: fs`) : la relecture du code du
//! domaine les ferme.

use std::fs;
use std::path::{Path, PathBuf};

/// `SetValue`, refusé partout sauf ici : un champ se remplit en tapant (ADR-0005, « Aucun
/// `SetValue` »).
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

/// Les préfixes refusés de plus dans le domaine : les chemins complets, l'import groupé (`use
/// std::{fs, io};`), qui ne nomme aucun chemin complet, et les préfixes nus qu'il laisse employer
/// (`fs::read_to_string`). Le domaine importe donc de `std` un chemin par ligne (`use std::fmt;`).
const MOTS_DU_DOMAINE: &[&str] = &[
    "std::fs",
    "std::env",
    "std::process",
    "windows::",
    "std::{",
    "fs::",
    "process::",
    "env::",
];

/// Les mots de lancement, de Rust comme de Win32 : aucun fichier de `tests/` ne les nomme, sauf le
/// module commun des essais et ce fichier. Des préfixes bornés à gauche seulement : `CARGO_BIN_EXE`
/// voit `CARGO_BIN_EXE_pilote`, `CreateProcess` voit `CreateProcessW` et `CreateProcessAsUserW`,
/// `ShellExecute` voit `ShellExecuteExW`.
const MOTS_DE_LANCEMENT: &[&str] = &[
    "CARGO_BIN_EXE",
    "Command::new",
    "std::process",
    "spawn",
    "CreateProcess",
    "ShellExecute",
];

/// Refusés dans `src/`, préfixes bornés à gauche : le jeton restreint vit dans les essais
/// (ADR-0005, « Le niveau d'intégrité se juge par comparaison ») ; la valeur ne passe jamais par le
/// cache, ni par les motifs LegacyIAccessible et Text, qui la portent aussi. Une tranche qui aurait
/// besoin du motif Text ne lève la règle qu'avec un marquage relu juste avant (`CurrentIsPassword`),
/// jugé dans le domaine et testé.
const MOTS_DU_JETON_ET_DES_VALEURS: &[&str] = &[
    "CreateRestrictedToken",
    "CreateProcessAsUser",
    "SetTokenInformation",
    "UIA_ValueValuePropertyId",
    "UIA_LegacyIAccessibleValuePropertyId",
    "UIA_LegacyIAccessiblePatternId",
    "IUIAutomationLegacyIAccessiblePattern",
    "UIA_TextPatternId",
    "UIA_TextPattern2Id",
    "UIA_TextEditPatternId",
    "UIA_TextChildPatternId",
    "IUIAutomationTextPattern",
    "IUIAutomationTextEditPattern",
    "IUIAutomationTextChildPattern",
    "IUIAutomationTextRange",
];

/// Le seul fichier qui appelle `ToUnicodeEx`.
const CLAVIER: &str = "src/bureau/clavier.rs";

/// La marque de l'octet bas du balayage, qu'un appel de `ToUnicodeEx` porte sur sa ligne ou sur
/// l'une des `SUIVANTES`.
const OCTET_BAS: &str = "OCTET_BAS";

/// L'appel de `ToUnicodeEx`, parenthèse comprise : un `use` ne l'appelle pas.
const APPEL_TO_UNICODE_EX: &str = "ToUnicodeEx(";

/// Les lignes qui suivent un appel de `ToUnicodeEx` et peuvent porter `OCTET_BAS`, celle de l'appel
/// en plus : rustfmt range un appel long un argument par ligne, et le balayage en est le deuxième.
const SUIVANTES: usize = 3;

/// Le seul fichier de `src/` qui lit l'horloge et l'environnement, et résout les chemins.
const SYSTEME: &str = "src/systeme.rs";

/// Le domaine, pur.
const DOMAINE: &str = "src/domaine/";

/// Ce fichier, le seul qui nomme `SetValue`.
const STATIQUE: &str = "tests/statique.rs";

/// Le module commun des essais, le seul fichier de `tests/`, avec celui-ci, qui lance un processus.
const COMMUN: &str = "tests/commun/mod.rs";

/// La couche Windows, qui n'a aucun test.
const BUREAU: &str = "src/bureau/";

/// Le seul fichier hors de la couche Windows qui crée le poste réel.
const LIB: &str = "src/lib.rs";

/// Le poste réel, que seuls la couche Windows, `LIB` et ce fichier nomment.
const BUREAU_REEL: &str = "BureauWindows";

/// L'attribut que porte, sur la ligne d'avant, chaque `#[test]` d'un fichier d'essai.
const IGNORE_DES_ESSAIS: &str = "#[ignore = \"il pilote le bureau : la CI le lance\"]";

/// Le début de l'attribut que porte, sur la ligne d'avant, chaque autre `#[test]` lu, hors du
/// domaine et de ce fichier.
const IGNORE: &str = "#[ignore = ";

/// Les racines lues, depuis la racine du dépôt.
const RACINES: &[&str] = &["src", "tests", "essai/appli/src"];

/// Les mots que `chemin` ne porte jamais, chacun avec sa sorte (vrai pour un mot entier, faux pour
/// un préfixe) : dans `src/`, hors de `SYSTEME`, tous les mots entiers et les préfixes sauf
/// `SetValue` ; dans tout `src/`, `SYSTEME` compris, les mots du jeton et des valeurs ; dans le
/// domaine, de plus `MOTS_DU_DOMAINE` ; partout sauf ici, `SetValue` ; dans `tests/`, hors de
/// `COMMUN` et d'ici, les mots de lancement ; hors de `BUREAU`, de `LIB` et d'ici, le poste réel.
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
    if chemin.starts_with("src/") {
        mots.extend(MOTS_DU_JETON_ET_DES_VALEURS.iter().map(|mot| (*mot, false)));
    }
    if chemin.starts_with(DOMAINE) {
        mots.extend(MOTS_DU_DOMAINE.iter().map(|mot| (*mot, false)));
    }
    if chemin != STATIQUE {
        mots.push((SET_VALUE, true));
    }
    if chemin.starts_with("tests/") && chemin != COMMUN && chemin != STATIQUE {
        mots.extend(MOTS_DE_LANCEMENT.iter().map(|mot| (*mot, false)));
    }
    if !chemin.starts_with(BUREAU) && chemin != LIB && chemin != STATIQUE {
        mots.push((BUREAU_REEL, false));
    }
    mots
}

/// Vrai si `chemin` est un fichier d'essai, `tests/essais_*.rs`.
fn est_un_essai(chemin: &str) -> bool {
    chemin
        .strip_prefix("tests/essais_")
        .is_some_and(|reste| reste.ends_with(".rs") && !reste.contains('/'))
}

/// L'attribut qu'un `#[test]` porte sur la ligne d'avant.
enum Attribut {
    /// `IGNORE_DES_ESSAIS`, exactement : un fichier d'essai.
    DesEssais,
    /// Un `#[ignore = …]`, de toute raison : tout le reste, hors du domaine et d'ici.
    Ignore,
}

/// L'attribut qu'exige chaque `#[test]` de `chemin` : aucun dans le domaine et dans ce fichier ;
/// celui des essais dans un fichier d'essai ; un `#[ignore = …]` partout ailleurs, le reste de
/// `src/` et de `tests/` comme l'application d'essai.
fn attribut_exige(chemin: &str) -> Option<Attribut> {
    if chemin.starts_with(DOMAINE) || chemin == STATIQUE {
        None
    } else if est_un_essai(chemin) {
        Some(Attribut::DesEssais)
    } else {
        Some(Attribut::Ignore)
    }
}

/// Les fautes du `#[test]` de la ligne `numero` de `chemin`, `precedente` étant la ligne d'avant :
/// l'attribut exigé qui manque, puis, sous la couche Windows, le test lui-même.
fn fautes_du_test(chemin: &str, numero: usize, precedente: Option<&str>) -> Vec<String> {
    let mut fautes = Vec::new();
    let avant = precedente.map(str::trim).unwrap_or_default();
    match attribut_exige(chemin) {
        Some(Attribut::DesEssais) if avant != IGNORE_DES_ESSAIS => {
            fautes.push(format!(
                "{chemin}:{numero} : #[test] sans {IGNORE_DES_ESSAIS}"
            ));
        }
        Some(Attribut::Ignore) if !avant.starts_with(IGNORE) => {
            fautes.push(format!("{chemin}:{numero} : #[test] sans #[ignore]"));
        }
        _ => {}
    }
    if chemin.starts_with(BUREAU) {
        fautes.push(format!(
            "{chemin}:{numero} : #[test] dans la couche Windows"
        ));
    }
    fautes
}

/// La faute de la ligne `numero` de `chemin` qui porte `#[test]` ailleurs qu'à son début, blancs
/// ôtés, hors de ce fichier, qui le cite dans ses chaînes : les attributs exigés se jugent sur la
/// ligne d'avant un `#[test]` en tête de ligne, et un test écrit après un autre attribut, sur la
/// même ligne, leur échapperait.
fn fautes_de_la_ligne_de_test(chemin: &str, numero: usize, ligne: &str) -> Option<String> {
    (chemin != STATIQUE && ligne.contains("#[test]") && !ligne.trim_start().starts_with("#[test]"))
        .then(|| format!("{chemin}:{numero} : #[test] hors de la tête de ligne"))
}

/// Les fautes des appels de `ToUnicodeEx` des lignes `lignes` de `chemin`, hors de ce fichier :
/// chaque ligne qui porte `APPEL_TO_UNICODE_EX` est une faute hors de `CLAVIER` ; dans `CLAVIER`,
/// si ni elle ni l'une de ses `SUIVANTES` ne porte `OCTET_BAS`. Une ligne de commentaire ne compte
/// ni pour l'appel ni pour l'octet bas.
fn fautes_de_to_unicode_ex(chemin: &str, lignes: &[&str]) -> Vec<String> {
    if chemin == STATIQUE {
        return Vec::new();
    }
    let mut fautes = Vec::new();
    for (indice, ligne) in lignes.iter().enumerate() {
        if est_un_commentaire(ligne) || !porte(ligne, APPEL_TO_UNICODE_EX, false) {
            continue;
        }
        if chemin != CLAVIER {
            fautes.push(format!(
                "{chemin}:{} : ToUnicodeEx hors de {CLAVIER}",
                indice + 1
            ));
        } else if !lignes[indice..]
            .iter()
            .take(1 + SUIVANTES)
            .any(|l| !est_un_commentaire(l) && porte(l, OCTET_BAS, true))
        {
            fautes.push(format!(
                "{chemin}:{} : ToUnicodeEx sans l'octet bas du balayage",
                indice + 1
            ));
        }
    }
    fautes
}

/// Vrai si `ligne` est une ligne de commentaire ou de documentation : son début, blancs ôtés, est
/// `//`.
fn est_un_commentaire(ligne: &str) -> bool {
    ligne.trim_start().starts_with("//")
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
/// compte pas, mais garde son numéro. Chaque faute d'un mot nomme le fichier, la ligne et le mot,
/// une fois par ligne et par mot, dans l'ordre de `mots_refuses` : « <chemin>:<ligne> contient
/// « <mot> » ». Une ligne dont le début, blancs ôtés, est `#[test]` porte ensuite les fautes de
/// `fautes_du_test`, jugées sur la ligne d'avant, telle quelle ; une ligne qui porte `#[test]`
/// ailleurs, celle de `fautes_de_la_ligne_de_test`. Après les lignes d'un fichier viennent les
/// fautes de ses appels de `ToUnicodeEx`, celles de `fautes_de_to_unicode_ex`.
fn controler(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    for (chemin, texte) in sources {
        let mots = mots_refuses(chemin);
        let lignes: Vec<&str> = texte.lines().collect();
        for (indice, ligne) in lignes.iter().enumerate() {
            if est_un_commentaire(ligne) {
                continue;
            }
            for (mot, entier) in &mots {
                if porte(ligne, mot, *entier) {
                    fautes.push(format!("{chemin}:{} contient « {mot} »", indice + 1));
                }
            }
            if ligne.trim_start().starts_with("#[test]") {
                let precedente = indice.checked_sub(1).map(|i| lignes[i]);
                fautes.extend(fautes_du_test(chemin, indice + 1, precedente));
            }
            fautes.extend(fautes_de_la_ligne_de_test(chemin, indice + 1, ligne));
        }
        fautes.extend(fautes_de_to_unicode_ex(chemin, &lignes));
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

/// Le parcours descend dans les sous-dossiers : sans lui, aucune règle du domaine ne viserait un
/// fichier réel. Les chemins exacts éprouvent aussi le `/` entre plusieurs segments.
#[test]
fn descend_dans_les_sous_dossiers() {
    let s = lire_les_sources();
    for f in ["src/domaine/mod.rs", "src/bureau/mod.rs"] {
        assert!(s.iter().any(|(c, _)| c == f), "{f}");
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

/// Un import groupé ne nomme pas `std::fs` en entier, et ce qu'il importe s'emploie ensuite par
/// son préfixe nu : le domaine refuse l'un et l'autre, chaque préfixe nu à sa ligne.
#[test]
fn refuse_un_import_groupe_dans_le_domaine() {
    refuse(
        "src/domaine/texte.rs",
        "use std::{fs, io};",
        "src/domaine/texte.rs:1 contient « std::{ »",
    );
    refuse(
        "src/domaine/texte.rs",
        "let t = fs::read_to_string(p);",
        "src/domaine/texte.rs:1 contient « fs:: »",
    );
    refuse(
        "src/domaine/texte.rs",
        "process::exit(1);",
        "src/domaine/texte.rs:1 contient « process:: »",
    );
    refuse(
        "src/domaine/texte.rs",
        "let a = env::args();",
        "src/domaine/texte.rs:1 contient « env:: »",
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

/// `SetValue`, refusé partout, ne se compte pas une seconde fois avec les mots de `src/`.
#[test]
fn set_value_dans_src_une_seule_faute() {
    refuse(
        "src/bureau/mod.rs",
        "x.SetValue(v)",
        "src/bureau/mod.rs:1 contient « SetValue »",
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

/// Une ligne de commentaire ne coupe pas la lecture du fichier, et compte dans les numéros.
#[test]
fn lit_et_numerote_apres_un_commentaire() {
    refuse(
        "src/domaine/gestes.rs",
        "//! doc\nlet t = Instant::now();",
        "src/domaine/gestes.rs:2 contient « Instant »",
    );
}

/// Une faute par ligne et par mot, dans l'ordre de `mots_refuses` : deux mots, deux fautes ; un
/// mot répété, une seule.
#[test]
fn une_faute_par_ligne_et_par_mot() {
    assert_eq!(
        controler(&[source(
            "src/domaine/gestes.rs",
            "let t = Instant::now(); sleep(d);"
        )]),
        vec![
            "src/domaine/gestes.rs:1 contient « Instant »".to_string(),
            "src/domaine/gestes.rs:1 contient « sleep »".to_string(),
        ]
    );
    refuse(
        "src/domaine/gestes.rs",
        "let t = Instant::now(); let u = Instant::now();",
        "src/domaine/gestes.rs:1 contient « Instant »",
    );
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

#[test]
fn refuse_un_essai_sans_ignore() {
    refuse(
        "tests/essais_clavier.rs",
        "#[test]\nfn a() {}",
        "tests/essais_clavier.rs:1 : #[test] sans #[ignore = \"il pilote le bureau : la CI le lance\"]",
    );
}

#[test]
fn admet_un_essai_ignore() {
    admet(
        "tests/essais_clavier.rs",
        "#[ignore = \"il pilote le bureau : la CI le lance\"]\n#[test]\nfn a() {}",
    );
}

#[test]
fn refuse_un_test_sans_ignore_hors_du_domaine() {
    refuse(
        "src/systeme.rs",
        "#[test]\nfn a() {}",
        "src/systeme.rs:1 : #[test] sans #[ignore]",
    );
}

#[test]
fn admet_un_test_ignore_hors_du_domaine() {
    admet("src/systeme.rs", "#[ignore = \"x\"]\n#[test]\nfn a() {}");
}

/// Un `#[test]` indenté, comme dans un `mod tests`, se voit, et son `#[ignore = …]` indenté aussi.
#[test]
fn refuse_un_test_indente_sans_ignore() {
    refuse(
        "src/systeme.rs",
        "mod tests {\n    #[test]\n    fn a() {}\n}",
        "src/systeme.rs:2 : #[test] sans #[ignore]",
    );
}

#[test]
fn admet_un_test_indente_ignore() {
    admet(
        "src/systeme.rs",
        "mod t {\n    #[ignore = \"x\"]\n    #[test]\n    fn a() {}\n}",
    );
}

/// Le reste de `tests/`, hors des fichiers d'essai et d'ici, est tenu au `#[ignore = …]` : le module
/// commun, un autre fichier, et un fichier sous un dossier `essais_*`, qui n'est pas un fichier
/// d'essai.
#[test]
fn refuse_un_test_sans_ignore_dans_tests() {
    refuse(
        "tests/commun/mod.rs",
        "#[test]\nfn a() {}",
        "tests/commun/mod.rs:1 : #[test] sans #[ignore]",
    );
    refuse(
        "tests/essais_x/a.rs",
        "#[test]\nfn a() {}",
        "tests/essais_x/a.rs:1 : #[test] sans #[ignore]",
    );
}

#[test]
fn admet_un_test_ignore_dans_tests() {
    admet("tests/autre.rs", "#[ignore = \"x\"]\n#[test]\nfn a() {}");
}

/// Le domaine et ce fichier ne sont pas tenus à l'attribut : leurs tests tournent partout.
#[test]
fn admet_un_test_du_domaine_et_du_controle() {
    admet("src/domaine/gestes.rs", "#[test]\nfn a() {}");
    admet("tests/statique.rs", "#[test]\nfn a() {}");
}

/// L'application d'essai n'est ni le domaine, ni ce fichier, ni un fichier d'essai : chacun de ses
/// tests porte un `#[ignore = …]`, et ne tourne qu'en CI.
#[test]
fn refuse_un_test_de_l_application_d_essai_sans_ignore() {
    refuse(
        "essai/appli/src/main.rs",
        "#[test]\nfn a() {}",
        "essai/appli/src/main.rs:1 : #[test] sans #[ignore]",
    );
}

#[test]
fn admet_un_test_ignore_de_l_application_d_essai() {
    admet(
        "essai/appli/src/main.rs",
        "#[ignore = \"x\"]\n#[test]\nfn a() {}",
    );
}

#[test]
fn chaque_mot_de_lancement_a_sa_plantation() {
    for mot in MOTS_DE_LANCEMENT {
        refuse(
            "tests/autre.rs",
            &format!("let x = {mot};"),
            &format!("tests/autre.rs:1 contient « {mot} »"),
        );
    }
    refuse(
        "tests/essais_gardes.rs",
        "env!(\"CARGO_BIN_EXE_pilote\")",
        "tests/essais_gardes.rs:1 contient « CARGO_BIN_EXE »",
    );
}

#[test]
fn admet_un_lancement_dans_le_module_commun() {
    admet("tests/commun/mod.rs", "Command::new(\"x\")");
}

#[test]
fn refuse_un_test_dans_la_couche_windows() {
    refuse(
        "src/bureau/clavier.rs",
        "#[ignore = \"x\"]\n#[test]\nfn a() {}",
        "src/bureau/clavier.rs:2 : #[test] dans la couche Windows",
    );
    refuse(
        "src/bureau/clavier.rs",
        "mod t {\n    #[ignore = \"x\"]\n    #[test]\n    fn a() {}\n}",
        "src/bureau/clavier.rs:3 : #[test] dans la couche Windows",
    );
}

#[test]
fn refuse_le_bureau_reel_hors_de_sa_place() {
    refuse(
        "src/domaine/gestes.rs",
        "BureauWindows::new()",
        "src/domaine/gestes.rs:1 contient « BureauWindows »",
    );
}

#[test]
fn refuse_un_lancement_win32_dans_un_essai() {
    refuse(
        "tests/essais_x.rs",
        "let _ = CreateProcessW(None, …);",
        "tests/essais_x.rs:1 contient « CreateProcess »",
    );
    refuse(
        "tests/essais_x.rs",
        "ShellExecuteExW(&mut info);",
        "tests/essais_x.rs:1 contient « ShellExecute »",
    );
    admet("tests/commun/mod.rs", "CreateProcessAsUserW(…);");
}

#[test]
fn refuse_le_jeton_et_les_chemins_de_la_valeur_dans_src() {
    for mot in MOTS_DU_JETON_ET_DES_VALEURS {
        refuse(
            "src/bureau/x.rs",
            &format!("let _ = {mot}W(…);"),
            &format!("src/bureau/x.rs:1 contient « {mot} »"),
        );
    }
    admet(
        "tests/commun/mod.rs",
        "CreateRestrictedToken(…); SetTokenInformation(…);",
    );
}

/// Chacun des quinze mots a sa plantation, et deux variantes se voient par leur préfixe. La règle
/// vaut pour tout `src/`, `src/systeme.rs` compris, qui échappe aux mots de l'horloge et de
/// l'environnement.
#[test]
fn chaque_mot_du_jeton_et_des_valeurs_a_sa_plantation() {
    for mot in MOTS_DU_JETON_ET_DES_VALEURS {
        refuse(
            "src/bureau/mod.rs",
            &format!("let x = {mot};"),
            &format!("src/bureau/mod.rs:1 contient « {mot} »"),
        );
    }
    for (variante, mot) in [
        ("IUIAutomationTextPattern2", "IUIAutomationTextPattern"),
        ("IUIAutomationTextRangeArray", "IUIAutomationTextRange"),
    ] {
        refuse(
            "src/bureau/mod.rs",
            &format!("let x = {variante};"),
            &format!("src/bureau/mod.rs:1 contient « {mot} »"),
        );
    }
    refuse(
        "src/systeme.rs",
        "let x = SetTokenInformation;",
        "src/systeme.rs:1 contient « SetTokenInformation »",
    );
}

#[test]
fn refuse_un_test_hors_de_la_tete_de_ligne() {
    refuse(
        "src/domaine/x.rs",
        "#[cfg(test)] #[test]\nfn a() {}",
        "src/domaine/x.rs:1 : #[test] hors de la tête de ligne",
    );
    refuse(
        "tests/essais_x.rs",
        "#[ignore = \"il pilote le bureau : la CI le lance\"] #[test]\nfn a() {}",
        "tests/essais_x.rs:1 : #[test] hors de la tête de ligne",
    );
    admet("src/domaine/x.rs", "// un #[test] cité dans un commentaire");
    admet("tests/statique.rs", "let t = \"#[test]\";");
}

#[test]
fn refuse_to_unicode_ex_hors_du_clavier() {
    refuse(
        "src/bureau/fenetres.rs",
        "let n = unsafe { ToUnicodeEx(vk, b & OCTET_BAS, &e, &mut t, 0, None) };",
        "src/bureau/fenetres.rs:1 : ToUnicodeEx hors de src/bureau/clavier.rs",
    );
}

#[test]
fn refuse_to_unicode_ex_sans_l_octet_bas() {
    refuse(
        "src/bureau/clavier.rs",
        "ToUnicodeEx(\n    u32::from(vk),\n    balayage,\n    &clavier,\n)",
        "src/bureau/clavier.rs:1 : ToUnicodeEx sans l'octet bas du balayage",
    );
    admet(
        "src/bureau/clavier.rs",
        "ToUnicodeEx(\n    u32::from(vk),\n    balayage & OCTET_BAS,\n)",
    );
    admet(
        "src/bureau/clavier.rs",
        "use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;",
    );
}

/// L'octet bas se cherche sur la ligne de l'appel ou sur l'une des trois suivantes, jamais plus
/// loin ; une ligne de commentaire ne compte ni pour l'appel ni pour l'octet bas.
#[test]
fn to_unicode_ex_compte_sa_ligne_et_ses_trois_suivantes() {
    admet(
        "src/bureau/clavier.rs",
        "ToUnicodeEx(vk, b & OCTET_BAS, &e)",
    );
    admet(
        "src/bureau/clavier.rs",
        "ToUnicodeEx(\n    a,\n    b,\n    c & OCTET_BAS,\n)",
    );
    refuse(
        "src/bureau/clavier.rs",
        "ToUnicodeEx(\n    a,\n    b,\n    c,\n    d & OCTET_BAS,\n)",
        "src/bureau/clavier.rs:1 : ToUnicodeEx sans l'octet bas du balayage",
    );
    refuse(
        "src/bureau/clavier.rs",
        "ToUnicodeEx(\n    // OCTET_BAS\n    balayage,\n)",
        "src/bureau/clavier.rs:1 : ToUnicodeEx sans l'octet bas du balayage",
    );
    admet(
        "src/domaine/texte.rs",
        "/// `ToUnicodeEx(vk, balayage)` produit…",
    );
}
