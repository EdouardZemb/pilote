//! Le modèle d'un élément de l'automatisation de Windows, tel que la couche le lit et que le
//! domaine le juge : ses types, la table des types de contrôle, la ligne qu'il écrit (ADR-0006,
//! « La sortie »), l'écart des fenêtres d'un processus que la retenue n'admet pas (ADR-0005, « Ce
//! qu'on lit reste dans la fenêtre retenue »), le document vide que la lecture sonde (ADR-0006,
//! « L'attente »), et le classement des erreurs de l'automatisation en arrêts (ADR-0006, « Les codes
//! de sortie »).
//!
//! L'arbre d'une fenêtre arrive en une liste plate, la fenêtre en tête (indice 0, la racine), en
//! ordre de document : le parent d'un élément le précède. Le domaine ne nomme aucune interface de
//! l'automatisation : il ne reçoit que des `ElementLu` et des codes. Chaque fonction traite un
//! élément `hors` comme absent, sauf `ancetres` et `profondeur`, qui ne jugent rien.

use crate::domaine::sortie::Arret;
use crate::domaine::sortie::Ligne;
use crate::domaine::sortie::echapper;
use std::iter;

/// Un élément lu. Chaque `Option` d'un motif est `Some` si et seulement si l'élément offre ce motif :
/// `coche`, Toggle ; `choisi`, SelectionItem ; `deplie`, ExpandCollapse ; `lecture_seule`, Value (la
/// ligne ne porte `valeur` que pour lui) ; `defile`, Scroll ; et `invocable`, Invoke. `fenetre` :
/// l'élément est une fenêtre (un HWND natif non nul). `hors` : écarté par `borner`, jamais posé par
/// la couche.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ElementLu {
    pub parent: Option<usize>,
    pub genre: i32,
    pub nom: String,
    pub id: String,
    pub classe: String,
    pub raccourci: String,
    pub acces: String,
    pub annonce: i32,
    pub motdepasse: bool,
    pub actif: bool,
    pub pid: u32,
    pub fenetre: bool,
    pub hors: bool,
    pub coche: Option<i32>,
    pub deplie: Option<i32>,
    pub choisi: Option<bool>,
    pub lecture_seule: Option<bool>,
    pub defile: Option<Defile>,
    pub invocable: bool,
}

/// Les sens où un élément qui offre le motif Scroll défile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Defile {
    pub vertical: bool,
    pub horizontal: bool,
}

/// Une erreur de l'automatisation : son code, le `HRESULT` lu en `u32`, et son texte, que la couche
/// écrit `<appel> : 0x<code>`, suivi de `: <message>` si Windows en donne un non vide, ou
/// `<appel> : rien rendu` pour un `HRESULT(0)` inattendu.
#[derive(Clone, Debug, PartialEq)]
pub struct ErreurUia {
    pub code: u32,
    pub texte: String,
}

/// L'identifiant du premier type de contrôle de l'automatisation, `Button`.
pub const PREMIER_TYPE: i32 = 50_000;

/// Les 41 types de contrôle de l'automatisation, sans « ControlType », dans l'ordre de leurs
/// identifiants, de `PREMIER_TYPE` à 50040 (`AppBar`).
pub const TYPES: [&str; 41] = [
    "Button",
    "Calendar",
    "CheckBox",
    "ComboBox",
    "Edit",
    "Hyperlink",
    "Image",
    "ListItem",
    "List",
    "Menu",
    "MenuBar",
    "MenuItem",
    "ProgressBar",
    "RadioButton",
    "ScrollBar",
    "Slider",
    "Spinner",
    "StatusBar",
    "Tab",
    "TabItem",
    "Text",
    "ToolBar",
    "ToolTip",
    "Tree",
    "TreeItem",
    "Custom",
    "Group",
    "Thumb",
    "DataGrid",
    "DataItem",
    "Document",
    "SplitButton",
    "Window",
    "Pane",
    "Header",
    "HeaderItem",
    "Table",
    "TitleBar",
    "Separator",
    "SemanticZoom",
    "AppBar",
];

/// Le type `Group`.
pub const GROUPE: i32 = 50_026;
/// Le type `Document`.
pub const DOCUMENT: i32 = 50_030;
/// Le type `Window`.
pub const FENETRE: i32 = 50_032;
/// Le type `Pane`.
pub const VOLET: i32 = 50_033;
/// Le type `TitleBar`.
pub const BARRE_DE_TITRE: i32 = 50_037;

/// `UIA_E_ELEMENTNOTAVAILABLE` : l'élément n'existe plus.
pub const UIA_DISPARU: u32 = 0x8004_0201;
/// `RPC_E_DISCONNECTED` : le fournisseur de l'élément est mort.
pub const RPC_DECONNECTE: u32 = 0x8001_0108;
/// `CO_E_OBJNOTCONNECTED` : l'objet n'est plus relié à son serveur.
pub const OBJET_DECONNECTE: u32 = 0x8004_01FD;
/// `UIA_E_TIMEOUT` : le délai de l'automatisation est dépassé.
pub const UIA_DELAI: u32 = 0x8013_1505;
/// `UIA_E_ELEMENTNOTENABLED` : l'élément est inactif.
pub const UIA_INACTIF: u32 = 0x8004_0200;
/// `UIA_E_NOTSUPPORTED` : l'élément ne prend pas le geste.
pub const UIA_NON_PRIS: u32 = 0x8004_0204;
/// `UIA_E_INVALIDOPERATION` : le geste n'est pas permis dans l'état de l'élément.
pub const UIA_OPERATION_INVALIDE: u32 = 0x8013_1509;

/// Le nom du type `genre`, celui de `TYPES` ; « Inconnu » hors de la table.
pub fn nom_du_type(genre: i32) -> &'static str {
    (PREMIER_TYPE..)
        .zip(TYPES)
        .find(|&(g, _)| g == genre)
        .map_or("Inconnu", |(_, nom)| nom)
}

/// Le type de ce nom dans `TYPES`, sans égard à la casse ASCII ; `None` pour un nom inconnu.
pub fn type_de_nom(nom: &str) -> Option<i32> {
    (PREMIER_TYPE..)
        .zip(TYPES)
        .find(|(_, t)| t.eq_ignore_ascii_case(nom))
        .map(|(g, _)| g)
}

/// Les ancêtres de l'élément `i`, du parent à la racine.
pub fn ancetres(arbre: &[ElementLu], i: usize) -> Vec<usize> {
    iter::successors(arbre[i].parent, |&p| arbre[p].parent).collect()
}

/// La profondeur de l'élément `i` : 0 pour la racine, le nombre de ses ancêtres sinon.
pub fn profondeur(arbre: &[ElementLu], i: usize) -> usize {
    ancetres(arbre, i).len()
}

/// Les états de `e`, dans l'ordre d'ADR-0006 (« La sortie ») : `coche`, `decoche` ou `indetermine`
/// pour Toggle 1, 0 ou 2 ; `choisi` ; `inactif` ; `deplie` pour ExpandCollapse 1 ou 2, `replie` pour
/// 0 ; `lecture-seule`.
pub fn etats(e: &ElementLu) -> Vec<&'static str> {
    let mut sortie = Vec::new();
    match e.coche {
        Some(1) => sortie.push("coche"),
        Some(0) => sortie.push("decoche"),
        Some(2) => sortie.push("indetermine"),
        _ => {}
    }
    if e.choisi == Some(true) {
        sortie.push("choisi");
    }
    if !e.actif {
        sortie.push("inactif");
    }
    match e.deplie {
        Some(1 | 2) => sortie.push("deplie"),
        Some(0) => sortie.push("replie"),
        _ => {}
    }
    if e.lecture_seule == Some(true) {
        sortie.push("lecture-seule");
    }
    sortie
}

/// Le groupe qui contient l'élément `i` (ADR-0006, « Le sélecteur ») : le nom du plus proche ancêtre
/// nommé de type `Group`, sinon celui du plus proche ancêtre nommé ; ni la racine, ni un ancêtre de
/// type `Window` ou `Document`, ni un ancêtre écarté ne comptent. `None` si aucun ne compte.
pub fn dans_de(arbre: &[ElementLu], i: usize) -> Option<&str> {
    let nommes: Vec<&ElementLu> = ancetres(arbre, i)
        .into_iter()
        .map(|a| &arbre[a])
        .filter(|a| {
            a.parent.is_some()
                && !a.hors
                && !a.nom.is_empty()
                && a.genre != FENETRE
                && a.genre != DOCUMENT
        })
        .collect();
    nommes
        .iter()
        .find(|a| a.genre == GROUPE)
        .or(nommes.first())
        .map(|a| a.nom.as_str())
}

/// Pose `hors` sur chaque élément, hors de la racine, qui est une fenêtre et dont `admis` refuse le
/// processus, et sur tout son sous-arbre (ADR-0005, « Ce qu'on lit reste dans la fenêtre retenue »).
/// Un élément qui n'est pas une fenêtre ne se juge pas par son processus : il vit dans la fenêtre
/// qui le contient, et un élément de WebView2 porte celui de `msedgewebview2`, un descendant de P.
/// Le sous-arbre se suit par les parents, sans dépendre de l'ordre de la liste.
pub fn borner(arbre: &mut [ElementLu], admis: impl Fn(u32) -> bool) {
    let refusee = |k: usize| {
        let e = &arbre[k];
        e.parent.is_some() && e.fenetre && !admis(e.pid)
    };
    let hors: Vec<bool> = (0..arbre.len())
        .map(|i| iter::once(i).chain(ancetres(arbre, i)).any(refusee))
        .collect();
    for (e, h) in arbre.iter_mut().zip(hors) {
        if h {
            e.hors = true;
        }
    }
}

/// Vrai si l'arbre est vide (ADR-0006, « L'attente ») : le `Document` de la page n'a aucun enfant,
/// ou aucun élément n'est d'un type hors de `Window`, `Pane` et `TitleBar`, le sous-arbre d'une
/// `TitleBar` ne comptant pas. Le `Document` de la page est le premier, en ordre de document, qui
/// n'a aucun ancêtre `Window` autre que la racine : un dialogue que W possède passe avant la page.
/// Ce premier n'a aucun ancêtre `Document` : un tel ancêtre le précéderait en ordre de document, et
/// répondrait avant lui. Un `Document` vide sous celui de la page ne rend pas l'arbre vide. Un élément
/// écarté ne compte nulle part : ni comme `Document` de la page, ni comme son enfant, ni comme
/// élément qui compte.
pub fn document_vide(arbre: &[ElementLu]) -> bool {
    let page = (0..arbre.len()).find(|&k| {
        !arbre[k].hors
            && arbre[k].genre == DOCUMENT
            && !ancetres(arbre, k)
                .into_iter()
                .any(|a| arbre[a].genre == FENETRE && arbre[a].parent.is_some())
    });
    let page_sans_enfant =
        page.is_some_and(|d| !arbre.iter().any(|e| !e.hors && e.parent == Some(d)));
    let compte = |k: usize| {
        let e = &arbre[k];
        !e.hors
            && e.genre != FENETRE
            && e.genre != VOLET
            && e.genre != BARRE_DE_TITRE
            && !ancetres(arbre, k)
                .into_iter()
                .any(|a| arbre[a].genre == BARRE_DE_TITRE)
    };
    page_sans_enfant || !(0..arbre.len()).any(compte)
}

/// Ajoute `(cle, texte)` aux champs si `texte` n'est pas vide.
fn champ_non_vide(champs: &mut Vec<(&'static str, String)>, cle: &'static str, texte: &str) {
    if !texte.is_empty() {
        champs.push((cle, texte.to_string()));
    }
}

/// La ligne de l'élément `i` (ADR-0006, « La sortie ») : `[<Type>] <nom>`, puis, dans cet ordre,
/// `id`, `classe`, `valeur` ou `motdepasse=oui`, `raccourci`, `acces`, `annonce`, `etat`, `dans`,
/// `rang`.
///
/// `ligne_d_element(arbre, i, valeur, marque, dans, rang)` : `valeur` est la valeur lue de
/// l'élément, s'il y en a une ; `marque` dit que l'appelant a vu l'élément marqué mot de passe
/// (dans le focus lu, ou par le marquage relu juste avant la valeur), ce que le cache ne dit pas
/// toujours ; `dans` demande la clé `dans` ; `rang` donne la clé `rang`.
///
/// Un nom vide n'a pas de chevrons ; un `id`, une `classe`, un `raccourci` ou un `acces` vides n'ont
/// pas leur clé. `motdepasse=oui` s'écrit, sans aucune valeur, si `marque` est vrai ou si le cache
/// marque l'élément ; sinon `valeur` ne paraît que pour un élément qui offre le motif Value et dont
/// la valeur est donnée, `valeur=""` si elle est vide (ADR-0005, « Les valeurs secrètes ne sortent
/// jamais »). `annonce` vaut `polite` (1) ou `assertive` (2), absente sinon ; `etat` suit `etats`,
/// absent s'il n'y en a aucun ; `dans`, s'il est demandé, nomme `dans_de`, absent s'il ne nomme
/// rien.
pub fn ligne_d_element(
    arbre: &[ElementLu],
    i: usize,
    valeur: Option<&str>,
    marque: bool,
    dans: bool,
    rang: Option<usize>,
) -> Ligne {
    let e = &arbre[i];
    let mut champs = Vec::new();
    champ_non_vide(&mut champs, "id", &e.id);
    champ_non_vide(&mut champs, "classe", &e.classe);
    if marque || e.motdepasse {
        champs.push(("motdepasse", "oui".to_string()));
    } else if let (Some(_), Some(v)) = (e.lecture_seule, valeur) {
        champs.push(("valeur", v.to_string()));
    }
    champ_non_vide(&mut champs, "raccourci", &e.raccourci);
    champ_non_vide(&mut champs, "acces", &e.acces);
    match e.annonce {
        1 => champs.push(("annonce", "polite".to_string())),
        2 => champs.push(("annonce", "assertive".to_string())),
        _ => {}
    }
    let etats = etats(e);
    if !etats.is_empty() {
        champs.push(("etat", etats.join(",")));
    }
    if dans && let Some(nom) = dans_de(arbre, i) {
        champs.push(("dans", nom.to_string()));
    }
    if let Some(r) = rang {
        champs.push(("rang", r.to_string()));
    }
    Ligne {
        genre: nom_du_type(e.genre),
        nom: (!e.nom.is_empty()).then(|| e.nom.clone()),
        champs,
    }
}

/// L'arrêt d'une erreur de l'automatisation (ADR-0006, « Les codes de sortie ») : l'élément disparu
/// (`UIA_DISPARU`, `RPC_DECONNECTE`, `OBJET_DECONNECTE`) en 2 ; le délai dépassé, l'élément inactif,
/// le geste refusé (`UIA_NON_PRIS`, `UIA_OPERATION_INVALIDE`) et tout autre code en 3, ce dernier
/// avec le texte de l'erreur, échappé pour tenir en une ligne. Les codes se classent sans lire le
/// texte, que Windows ne donne pas pour tous et qu'il prête à tort à certains.
pub fn arret_uia(e: &ErreurUia) -> Arret {
    match e.code {
        UIA_DISPARU | RPC_DECONNECTE | OBJET_DECONNECTE => {
            Arret::Introuvable("l'élément a disparu".to_string())
        }
        UIA_DELAI => {
            Arret::Echec("échec du système : délai de l'automatisation dépassé".to_string())
        }
        UIA_INACTIF => Arret::Echec("échec du système : l'élément est inactif".to_string()),
        UIA_NON_PRIS | UIA_OPERATION_INVALIDE => {
            Arret::Echec("échec du système : l'élément refuse le geste".to_string())
        }
        _ => Arret::Echec(format!("échec du système : {}", echapper(&e.texte))),
    }
}

/// Les constructeurs des tests, que les modules de la lecture et des gestes réemploient.
#[cfg(test)]
pub(crate) mod essai {
    use super::DOCUMENT;
    use super::Defile;
    use super::ElementLu;
    use super::FENETRE;
    use super::GROUPE;
    use super::VOLET;

    /// Un élément actif du processus 100, sans motif, de ce parent, de ce type et de ce nom.
    pub fn e(parent: Option<usize>, genre: i32, nom: &str) -> ElementLu {
        ElementLu {
            parent,
            genre,
            nom: nom.into(),
            pid: 100,
            actif: true,
            ..Default::default()
        }
    }

    /// L'arbre de l'application d'essai, que les tests reprennent par ses indices : sous la fenêtre
    /// et son volet, le document, puis un bouton, un champ, un champ de mot de passe, une case, trois
    /// groupes d'un bouton radio « Toutes » chacun, deux groupes « Options » emboîtés, un cadre qui
    /// défile verticalement et son titre.
    pub fn arbre_d_essai() -> Vec<ElementLu> {
        let id = |id: &str, e: ElementLu| ElementLu { id: id.into(), ..e };
        vec![
            e(None, FENETRE, "Appli d'essai"),
            e(Some(0), VOLET, ""),
            e(Some(1), DOCUMENT, "Appli d'essai"),
            ElementLu {
                invocable: true,
                ..id("depart", e(Some(2), 50_000, "Départ"))
            },
            ElementLu {
                lecture_seule: Some(false),
                ..id("champ", e(Some(2), 50_004, "Champ"))
            },
            ElementLu {
                motdepasse: true,
                lecture_seule: Some(false),
                ..id("motdepasse", e(Some(2), 50_004, "Mot de passe"))
            },
            ElementLu {
                coche: Some(0),
                ..id("case", e(Some(2), 50_002, "Case"))
            },
            e(Some(2), GROUPE, "Couleur"),
            ElementLu {
                choisi: Some(true),
                ..id("couleur-toutes", e(Some(7), 50_013, "Toutes"))
            },
            e(Some(2), GROUPE, "Taille"),
            ElementLu {
                choisi: Some(true),
                ..id("taille-toutes", e(Some(9), 50_013, "Toutes"))
            },
            e(Some(2), GROUPE, "Forme"),
            ElementLu {
                choisi: Some(true),
                ..id("forme-toutes", e(Some(11), 50_013, "Toutes"))
            },
            e(Some(2), GROUPE, "Options"),
            ElementLu {
                coche: Some(0),
                ..id("option-externe", e(Some(13), 50_002, "Option externe"))
            },
            e(Some(13), GROUPE, "Options"),
            ElementLu {
                coche: Some(0),
                ..id("option-interne", e(Some(15), 50_002, "Option interne"))
            },
            ElementLu {
                defile: Some(Defile {
                    vertical: true,
                    horizontal: false,
                }),
                ..id("cadre", e(Some(2), GROUPE, "Cadre des lignes"))
            },
            id("titre-defilee", e(Some(2), 50_020, "Lignes")),
        ]
    }

    /// `[Window « Appli d'essai », Pane sans nom, Document sans nom ni enfant]`.
    pub fn vide() -> Vec<ElementLu> {
        vec![
            e(None, FENETRE, "Appli d'essai"),
            e(Some(0), VOLET, ""),
            e(Some(1), DOCUMENT, ""),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::essai::arbre_d_essai;
    use super::essai::e;
    use super::essai::vide;
    use super::*;
    use crate::domaine::sortie::formater;

    /// La ligne écrite de l'élément `i`.
    fn ligne(
        arbre: &[ElementLu],
        i: usize,
        valeur: Option<&str>,
        marque: bool,
        dans: bool,
        rang: Option<usize>,
    ) -> String {
        formater(&ligne_d_element(arbre, i, valeur, marque, dans, rang))
    }

    #[test]
    fn chaque_type_a_son_nom() {
        assert_eq!(nom_du_type(50_000), "Button");
        assert_eq!(nom_du_type(50_026), "Group");
        assert_eq!(nom_du_type(50_040), "AppBar");
        assert_eq!(nom_du_type(49_999), "Inconnu");
        assert_eq!(nom_du_type(50_041), "Inconnu");
        assert_eq!(type_de_nom("radiobutton"), Some(50_013));
        assert_eq!(type_de_nom("APPBAR"), Some(50_040));
        assert_eq!(type_de_nom("Inconnu"), None);
        assert_eq!(type_de_nom("Bouton"), None);
    }

    // Les constantes nommées tombent sur leur type dans la table, et les codes de l'automatisation
    // valent ceux de Windows : une constante fausse ferait classer un autre code, ou juger un autre
    // type, sans qu'aucun autre test le voie, puisqu'ils les nomment.
    #[test]
    fn constantes_valent_celles_de_windows() {
        assert_eq!(nom_du_type(PREMIER_TYPE), "Button");
        assert_eq!(
            [GROUPE, DOCUMENT, FENETRE, VOLET, BARRE_DE_TITRE].map(nom_du_type),
            ["Group", "Document", "Window", "Pane", "TitleBar"]
        );
        assert_eq!(
            [
                UIA_DISPARU,
                RPC_DECONNECTE,
                OBJET_DECONNECTE,
                UIA_DELAI,
                UIA_INACTIF,
                UIA_NON_PRIS,
                UIA_OPERATION_INVALIDE
            ],
            [
                0x8004_0201,
                0x8001_0108,
                0x8004_01FD,
                0x8013_1505,
                0x8004_0200,
                0x8004_0204,
                0x8013_1509
            ]
        );
    }

    #[test]
    fn ligne_d_element_dans_l_ordre_des_cles() {
        let case = ElementLu {
            id: "case".into(),
            classe: "c".into(),
            raccourci: "Ctrl+K".into(),
            acces: "k".into(),
            annonce: 1,
            coche: Some(1),
            actif: false,
            ..e(Some(1), 50_002, "Case")
        };
        let arbre = vec![
            e(None, FENETRE, "Fenêtre"),
            e(Some(0), GROUPE, "Groupe"),
            case,
        ];
        assert_eq!(
            ligne(&arbre, 2, None, false, true, Some(2)),
            "[CheckBox] <Case> id=case classe=c raccourci=Ctrl+K acces=k annonce=polite \
             etat=coche,inactif dans=Groupe rang=2"
        );
        // Sans `dans` ni rang demandés, la ligne n'a pas leurs clés, même sous un groupe nommé.
        assert_eq!(
            ligne(&arbre, 2, None, false, false, None),
            "[CheckBox] <Case> id=case classe=c raccourci=Ctrl+K acces=k annonce=polite \
             etat=coche,inactif"
        );
    }

    #[test]
    fn valeur_suit_la_classe() {
        let champ = ElementLu {
            id: "champ".into(),
            lecture_seule: Some(true),
            ..e(None, 50_004, "Champ")
        };
        let arbre = vec![champ.clone()];
        assert_eq!(
            ligne(&arbre, 0, Some("deux mots"), false, false, None),
            "[Edit] <Champ> id=champ valeur=\"deux mots\" etat=lecture-seule"
        );
        assert_eq!(
            ligne(&arbre, 0, Some(""), false, false, None),
            "[Edit] <Champ> id=champ valeur=\"\" etat=lecture-seule"
        );
        // La valeur suit la classe et précède le raccourci.
        let complet = vec![ElementLu {
            classe: "c".into(),
            raccourci: "r".into(),
            ..champ
        }];
        assert_eq!(
            ligne(&complet, 0, Some("v"), false, false, None),
            "[Edit] <Champ> id=champ classe=c valeur=v raccourci=r etat=lecture-seule"
        );
        // Un élément qui n'offre pas le motif Value n'a pas de valeur, même si on lui en donne une.
        assert_eq!(
            ligne(&[e(None, 50_000, "B")], 0, Some("x"), false, false, None),
            "[Button] <B>"
        );
    }

    #[test]
    fn motdepasse_remplace_la_valeur() {
        let a = arbre_d_essai();
        let marque_en_cache = ligne(&a, 5, Some("x"), false, false, None);
        assert!(marque_en_cache.contains("motdepasse=oui"));
        assert!(!marque_en_cache.contains("valeur=") && !marque_en_cache.contains('x'));
        assert_eq!(
            marque_en_cache,
            "[Edit] <Mot de passe> id=motdepasse motdepasse=oui"
        );
        // Sans valeur lue, l'élément marqué le dit encore.
        assert_eq!(
            ligne(&a, 5, None, false, false, None),
            "[Edit] <Mot de passe> id=motdepasse motdepasse=oui"
        );
        // Un élément que le cache ne marque pas, marqué par l'appelant (le focus lu, ou le marquage
        // relu juste avant la valeur) : sa valeur ne sort pas davantage.
        assert!(!a[4].motdepasse);
        let marque_a_la_lecture = ligne(&a, 4, Some("x"), true, false, None);
        assert!(marque_a_la_lecture.contains("motdepasse=oui"));
        assert!(!marque_a_la_lecture.contains("valeur=") && !marque_a_la_lecture.contains('x'));
        assert_eq!(
            marque_a_la_lecture,
            "[Edit] <Champ> id=champ motdepasse=oui"
        );
    }

    #[test]
    fn nom_vide_sans_chevrons_et_annonce_assertive() {
        let volet = |annonce| {
            vec![ElementLu {
                annonce,
                ..e(None, VOLET, "")
            }]
        };
        assert_eq!(ligne(&volet(0), 0, None, false, false, None), "[Pane]");
        assert_eq!(
            ligne(&volet(2), 0, None, false, false, None),
            "[Pane] annonce=assertive"
        );
        assert_eq!(ligne(&volet(3), 0, None, false, false, None), "[Pane]");
    }

    #[test]
    fn etats_dans_l_ordre() {
        let b = e(None, 50_000, "B");
        assert_eq!(etats(&b), Vec::<&str>::new());
        let avec = |f: fn(&mut ElementLu)| {
            let mut x = b.clone();
            f(&mut x);
            etats(&x)
        };
        assert_eq!(avec(|x| x.coche = Some(1)), ["coche"]);
        assert_eq!(avec(|x| x.coche = Some(0)), ["decoche"]);
        assert_eq!(avec(|x| x.coche = Some(2)), ["indetermine"]);
        assert_eq!(avec(|x| x.coche = Some(3)), Vec::<&str>::new());
        assert_eq!(
            avec(|x| {
                x.choisi = Some(true);
                x.actif = false;
            }),
            ["choisi", "inactif"]
        );
        assert_eq!(avec(|x| x.choisi = Some(false)), Vec::<&str>::new());
        assert_eq!(avec(|x| x.deplie = Some(1)), ["deplie"]);
        assert_eq!(avec(|x| x.deplie = Some(2)), ["deplie"]);
        assert_eq!(avec(|x| x.deplie = Some(0)), ["replie"]);
        assert_eq!(avec(|x| x.deplie = Some(3)), Vec::<&str>::new());
        assert_eq!(avec(|x| x.lecture_seule = Some(true)), ["lecture-seule"]);
        assert_eq!(avec(|x| x.lecture_seule = Some(false)), Vec::<&str>::new());
        assert_eq!(
            avec(|x| {
                x.coche = Some(1);
                x.choisi = Some(true);
                x.actif = false;
                x.deplie = Some(0);
                x.lecture_seule = Some(true);
            }),
            ["coche", "choisi", "inactif", "replie", "lecture-seule"]
        );
    }

    #[test]
    fn dans_nomme_le_groupe_le_plus_proche() {
        let mut a = arbre_d_essai();
        assert_eq!(dans_de(&a, 16), Some("Options"));
        assert_eq!(dans_de(&a, 8), Some("Couleur"));
        assert_eq!(dans_de(&a, 4), None);
        assert_eq!(ancetres(&a, 16), vec![15, 13, 2, 1, 0]);
        assert_eq!(profondeur(&a, 16), 5);
        assert_eq!(profondeur(&a, 0), 0);
        // Un dialogue dans l'arbre de W.
        a.push(e(Some(2), FENETRE, "Dialogue")); // 19
        a.push(e(Some(19), 50_000, "Fermer")); // 20
        assert_eq!(dans_de(&a, 20), None);
        // Un élément sous une List nommée « Lignes », elle-même sous un Group sans nom.
        a.push(e(Some(2), GROUPE, "")); // 21
        a.push(e(Some(21), 50_008, "Lignes")); // 22
        a.push(e(Some(22), 50_007, "Ligne 1")); // 23
        assert_eq!(dans_de(&a, 23), Some("Lignes"));
        // Un groupe nommé passe avant un ancêtre nommé plus proche d'un autre type.
        a.push(e(Some(7), 50_008, "Teintes")); // 24
        a.push(e(Some(24), 50_007, "Rouge")); // 25
        assert_eq!(dans_de(&a, 25), Some("Couleur"));
        // La racine ne compte pas, même nommée et d'un autre type que Window.
        let r = vec![e(None, VOLET, "Racine"), e(Some(0), 50_000, "B")];
        assert_eq!(dans_de(&r, 1), None);
    }

    // Un ancêtre écarté ne compte pas pour `dans`, comme tout élément `hors`.
    #[test]
    fn dans_ne_compte_pas_un_ancetre_ecarte() {
        let mut a = arbre_d_essai();
        a[7].hors = true;
        assert_eq!(dans_de(&a, 8), None);
    }

    #[test]
    fn document_vide_regle() {
        let w = || e(None, FENETRE, "W");
        assert!(document_vide(&vide()));
        assert!(document_vide(&[
            w(),
            e(Some(0), BARRE_DE_TITRE, ""),
            e(Some(1), 50_000, "Fermer"),
        ]));
        assert!(!document_vide(&[w(), e(Some(0), 50_000, "B")]));
        assert!(!document_vide(&arbre_d_essai()));
        assert!(!document_vide(&[
            w(),
            e(Some(0), VOLET, ""),
            e(Some(1), DOCUMENT, ""),
            e(Some(2), 50_025, "Contenu"),
        ]));
        // Un Document vide sous celui de la page ne rend pas l'arbre vide.
        assert!(!document_vide(&[
            w(),
            e(Some(0), VOLET, ""),
            e(Some(1), DOCUMENT, ""),
            e(Some(2), 50_000, "B"),
            e(Some(2), DOCUMENT, ""),
        ]));
        // Le premier Document le plus extérieur, celui de la page, est vide.
        assert!(document_vide(&[
            w(),
            e(Some(0), VOLET, ""),
            e(Some(1), DOCUMENT, ""),
            e(Some(1), DOCUMENT, ""),
            e(Some(3), 50_000, "B"),
        ]));
        // Le Document d'un dialogue que W possède n'est pas celui de la page.
        assert!(document_vide(&[
            w(),
            e(Some(0), FENETRE, "Dialogue"),
            e(Some(1), DOCUMENT, ""),
            e(Some(2), 50_000, "B"),
            e(Some(0), VOLET, ""),
            e(Some(4), DOCUMENT, ""),
        ]));
        // Rien d'écarté ne compte.
        let mut a = arbre_d_essai();
        for x in &mut a[2..] {
            x.hors = true;
        }
        assert!(document_vide(&a));
    }

    #[test]
    fn borner_ecarte_une_fenetre_d_un_processus_non_admis() {
        let mut a = arbre_d_essai();
        // 19, une fenêtre possédée.
        a.push(ElementLu {
            fenetre: true,
            pid: 300,
            ..e(Some(0), FENETRE, "Invite")
        });
        a.push(ElementLu {
            pid: 300,
            lecture_seule: Some(false),
            id: "secret".into(),
            ..e(Some(19), 50_004, "Secret")
        });
        // 21, une fenêtre fille.
        a.push(ElementLu {
            fenetre: true,
            pid: 400,
            ..e(Some(1), FENETRE, "Contenu")
        });
        // 22, pas une fenêtre : son processus ne se juge pas.
        a.push(ElementLu {
            pid: 300,
            ..e(Some(1), 50_020, "Texte")
        });
        // Le Pane de W, une fenêtre de P : jamais écarté.
        a[1].fenetre = true;
        borner(&mut a, |pid| pid == 100 || pid == 400);
        assert!(a[19].hors && a[20].hors);
        assert!(!a[21].hors && !a[22].hors && !a[1].hors && !a[0].hors);
        assert!((0..19).all(|i| !a[i].hors));
        // Jamais la racine.
        let mut r = arbre_d_essai();
        r[0].fenetre = true;
        r[0].pid = 300;
        borner(&mut r, |_| false);
        assert!(!r[0].hors);
    }

    #[test]
    fn arret_uia_classe_les_codes() {
        let e = |code| ErreurUia {
            code,
            texte: "Invoke : 0x80004005".into(),
        };
        for c in [UIA_DISPARU, RPC_DECONNECTE, OBJET_DECONNECTE] {
            assert_eq!(
                arret_uia(&e(c)),
                Arret::Introuvable("l'élément a disparu".into())
            );
        }
        assert_eq!(
            arret_uia(&e(UIA_DELAI)),
            Arret::Echec("échec du système : délai de l'automatisation dépassé".into())
        );
        assert_eq!(
            arret_uia(&e(UIA_INACTIF)),
            Arret::Echec("échec du système : l'élément est inactif".into())
        );
        for c in [UIA_NON_PRIS, UIA_OPERATION_INVALIDE] {
            assert_eq!(
                arret_uia(&e(c)),
                Arret::Echec("échec du système : l'élément refuse le geste".into())
            );
        }
        assert_eq!(
            arret_uia(&e(0x8000_4005)),
            Arret::Echec("échec du système : Invoke : 0x80004005".into())
        );
    }

    // Le texte d'un autre code s'écrit échappé : le message de Windows qu'il cite tient en une ligne.
    #[test]
    fn arret_uia_echappe_le_texte() {
        let e = ErreurUia {
            code: 0x8000_4005,
            texte: "Invoke : 0x80004005 : a\r\nb".into(),
        };
        assert_eq!(
            arret_uia(&e),
            Arret::Echec(r"échec du système : Invoke : 0x80004005 : a\r\nb".into())
        );
    }
}
