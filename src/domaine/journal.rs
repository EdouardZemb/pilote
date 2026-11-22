//! Le journal : une ligne par appel de `pilote` (ADR-0005, « Le journal »), qui garde la date
//! locale, la session, la sous-commande, le processus visé, le code de sortie et la longueur du
//! texte tapé, jamais le texte. Les champs se séparent par une tabulation et la ligne finit par un
//! retour ; un champ absent reste vide ; chaque champ passe par `echapper`, si bien qu'une session
//! ou un processus qui porte une tabulation ou un retour ne coupe ni ses voisins ni la ligne. La
//! sous-commande se lit dans une liste fermée : un premier mot inconnu, un texte tapé compris,
//! s'écrit `inconnue`.

use crate::domaine::sortie::Code;
use crate::domaine::sortie::echapper;
use std::ffi::OsString;

/// La date et l'heure locales d'un appel, à la seconde.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateLocale {
    pub annee: u16,
    pub mois: u8,
    pub jour: u8,
    pub heure: u8,
    pub minute: u8,
    pub seconde: u8,
}

/// Les sous-commandes que le journal nomme : les 21 du lot, puis `aide` et `version`.
pub const SOUS_COMMANDES: [&str; 23] = [
    "fenetres",
    "activer",
    "touche",
    "taper",
    "focus",
    "arbre",
    "trouver",
    "focaliser",
    "invoquer",
    "choisir",
    "valeur",
    "defiler",
    "defilables",
    "capture",
    "molette",
    "clic",
    "taille",
    "lancer",
    "fermer",
    "nettoyer",
    "annonces",
    "aide",
    "version",
];

/// La sous-commande de `args`, le nom du programme en tête, lue sur le seul premier argument qui
/// le suit : `--version` et `-V` donnent `version`, `--help` et `-h` donnent `aide`, un nom de
/// `SOUS_COMMANDES` se rend tel quel ; tout autre mot (`help` compris), un argument absent ou qui
/// n'est pas de l'Unicode valide donnent `inconnue`. Rien d'autre des arguments ne passe au
/// journal : ni une option, ni un texte tapé.
pub fn sous_commande(args: &[OsString]) -> &'static str {
    match args.get(1).and_then(|premier| premier.to_str()) {
        Some("--version" | "-V") => "version",
        Some("--help" | "-h") => "aide",
        Some(mot) => SOUS_COMMANDES
            .into_iter()
            .find(|nom| *nom == mot)
            .unwrap_or("inconnue"),
        None => "inconnue",
    }
}

/// Un appel de `pilote`, tel que le journal le garde.
#[derive(Clone, Debug, PartialEq)]
pub struct Appel<'a> {
    /// La date locale de l'appel.
    pub date: DateLocale,
    /// La session de Claude Code (`CLAUDE_CODE_SESSION_ID`), si la variable existe.
    pub session: Option<&'a str>,
    /// La sous-commande, que rend `sous_commande`.
    pub sous_commande: &'static str,
    /// Le nom du processus visé, s'il y en a un.
    pub processus: Option<&'a str>,
    /// Le code de sortie.
    pub code: Code,
    /// La longueur du texte tapé, pour `taper` : jamais le texte.
    pub longueur: Option<usize>,
}

/// La ligne du journal : la date, `AAAA-MM-JJTHH:MM:SS`, la session, la sous-commande, le
/// processus, la valeur du code et la longueur, dans cet ordre, séparés par une tabulation, chacun
/// échappé, un champ absent vide, puis un retour à la ligne.
pub fn ligne(appel: &Appel) -> String {
    let d = appel.date;
    let champs = [
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            d.annee, d.mois, d.jour, d.heure, d.minute, d.seconde
        ),
        appel.session.unwrap_or_default().to_string(),
        appel.sous_commande.to_string(),
        appel.processus.unwrap_or_default().to_string(),
        appel.code.valeur().to_string(),
        appel
            .longueur
            .map(|longueur| longueur.to_string())
            .unwrap_or_default(),
    ];
    let echappes = champs.map(|champ| echapper(&champ));
    format!("{}\n", echappes.join("\t"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::cli::Aide;
    use crate::domaine::cli::Commande;
    use crate::domaine::cli::Source;
    use crate::domaine::cli::analyser;
    use crate::domaine::cli::essai::a;

    /// Une date locale.
    fn d(annee: u16, mois: u8, jour: u8, heure: u8, minute: u8, seconde: u8) -> DateLocale {
        DateLocale {
            annee,
            mois,
            jour,
            heure,
            minute,
            seconde,
        }
    }

    /// Un appel complet de référence : chaque champ présent, et distinct des autres.
    fn appel() -> Appel<'static> {
        Appel {
            date: d(2026, 11, 21, 14, 3, 9),
            session: Some("s1"),
            sous_commande: "taper",
            processus: Some("appli-essai"),
            code: Code::Fait,
            longueur: Some(12),
        }
    }

    #[test]
    fn ligne_forme_exacte() {
        assert_eq!(
            ligne(&Appel {
                date: d(2026, 11, 21, 14, 3, 9),
                session: Some("s1"),
                sous_commande: "taper",
                processus: Some("appli-essai"),
                code: Code::Fait,
                longueur: Some(12)
            }),
            "2026-11-21T14:03:09\ts1\ttaper\tappli-essai\t0\t12\n"
        );
    }
    #[test]
    fn champs_absents_vides() {
        assert_eq!(
            ligne(&Appel {
                date: d(2026, 1, 2, 3, 4, 5),
                session: None,
                sous_commande: "inconnue",
                processus: None,
                code: Code::Usage,
                longueur: None
            }),
            "2026-01-02T03:04:05\t\tinconnue\t\t64\t\n"
        );
    }
    #[test]
    fn ligne_echappe_une_tabulation_de_fenetre() {
        let l = ligne(&Appel {
            session: Some("a\tb\nc"),
            ..appel()
        });
        assert_eq!(l.matches('\t').count(), 5);
        assert_eq!(l.matches('\n').count(), 1);
        assert!(l.contains(r"a\tb\nc"));
    }
    #[test]
    fn sous_commande_d_une_liste_fermee() {
        assert_eq!(sous_commande(&a("pilote taper --texte x")), "taper");
        assert_eq!(sous_commande(&a("pilote secret-tape")), "inconnue");
        assert_eq!(sous_commande(&a("pilote help")), "inconnue");
        assert_eq!(sous_commande(&a("pilote --version")), "version");
        assert_eq!(sous_commande(&a("pilote -V")), "version");
        assert_eq!(sous_commande(&a("pilote --help")), "aide");
        assert_eq!(sous_commande(&a("pilote -h")), "aide");
        assert_eq!(sous_commande(&a("pilote")), "inconnue");
    }
    #[test]
    fn ligne_d_un_appel_mal_forme_ne_porte_pas_ses_arguments() {
        let args = a("pilote taper --fenetre xyz --txt motdepasse");
        let l = ligne(&Appel {
            sous_commande: sous_commande(&args),
            processus: None,
            session: None,
            code: Code::Usage,
            longueur: None,
            date: d(2026, 1, 1, 0, 0, 0),
        });
        assert_eq!(l, "2026-01-01T00:00:00\t\ttaper\t\t64\t\n");
    }

    /// La liste fermée, nom par nom et dans l'ordre : les 21 sous-commandes du lot, puis `aide` et
    /// `version`, que le journal nomme sans que l'analyse les connaisse.
    #[test]
    fn sous_commandes_du_lot_puis_aide_et_version() {
        assert_eq!(
            SOUS_COMMANDES,
            [
                "fenetres",
                "activer",
                "touche",
                "taper",
                "focus",
                "arbre",
                "trouver",
                "focaliser",
                "invoquer",
                "choisir",
                "valeur",
                "defiler",
                "defilables",
                "capture",
                "molette",
                "clic",
                "taille",
                "lancer",
                "fermer",
                "nettoyer",
                "annonces",
                "aide",
                "version"
            ]
        );
    }

    /// Les 21 premiers noms sont les sous-commandes de l'analyse : chacun y rend son aide. `aide`
    /// et `version` n'en sont pas : l'analyse les refuse comme des sous-commandes inconnues.
    #[test]
    fn les_21_du_lot_sont_les_sous_commandes_de_l_analyse() {
        for nom in &SOUS_COMMANDES[..21] {
            let rendu = analyser(&a(&format!("{nom} --help")));
            assert!(
                matches!(
                    rendu,
                    Err(Aide {
                        code: Code::Fait,
                        ..
                    })
                ),
                "{nom}"
            );
        }
        for nom in &SOUS_COMMANDES[21..] {
            assert_eq!(
                analyser(&a(nom)),
                Err(Aide {
                    code: Code::Usage,
                    texte: format!("sous-commande inconnue : {nom}")
                })
            );
        }
    }

    /// Chaque nom de la liste, en premier argument, se rend tel quel.
    #[test]
    fn chaque_nom_de_la_liste_se_rend() {
        for nom in SOUS_COMMANDES {
            assert_eq!(sous_commande(&a(nom)), nom);
        }
    }

    /// La date s'écrit sur quatre chiffres, puis deux pour chacun des autres, complétés par des
    /// zéros, jamais par des espaces.
    #[test]
    fn date_sur_quatre_puis_deux_chiffres() {
        assert_eq!(
            ligne(&Appel {
                date: d(987, 1, 2, 3, 4, 5),
                ..appel()
            }),
            "0987-01-02T03:04:05\ts1\ttaper\tappli-essai\t0\t12\n"
        );
    }

    /// Le processus s'échappe comme la session : sa tabulation, son retour chariot et son chevron
    /// ne coupent ni le champ ni la ligne.
    #[test]
    fn ligne_echappe_le_processus() {
        assert_eq!(
            ligne(&Appel {
                processus: Some("p\tq\r>"),
                ..appel()
            }),
            concat!("2026-11-21T14:03:09\ts1\ttaper\t", r"p\tq\r\>", "\t0\t12\n")
        );
    }

    /// Aucune source du texte à taper, `--texte`, `--fichier` ou l'entrée standard, ne laisse son
    /// texte ni son chemin dans la ligne : la sous-commande se lit dans la liste fermée, et seule
    /// la longueur du texte s'y écrit.
    #[test]
    fn ligne_ne_porte_le_texte_d_aucune_source() {
        let sources = [
            (
                "pilote taper --fenetre appli-essai --texte motdepasse",
                Source::Texte("motdepasse".into()),
            ),
            (
                "pilote taper --fenetre appli-essai --fichier motdepasse.txt",
                Source::Fichier("motdepasse.txt".into()),
            ),
            ("pilote taper --fenetre appli-essai -", Source::Entree),
        ];
        for (ligne_de_commande, attendue) in sources {
            let args = a(ligne_de_commande);
            match analyser(&args) {
                Ok(Commande::Taper { source, .. }) => assert_eq!(source, attendue),
                autre => panic!("{ligne_de_commande} : {autre:?}"),
            }
            let l = ligne(&Appel {
                sous_commande: sous_commande(&args),
                longueur: Some(10),
                ..appel()
            });
            assert_eq!(l, "2026-11-21T14:03:09\ts1\ttaper\tappli-essai\t0\t10\n");
        }
    }
}
