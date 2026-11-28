//! La sortie de `pilote` : les codes de sortie, les lignes de la sortie standard et leurs
//! échappements, les arrêts, et l'issue d'une commande. Ce qui s'écrit tient en une ligne : un nom,
//! un titre, un chemin ou un caractère qui porterait un retour ou un contrôle s'y écrit échappé.

/// Le code de sortie de `pilote`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// 0 : la commande est faite.
    Fait,
    /// 1 : une garde refuse.
    Refus,
    /// 2 : la fenêtre visée est absente, ou plusieurs répondent.
    Introuvable,
    /// 3 : le système a échoué.
    Echec,
    /// 64 : l'usage est faux.
    Usage,
}

impl Code {
    /// La valeur du code de sortie : 0, 1, 2, 3 ou 64.
    pub fn valeur(self) -> u8 {
        match self {
            Code::Fait => 0,
            Code::Refus => 1,
            Code::Introuvable => 2,
            Code::Echec => 3,
            Code::Usage => 64,
        }
    }
}

/// Une ligne de la sortie standard : `[genre]`, son nom entre chevrons s'il existe, puis ses
/// champs `clé=valeur`, dans l'ordre.
#[derive(Clone, Debug, PartialEq)]
pub struct Ligne {
    pub genre: &'static str,
    pub nom: Option<String>,
    pub champs: Vec<(&'static str, String)>,
}

/// `texte` échappé : `\` devient `\\`, le retour à la ligne `\n`, le retour chariot `\r`, la
/// tabulation `\t`, `>` devient `\>`, et tout autre caractère de contrôle (catégorie Cc), U+2028
/// et U+2029 s'écrivent `\u{XXXX}`, en chiffres hexadécimaux majuscules, quatre au moins.
pub fn echapper(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for c in texte.chars() {
        match c {
            '\\' => sortie.push_str(r"\\"),
            '\n' => sortie.push_str(r"\n"),
            '\r' => sortie.push_str(r"\r"),
            '\t' => sortie.push_str(r"\t"),
            '>' => sortie.push_str(r"\>"),
            c if c.is_control() || c == '\u{2028}' || c == '\u{2029}' => {
                sortie.push_str(&format!(r"\u{{{:04X}}}", u32::from(c)));
            }
            c => sortie.push(c),
        }
    }
    sortie
}

/// La valeur d'un champ : `texte` échappé, ses guillemets en `\"`, le tout entre guillemets s'il
/// contient un guillemet ou une espace, c'est-à-dire tout caractère blanc d'Unicode resté tel quel
/// après l'échappement, U+00A0, U+2007 et U+202F compris ; vide, `""`. Le blanc se juge sur le
/// texte échappé : une tabulation ou un retour, écrits `\t` ou `\n`, ne changent pas de forme, et
/// un texte que l'application pilotée choisit ne peut pas forger une clé après un blanc qu'une
/// session prendrait pour un séparateur.
pub fn valeur(texte: &str) -> String {
    let echappe = echapper(texte).replace('"', r#"\""#);
    if texte.is_empty() || texte.contains('"') || echappe.chars().any(char::is_whitespace) {
        format!("\"{echappe}\"")
    } else {
        echappe
    }
}

/// La ligne écrite : `[genre]`, puis ` <nom>`, le nom échappé, s'il existe, puis ` clé=valeur`
/// pour chaque champ, dans l'ordre.
pub fn formater(ligne: &Ligne) -> String {
    let mut sortie = format!("[{}]", ligne.genre);
    if let Some(nom) = &ligne.nom {
        sortie.push_str(" <");
        sortie.push_str(&echapper(nom));
        sortie.push('>');
    }
    for (cle, texte) in &ligne.champs {
        sortie.push(' ');
        sortie.push_str(cle);
        sortie.push('=');
        sortie.push_str(&valeur(texte));
    }
    sortie
}

/// Un arrêt : la commande s'arrête, avec son code et son message d'une ligne, qui part sur la
/// sortie d'erreur.
#[derive(Clone, Debug, PartialEq)]
pub enum Arret {
    /// 64 : l'usage est faux.
    Usage(String),
    /// 1 : une garde refuse.
    Refus(String),
    /// 2 : la fenêtre visée est absente, ou plusieurs répondent.
    Introuvable(String),
    /// 3 : le système a échoué.
    Echec(String),
}

impl Arret {
    /// Le code de sortie de l'arrêt.
    pub fn code(&self) -> Code {
        match self {
            Arret::Usage(_) => Code::Usage,
            Arret::Refus(_) => Code::Refus,
            Arret::Introuvable(_) => Code::Introuvable,
            Arret::Echec(_) => Code::Echec,
        }
    }

    /// Le message de l'arrêt.
    pub fn message(&self) -> &str {
        match self {
            Arret::Usage(message)
            | Arret::Refus(message)
            | Arret::Introuvable(message)
            | Arret::Echec(message) => message,
        }
    }
}

/// L'échec du système : « échec du système : <quoi> » (ADR-0006, « Les messages »). Chaque échec
/// du système passe par elle : le préfixe ne s'écrit qu'ici.
pub(crate) fn echec(quoi: &str) -> Arret {
    Arret::Echec(format!("échec du système : {quoi}"))
}

/// L'issue d'une commande : son code, ses lignes de la sortie standard, et le message de la sortie
/// d'erreur, s'il y en a un.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub code: Code,
    pub sortie: Vec<String>,
    pub erreur: Option<String>,
}

impl Issue {
    /// La commande est faite : le code `Fait`, ces lignes, sans erreur.
    pub fn fait(sortie: Vec<String>) -> Issue {
        Issue {
            code: Code::Fait,
            sortie,
            erreur: None,
        }
    }

    /// Fait, ces lignes, et un avis sur la sortie d'erreur.
    pub fn avis(sortie: Vec<String>, avis: String) -> Issue {
        Issue {
            code: Code::Fait,
            sortie,
            erreur: Some(avis),
        }
    }

    /// La commande s'arrête : le code de l'arrêt, sans sortie, l'erreur étant son message.
    pub fn arret(a: Arret) -> Issue {
        Issue {
            code: a.code(),
            sortie: Vec::new(),
            erreur: Some(a.message().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_valent_0_1_2_3_64() {
        assert_eq!(
            [
                Code::Fait,
                Code::Refus,
                Code::Introuvable,
                Code::Echec,
                Code::Usage
            ]
            .map(Code::valeur),
            [0, 1, 2, 3, 64]
        );
    }
    #[test]
    fn arrets_portent_leur_code() {
        assert_eq!(
            [
                Arret::Usage("u".into()).code(),
                Arret::Refus("r".into()).code(),
                Arret::Introuvable("i".into()).code(),
                Arret::Echec("e".into()).code()
            ],
            [Code::Usage, Code::Refus, Code::Introuvable, Code::Echec]
        );
    }
    #[test]
    fn echappe_barre_retour_tab_chevron() {
        assert_eq!(echapper("a\\b\nc\rd\te>f"), r"a\\b\nc\rd\te\>f");
    }
    #[test]
    fn sortie_echappe_les_controles() {
        assert_eq!(
            echapper("a\u{7}b\u{2028}c\u{2029}d\u{85}"),
            r"a\u{0007}b\u{2028}c\u{2029}d\u{0085}"
        );
    }
    #[test]
    fn valeur_entre_guillemets_si_espace_ou_guillemet() {
        assert_eq!(valeur("a b"), r#""a b""#);
        assert_eq!(valeur(r#"a"b"#), r#""a\"b""#);
        assert_eq!(valeur("ab"), "ab");
    }
    // Un texte que l'application pilotée choisit (un `id`, une classe, une valeur) ne forge pas de
    // clé dans la ligne : tout caractère blanc d'Unicode resté tel quel après l'échappement met la
    // valeur entre guillemets, ce caractère gardé tel quel. Une tabulation, que l'échappement écrit
    // `\t`, ne change pas de forme.
    #[test]
    fn valeur_entre_guillemets_si_blanc_d_unicode() {
        assert_eq!(valeur("a\u{A0}motdepasse=oui"), "\"a\u{A0}motdepasse=oui\"");
        for blanc in ['\u{2007}', '\u{202F}', '\u{3000}'] {
            let texte = format!("a{blanc}motdepasse=oui");
            assert_eq!(valeur(&texte), format!("\"{texte}\""));
        }
        assert_eq!(valeur("a\tb"), r"a\tb");
    }
    #[test]
    fn valeur_vide_s_ecrit_guillemets() {
        assert_eq!(valeur(""), r#""""#);
    }
    #[test]
    fn ligne_fenetre_forme_exacte() {
        let l = Ligne {
            genre: "Fenetre",
            nom: Some("Appli d'essai".into()),
            champs: vec![
                ("processus", "appli-essai".into()),
                ("pid", "100".into()),
                ("premier-plan", "oui".into()),
            ],
        };
        assert_eq!(
            formater(&l),
            "[Fenetre] <Appli d'essai> processus=appli-essai pid=100 premier-plan=oui"
        );
    }
    #[test]
    fn ligne_echappe_son_nom_et_ses_valeurs() {
        let l = Ligne {
            genre: "Fenetre",
            nom: Some("a>b".into()),
            champs: vec![("processus", "mon appli".into())],
        };
        assert_eq!(formater(&l), r#"[Fenetre] <a\>b> processus="mon appli""#);
    }
    #[test]
    fn ligne_sans_nom() {
        assert_eq!(
            formater(&Ligne {
                genre: "Ecoute",
                nom: None,
                champs: vec![]
            }),
            "[Ecoute]"
        );
    }
    #[test]
    fn refus_en_2_garde_une_ligne() {
        let i = Issue::arret(Arret::Introuvable(format!(
            "plusieurs fenêtres : <{}>",
            echapper("a\nb")
        )));
        assert_eq!(i.code, Code::Introuvable);
        assert!(!i.erreur.unwrap().contains('\n'));
    }

    // Le message de chaque arrêt, lu tel quel : un message vidé ou remplacé ferait échouer ce test.
    // Puis les deux formes d'`Issue` entières, et les chiffres hexadécimaux en majuscules que
    // promet `echapper`.
    #[test]
    fn arrets_portent_leur_message() {
        assert_eq!(
            [
                Arret::Usage("u".into()).message(),
                Arret::Refus("r".into()).message(),
                Arret::Introuvable("i".into()).message(),
                Arret::Echec("e".into()).message()
            ],
            ["u", "r", "i", "e"]
        );
    }
    #[test]
    fn issue_fait_sans_erreur() {
        assert_eq!(
            Issue::fait(vec!["l".into()]),
            Issue {
                code: Code::Fait,
                sortie: vec!["l".into()],
                erreur: None
            }
        );
    }
    #[test]
    fn issue_arret_porte_le_message_sans_sortie() {
        assert_eq!(
            Issue::arret(Arret::Refus("poste verrouillé".into())),
            Issue {
                code: Code::Refus,
                sortie: vec![],
                erreur: Some("poste verrouillé".into())
            }
        );
    }
    #[test]
    fn issue_avis_garde_sa_sortie() {
        assert_eq!(
            Issue::avis(vec!["a".into()], "b".into()),
            Issue {
                code: Code::Fait,
                sortie: vec!["a".into()],
                erreur: Some("b".into())
            }
        );
    }
    #[test]
    fn echec_dit_ce_qui_a_echoue() {
        assert_eq!(
            echec("SendInput"),
            Arret::Echec("échec du système : SendInput".into())
        );
    }
    #[test]
    fn echappe_en_hexadecimal_majuscule() {
        assert_eq!(echapper("\u{1b}\u{7f}\u{9b}"), r"\u{001B}\u{007F}\u{009B}");
    }

    // `valeur` échappe, et une valeur qui porte un retour sort sur une seule ligne : ses entrées
    // portent chacune un caractère que `echapper` transforme, si bien qu'une `valeur` qui se
    // passerait d'`echapper` ferait échouer ce test.
    #[test]
    fn valeur_echappe_ses_caracteres() {
        assert_eq!(valeur("a\nb>c\\d\u{7}"), r"a\nb\>c\\d\u{0007}");
        assert_eq!(valeur("a b\n"), r#""a b\n""#);
        assert_eq!(valeur("a\"b\n"), r#""a\"b\n""#);
    }
    #[test]
    fn ligne_dont_la_valeur_porte_un_retour_tient_en_une_ligne() {
        let l = Ligne {
            genre: "Fenetre",
            nom: Some("t\nu".into()),
            champs: vec![("processus", "a\nb".into())],
        };
        let ecrite = formater(&l);
        assert_eq!(ecrite, r"[Fenetre] <t\nu> processus=a\nb");
        assert!(!ecrite.contains('\n'));
    }
}
