//! Les touches de `touche` : leurs noms, leurs combinaisons, celles que `pilote` refuse, et le lot
//! d'événements qui frappe une combinaison. L'analyse ne lit aucune fenêtre (décision 5) : une
//! combinaison interdite sort en 1, un nom inconnu en 64, le texte reçu échappé dans le message
//! (décision 6). Chaque lot relâche ce qu'il enfonce, dans l'ordre inverse.

use crate::domaine::sortie::Arret;
use crate::domaine::sortie::echapper;

/// Un événement du clavier ou de la souris, tel que l'envoi le reçoit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evenement {
    /// Une touche virtuelle et l'octet bas de son code de balayage, étendu ou non, enfoncée ou
    /// relâchée.
    Touche {
        vk: u8,
        scan: u8,
        etendue: bool,
        relachee: bool,
    },
    /// Une unité UTF-16, enfoncée ou relâchée.
    Unicode { unite: u16, relachee: bool },
    /// Un mouvement de souris nul.
    SourisNulle,
}

/// Une touche nommée sur la ligne de commande. `F` va de 1 à 12 ; `Lettre` est une majuscule, de
/// A à Z. `Barre` et `Interrogation` sont les caractères `/` et `?`, que la disposition du clavier
/// place (décision 9) : ils n'ont pas de touche virtuelle fixe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NomTouche {
    Tab,
    Echap,
    Entree,
    Espace,
    Retour,
    Suppr,
    Inser,
    Haut,
    Bas,
    Gauche,
    Droite,
    Debut,
    Fin,
    PageHaut,
    PageBas,
    F(u8),
    Lettre(char),
    Barre,
    Interrogation,
}

/// Une combinaison : les modificateurs demandés, puis la touche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combinaison {
    pub ctrl: bool,
    pub maj: bool,
    pub alt: bool,
    pub touche: NomTouche,
}

/// Les touches virtuelles de Windows que `pilote` emploie.
pub mod vk {
    pub const RETOUR_CHARIOT: u8 = 0x0D;
    pub const TAB: u8 = 0x09;
    pub const ECHAP: u8 = 0x1B;
    pub const ESPACE: u8 = 0x20;
    pub const RETOUR: u8 = 0x08;
    pub const SUPPR: u8 = 0x2E;
    pub const INSER: u8 = 0x2D;
    pub const DEBUT: u8 = 0x24;
    pub const FIN: u8 = 0x23;
    pub const PAGE_HAUT: u8 = 0x21;
    pub const PAGE_BAS: u8 = 0x22;
    pub const GAUCHE: u8 = 0x25;
    pub const HAUT: u8 = 0x26;
    pub const DROITE: u8 = 0x27;
    pub const BAS: u8 = 0x28;
    /// F1 ; F1 à F12 vont de 0x70 à 0x7B.
    pub const F1: u8 = 0x70;
    /// A ; A à Z vont de 0x41 à 0x5A.
    pub const A: u8 = 0x41;
    // Maj, Ctrl et Alt sans côté, puis de chaque côté.
    pub const MAJ: u8 = 0x10;
    pub const CTRL: u8 = 0x11;
    pub const ALT: u8 = 0x12;
    pub const MAJ_G: u8 = 0xA0;
    pub const MAJ_D: u8 = 0xA1;
    pub const CTRL_G: u8 = 0xA2;
    pub const CTRL_D: u8 = 0xA3;
    pub const ALT_G: u8 = 0xA4;
    pub const ALT_D: u8 = 0xA5;
    pub const WIN_G: u8 = 0x5B;
    pub const WIN_D: u8 = 0x5C;
    pub const VERR_MAJ: u8 = 0x14;
    /// Une touche virtuelle que Windows n'attribue à rien.
    pub const SANS_EFFET: u8 = 0xE8;
    /// Les modificateurs qu'une main peut tenir enfoncés : Maj, Ctrl, Alt et Windows, de chaque
    /// côté.
    pub const MODIFICATEURS: [u8; 8] = [MAJ_G, MAJ_D, CTRL_G, CTRL_D, ALT_G, ALT_D, WIN_G, WIN_D];
}

/// Analyse une touche ou une combinaison de `touche`. Les noms se découpent sur `+`, sans égard à
/// la casse (ASCII) ; `Ctrl`, `Maj` et `Alt` sont les modificateurs, dans tout ordre, et la touche
/// vient en dernier. Une lettre, de A à Z, n'est admise qu'avec Ctrl ; `barre` et `interrogation`
/// s'écrivent seules.
///
/// Les combinaisons interdites feraient sortir la frappe de la fenêtre visée : Win ou Windows en
/// tout morceau, Alt avec Tab, Ctrl et Alt avec Suppr, Alt ou Ctrl avec Echap. Elles se
/// reconnaissent sur les noms, sous toute casse et dans tout ordre, avant tout nom inconnu, et
/// refusent en 1, « touche interdite : <texte> ». Tout autre texte qui n'a pas cette forme sort en
/// 64, « touche inconnue : <texte> ». Le texte reçu y est échappé.
pub fn analyser_touche(texte: &str) -> Result<Combinaison, Arret> {
    let morceaux: Vec<String> = texte.split('+').map(str::to_ascii_lowercase).collect();
    let porte = |nom: &str| morceaux.iter().any(|m| m == nom);
    let (ctrl, alt) = (porte("ctrl"), porte("alt"));
    if porte("win")
        || porte("windows")
        || (alt && porte("tab"))
        || (ctrl && alt && porte("suppr"))
        || ((alt || ctrl) && porte("echap"))
    {
        return Err(Arret::Refus(format!(
            "touche interdite : {}",
            echapper(texte)
        )));
    }
    let inconnue = || Arret::Usage(format!("touche inconnue : {}", echapper(texte)));
    let (nom, modificateurs) = morceaux.split_last().ok_or_else(inconnue)?;
    let touche = nom_touche(nom).ok_or_else(inconnue)?;
    let mut c = Combinaison {
        ctrl: false,
        maj: false,
        alt: false,
        touche,
    };
    for m in modificateurs {
        match m.as_str() {
            "ctrl" => c.ctrl = true,
            "maj" => c.maj = true,
            "alt" => c.alt = true,
            _ => return Err(inconnue()),
        }
    }
    match touche {
        NomTouche::Lettre(_) if !c.ctrl => Err(inconnue()),
        NomTouche::Barre | NomTouche::Interrogation if c.ctrl || c.maj || c.alt => Err(inconnue()),
        _ => Ok(c),
    }
}

/// La touche que nomme `nom`, en minuscules : un nom du design, F1 à F12, ou une lettre seule.
fn nom_touche(nom: &str) -> Option<NomTouche> {
    let touche = match nom {
        "tab" => NomTouche::Tab,
        "echap" => NomTouche::Echap,
        "entree" => NomTouche::Entree,
        "espace" => NomTouche::Espace,
        "retour" => NomTouche::Retour,
        "suppr" => NomTouche::Suppr,
        "inser" => NomTouche::Inser,
        "haut" => NomTouche::Haut,
        "bas" => NomTouche::Bas,
        "gauche" => NomTouche::Gauche,
        "droite" => NomTouche::Droite,
        "debut" => NomTouche::Debut,
        "fin" => NomTouche::Fin,
        "pagehaut" => NomTouche::PageHaut,
        "pagebas" => NomTouche::PageBas,
        "barre" => NomTouche::Barre,
        "interrogation" => NomTouche::Interrogation,
        _ => return lettre(nom).or_else(|| fonction(nom)),
    };
    Some(touche)
}

/// La lettre que nomme `nom`, une seule lettre ASCII minuscule, rendue en majuscule.
fn lettre(nom: &str) -> Option<NomTouche> {
    match nom.as_bytes() {
        [l] if l.is_ascii_lowercase() => {
            Some(NomTouche::Lettre(char::from(l.to_ascii_uppercase())))
        }
        _ => None,
    }
}

/// La touche de fonction que nomme `nom`, de `f1` à `f12`, écrite sans zéro en tête.
fn fonction(nom: &str) -> Option<NomTouche> {
    (1..=12).find(|n| nom == format!("f{n}")).map(NomTouche::F)
}

/// La touche virtuelle de `touche` ; `None` pour `Barre` et `Interrogation`, que la disposition
/// place.
pub fn vk(touche: NomTouche) -> Option<u8> {
    let code = match touche {
        NomTouche::Tab => vk::TAB,
        NomTouche::Echap => vk::ECHAP,
        NomTouche::Entree => vk::RETOUR_CHARIOT,
        NomTouche::Espace => vk::ESPACE,
        NomTouche::Retour => vk::RETOUR,
        NomTouche::Suppr => vk::SUPPR,
        NomTouche::Inser => vk::INSER,
        NomTouche::Haut => vk::HAUT,
        NomTouche::Bas => vk::BAS,
        NomTouche::Gauche => vk::GAUCHE,
        NomTouche::Droite => vk::DROITE,
        NomTouche::Debut => vk::DEBUT,
        NomTouche::Fin => vk::FIN,
        NomTouche::PageHaut => vk::PAGE_HAUT,
        NomTouche::PageBas => vk::PAGE_BAS,
        NomTouche::F(n) => vk::F1 + n - 1,
        // La touche virtuelle d'une lettre est son code ASCII majuscule.
        NomTouche::Lettre(l) => return u8::try_from(l).ok(),
        NomTouche::Barre | NomTouche::Interrogation => return None,
    };
    Some(code)
}

/// Le caractère de `touche` : `/` pour `Barre`, `?` pour `Interrogation`, `None` sinon.
pub fn caractere(touche: NomTouche) -> Option<char> {
    match touche {
        NomTouche::Barre => Some('/'),
        NomTouche::Interrogation => Some('?'),
        _ => None,
    }
}

/// L'événement de la touche virtuelle `vk`, enfoncée ou relâchée. `balayage_ex` est ce que rend
/// `MapVirtualKeyExW` (`MAPVK_VK_TO_VSC_EX`) : son octet bas est le code de balayage, et
/// l'événement est étendu quand son octet haut vaut 0xE0, ou quand `vk` est l'une de 0x21 à 0x28,
/// 0x2D ou 0x2E (PageHaut, PageBas, Fin, Debut, les flèches, Inser, Suppr), dont
/// `MapVirtualKeyExW` rend le code du pavé numérique, sans 0xE0 (décision 9).
pub fn evenement(vk: u8, balayage_ex: u16, relachee: bool) -> Evenement {
    let [haut, bas] = balayage_ex.to_be_bytes();
    let etendue = haut == 0xE0 || matches!(vk, vk::PAGE_HAUT..=vk::BAS | vk::INSER | vk::SUPPR);
    Evenement::Touche {
        vk,
        scan: bas,
        etendue,
        relachee,
    }
}

/// Le lot qui enfonce les touches virtuelles `enfoncees` dans l'ordre donné, puis les relâche
/// toutes dans l'ordre inverse. Les lots d'une combinaison (`lot_combinaison`) et d'un caractère
/// (`texte::lot_unicode`, `texte::lot_clavier`) passent par lui : chacun relâche ainsi tout ce
/// qu'il enfonce. Chaque touche prend son balayage de `balayage`, une fois, à l'enfoncement, et
/// son relâchement le reprend.
pub fn lot_touches(enfoncees: &[u8], balayage: &mut dyn FnMut(u8) -> u16) -> Vec<Evenement> {
    let paires: Vec<(u8, u16)> = enfoncees.iter().map(|&v| (v, balayage(v))).collect();
    let mut lot: Vec<Evenement> = paires
        .iter()
        .map(|&(v, b)| evenement(v, b, false))
        .collect();
    lot.extend(paires.iter().rev().map(|&(v, b)| evenement(v, b, true)));
    lot
}

/// Le lot, envoyé d'un seul tenant, qui frappe la combinaison `c`, sa touche étant la touche
/// virtuelle `vk` : Ctrl gauche, Maj gauche et Alt gauche enfoncées dans cet ordre, si `c` les
/// demande, la touche enfoncée puis relâchée, puis les modificateurs relâchés dans l'ordre
/// inverse (`lot_touches`). Chaque touche prend son balayage de `balayage`, une fois.
pub fn lot_combinaison(
    c: &Combinaison,
    vk: u8,
    balayage: &mut dyn FnMut(u8) -> u16,
) -> Vec<Evenement> {
    let enfoncees: Vec<u8> = [
        (c.ctrl, vk::CTRL_G),
        (c.maj, vk::MAJ_G),
        (c.alt, vk::ALT_G),
        (true, vk),
    ]
    .into_iter()
    .filter(|&(demandee, _)| demandee)
    .map(|(_, v)| v)
    .collect();
    lot_touches(&enfoncees, balayage)
}

#[cfg(test)]
pub(crate) mod essai {
    use super::Evenement;

    /// Ce que `MapVirtualKeyExW` (`MAPVK_VK_TO_VSC_EX`) rend sur le poste, disposition française
    /// (KBDFR), relevé en lecture seule : les flèches, Inser, Suppr, Debut, Fin, PageHaut et
    /// PageBas sans 0xE0, seuls Ctrl droite et Alt droite avec ; 0 pour toute autre touche.
    pub(crate) fn balayage_essai(vk: u8) -> u16 {
        match vk {
            0x26 => 0x48,   // Haut
            0x28 => 0x50,   // Bas
            0x25 => 0x4B,   // Gauche
            0x27 => 0x4D,   // Droite
            0x2D => 0x52,   // Inser
            0x2E => 0x53,   // Suppr
            0x24 => 0x47,   // Debut
            0x23 => 0x4F,   // Fin
            0x21 => 0x49,   // PageHaut
            0x22 => 0x51,   // PageBas
            0xA3 => 0xE01D, // Ctrl droite
            0xA5 => 0xE038, // Alt droite
            0x09 => 0x0F,   // Tab
            0x0D => 0x1C,   // Entrée
            0xA0 => 0x2A,   // Maj gauche
            0xA2 => 0x1D,   // Ctrl gauche
            0xA4 => 0x38,   // Alt gauche
            0x30 => 0x0B,   // la touche de « à » et de « 0 »
            0xBF => 0x34,   // la touche de « : » et de « / »
            0x41 => 0x10,   // la touche de « a »
            0x42 => 0x30,   // la touche de « b »
            _ => 0,
        }
    }

    /// Les couples `(vk, relachee)` des événements de touche de `lot`, dans l'ordre.
    pub(crate) fn touches(lot: &[Evenement]) -> Vec<(u8, bool)> {
        lot.iter()
            .filter_map(|e| match *e {
                Evenement::Touche { vk, relachee, .. } => Some((vk, relachee)),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::essai::balayage_essai;
    use super::essai::touches;
    use super::*;

    #[test]
    fn noms_du_design() {
        for (t, v) in [
            ("Tab", 0x09),
            ("Maj+Tab", 0x09),
            ("Echap", 0x1B),
            ("Entree", 0x0D),
            ("Espace", 0x20),
            ("Retour", 0x08),
            ("Suppr", 0x2E),
            ("Haut", 0x26),
            ("Bas", 0x28),
            ("Gauche", 0x25),
            ("Droite", 0x27),
            ("Debut", 0x24),
            ("Fin", 0x23),
            ("PageHaut", 0x21),
            ("PageBas", 0x22),
            ("F1", 0x70),
            ("F12", 0x7B),
            ("Ctrl+A", 0x41),
            ("Ctrl+Z", 0x5A),
            ("Alt+F4", 0x73),
            ("Inser", 0x2D),
        ] {
            assert_eq!(vk(analyser_touche(t).unwrap().touche), Some(v), "{t}");
        }
    }
    #[test]
    fn f13_est_inconnue() {
        assert_eq!(
            analyser_touche("F13"),
            Err(Arret::Usage("touche inconnue : F13".into()))
        );
    }
    #[test]
    fn noms_sans_egard_a_la_casse() {
        assert_eq!(analyser_touche("maj+tab"), analyser_touche("Maj+Tab"));
    }
    #[test]
    fn modificateurs_dans_tout_ordre() {
        assert_eq!(
            analyser_touche("Maj+Ctrl+Tab"),
            analyser_touche("Ctrl+Maj+Tab")
        );
    }
    #[test]
    fn lettre_seule_est_un_usage() {
        assert!(matches!(analyser_touche("a"), Err(Arret::Usage(_))));
    }
    #[test]
    fn touches_interdites_sortent_en_1() {
        for t in [
            "Win",
            "Win+R",
            "Windows",
            "Alt+Tab",
            "alt+maj+tab",
            "Ctrl+Alt+Tab",
            "Ctrl+Alt+Suppr",
            "Alt+Echap",
            "Ctrl+Echap",
            "Ctrl+Maj+Echap",
        ] {
            assert_eq!(
                analyser_touche(t),
                Err(Arret::Refus(format!("touche interdite : {t}"))),
                "{t}"
            );
        }
    }
    #[test]
    fn alt_f4_est_permis() {
        assert!(analyser_touche("Alt+F4").is_ok());
    }
    #[test]
    fn touche_inconnue_sort_en_64() {
        assert_eq!(
            analyser_touche("Bidule"),
            Err(Arret::Usage("touche inconnue : Bidule".into()))
        );
    }
    #[test]
    fn caracteres_nommes_passent_par_la_disposition() {
        let b = analyser_touche("barre").unwrap();
        assert_eq!((vk(b.touche), caractere(b.touche)), (None, Some('/')));
        let i = analyser_touche("interrogation").unwrap();
        assert_eq!((vk(i.touche), caractere(i.touche)), (None, Some('?')));
    }
    #[test]
    fn une_combinaison_part_en_un_seul_envoi() {
        let c = analyser_touche("Ctrl+Maj+Tab").unwrap();
        assert_eq!(
            touches(&lot_combinaison(&c, 0x09, &mut balayage_essai)),
            vec![
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
    fn fleches_et_inser_sont_etendues() {
        for t in ["Haut", "Bas", "Inser", "Suppr", "PageHaut"] {
            let c = analyser_touche(t).unwrap();
            let lot = lot_combinaison(&c, vk(c.touche).unwrap(), &mut balayage_essai);
            assert!(
                matches!(lot[0], Evenement::Touche { etendue: true, .. }),
                "{t}"
            );
        }
        let c = analyser_touche("Tab").unwrap();
        assert!(matches!(
            lot_combinaison(&c, 0x09, &mut balayage_essai)[0],
            Evenement::Touche {
                etendue: false,
                scan: 0x0F,
                ..
            }
        ));
    }
    #[test]
    fn evenement_coupe_le_balayage() {
        assert_eq!(
            evenement(0xA5, 0xE038, false),
            Evenement::Touche {
                vk: 0xA5,
                scan: 0x38,
                etendue: true,
                relachee: false
            }
        );
        assert_eq!(
            evenement(0x26, 0x0048, false),
            Evenement::Touche {
                vk: 0x26,
                scan: 0x48,
                etendue: true,
                relachee: false
            }
        );
        assert_eq!(
            evenement(0x21, 0x0049, true),
            Evenement::Touche {
                vk: 0x21,
                scan: 0x49,
                etendue: true,
                relachee: true
            }
        );
        assert_eq!(
            evenement(0x2E, 0x0053, false),
            Evenement::Touche {
                vk: 0x2E,
                scan: 0x53,
                etendue: true,
                relachee: false
            }
        );
        assert_eq!(
            evenement(0x20, 0x0039, false),
            Evenement::Touche {
                vk: 0x20,
                scan: 0x39,
                etendue: false,
                relachee: false
            }
        );
        assert_eq!(
            evenement(0x29, 0x0000, false),
            Evenement::Touche {
                vk: 0x29,
                scan: 0,
                etendue: false,
                relachee: false
            }
        );
    }
    #[test]
    fn chaque_lot_relache_ce_qu_il_enfonce() {
        for t in [
            "Tab",
            "Maj+Tab",
            "Ctrl+A",
            "Ctrl+Maj+Tab",
            "Alt+F4",
            "Ctrl+Alt+Haut",
        ] {
            let c = analyser_touche(t).unwrap();
            let lot = touches(&lot_combinaison(
                &c,
                vk(c.touche).unwrap(),
                &mut balayage_essai,
            ));
            let mut enfoncees = Vec::new();
            for (v, relachee) in lot {
                if relachee {
                    assert_eq!(enfoncees.pop(), Some(v), "{t}")
                } else {
                    enfoncees.push(v)
                }
            }
            assert!(enfoncees.is_empty(), "{t}");
        }
    }

    // Le texte reçu entre échappé dans les deux messages : il tient en une ligne (décision 6).
    #[test]
    fn touche_recue_echappee() {
        assert_eq!(
            analyser_touche("a\nb"),
            Err(Arret::Usage(r"touche inconnue : a\nb".into()))
        );
        assert_eq!(
            analyser_touche("Win+a\nb"),
            Err(Arret::Refus(r"touche interdite : Win+a\nb".into()))
        );
    }

    // `barre` et `interrogation` s'écrivent seules : la disposition place le caractère avec ses
    // propres modificateurs, et aucun lot ne frapperait un modificateur demandé en plus.
    #[test]
    fn caractere_nomme_refuse_un_modificateur() {
        for t in ["Ctrl+barre", "Maj+interrogation", "alt+Barre"] {
            assert_eq!(
                analyser_touche(t),
                Err(Arret::Usage(format!("touche inconnue : {t}"))),
                "{t}"
            );
        }
    }

    // La forme analysée entière : chaque modificateur à sa place, la lettre en majuscule.
    #[test]
    fn analyse_rend_la_combinaison() {
        assert_eq!(
            analyser_touche("Ctrl+Maj+Alt+F4"),
            Ok(Combinaison {
                ctrl: true,
                maj: true,
                alt: true,
                touche: NomTouche::F(4)
            })
        );
        assert_eq!(
            analyser_touche("Tab"),
            Ok(Combinaison {
                ctrl: false,
                maj: false,
                alt: false,
                touche: NomTouche::Tab
            })
        );
        let ctrl_a = Ok(Combinaison {
            ctrl: true,
            maj: false,
            alt: false,
            touche: NomTouche::Lettre('A'),
        });
        assert_eq!(analyser_touche("ctrl+a"), ctrl_a);
        assert_eq!(analyser_touche("CTRL+A"), ctrl_a);
    }

    // Chaque lettre et chaque touche de fonction, par sa touche virtuelle.
    #[test]
    fn lettres_et_fonctions_par_leur_code() {
        for (i, l) in ('A'..='Z').enumerate() {
            let c = analyser_touche(&format!("Ctrl+{l}")).unwrap();
            assert_eq!(c.touche, NomTouche::Lettre(l), "{l}");
            assert_eq!(vk(c.touche), Some(0x41 + i as u8), "{l}");
        }
        for n in 1..=12u8 {
            let c = analyser_touche(&format!("F{n}")).unwrap();
            assert_eq!(c.touche, NomTouche::F(n), "F{n}");
            assert_eq!(vk(c.touche), Some(0x6F + n), "F{n}");
        }
    }

    // Une lettre n'est admise qu'avec Ctrl, même accompagnée d'un autre modificateur.
    #[test]
    fn lettre_sans_ctrl_est_inconnue() {
        for t in ["Maj+A", "Alt+A", "Maj+Alt+A"] {
            assert_eq!(
                analyser_touche(t),
                Err(Arret::Usage(format!("touche inconnue : {t}"))),
                "{t}"
            );
        }
        assert!(analyser_touche("Ctrl+Maj+A").is_ok());
        assert!(analyser_touche("Ctrl+Alt+A").is_ok());
    }

    // Hors des noms du design : ni chiffre, ni lettre accentuée, ni deux lettres, ni F0 ou F01,
    // ni morceau vide, ni modificateur seul, ni touche avant ses modificateurs. La casse ne se
    // replie qu'en ASCII : le signe kelvin (U+212A) ne vaut pas « k ».
    #[test]
    fn hors_des_noms_tout_est_inconnu() {
        for t in [
            "Ctrl+1",
            "Ctrl+é",
            "Ctrl+AB",
            "F0",
            "F01",
            "",
            "+",
            "Ctrl+",
            "Ctrl",
            "Maj+Alt",
            "Tab+Ctrl",
            "Ctrl+Tab+A",
            "Ctrl + A",
            "Ctrl+\u{212A}",
        ] {
            assert_eq!(
                analyser_touche(t),
                Err(Arret::Usage(format!("touche inconnue : {t}"))),
                "{t:?}"
            );
        }
    }

    // Une combinaison interdite se reconnaît sur ses noms, dans tout ordre et en tout morceau,
    // avant tout nom inconnu.
    #[test]
    fn interdites_dans_tout_ordre_et_avant_l_inconnu() {
        for t in [
            "Tab+Alt",
            "Echap+Ctrl",
            "Suppr+Alt+Ctrl",
            "Ctrl+Alt+Maj+Suppr",
            "WINDOWS+r",
            "Bidule+Win",
            "Alt+Tab+Bidule",
            "Maj+Win",
        ] {
            assert_eq!(
                analyser_touche(t),
                Err(Arret::Refus(format!("touche interdite : {t}"))),
                "{t}"
            );
        }
    }

    // Les voisines des interdites restent permises : Suppr sans Ctrl et Alt ensemble, Echap sans
    // Ctrl ni Alt, Tab sans Alt.
    #[test]
    fn voisines_des_interdites_sont_permises() {
        for t in [
            "Ctrl+Suppr",
            "Alt+Suppr",
            "Maj+Echap",
            "Echap",
            "Ctrl+Tab",
            "Maj+Tab",
            "Ctrl+Maj+Tab",
        ] {
            assert!(analyser_touche(t).is_ok(), "{t}");
        }
    }

    // Les autres touches ont une touche virtuelle et pas de caractère.
    #[test]
    fn les_autres_touches_n_ont_pas_de_caractere() {
        for t in ["Tab", "F1", "Ctrl+A", "Espace"] {
            let c = analyser_touche(t).unwrap();
            assert_eq!(caractere(c.touche), None, "{t}");
        }
    }

    // Chaque touche virtuelle que le plan fixe, par sa valeur.
    #[test]
    fn touches_virtuelles_par_leur_valeur() {
        assert_eq!(
            [
                vk::RETOUR_CHARIOT,
                vk::TAB,
                vk::ECHAP,
                vk::ESPACE,
                vk::RETOUR,
                vk::SUPPR,
                vk::INSER,
                vk::DEBUT,
                vk::FIN,
                vk::PAGE_HAUT,
                vk::PAGE_BAS,
                vk::GAUCHE,
                vk::HAUT,
                vk::DROITE,
                vk::BAS,
                vk::F1,
                vk::A
            ],
            [
                0x0D, 0x09, 0x1B, 0x20, 0x08, 0x2E, 0x2D, 0x24, 0x23, 0x21, 0x22, 0x25, 0x26, 0x27,
                0x28, 0x70, 0x41
            ]
        );
        assert_eq!(
            [
                vk::MAJ,
                vk::CTRL,
                vk::ALT,
                vk::MAJ_G,
                vk::MAJ_D,
                vk::CTRL_G,
                vk::CTRL_D,
                vk::ALT_G,
                vk::ALT_D,
                vk::WIN_G,
                vk::WIN_D,
                vk::VERR_MAJ,
                vk::SANS_EFFET
            ],
            [
                0x10, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C, 0x14, 0xE8
            ]
        );
        assert_eq!(
            vk::MODIFICATEURS,
            [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C]
        );
    }

    // Étendue : chaque touche virtuelle de 0x21 à 0x28, 0x2D et 0x2E, quel que soit le balayage ;
    // ailleurs, le seul octet haut 0xE0, jamais 0xE1. Le code garde l'octet bas.
    #[test]
    fn etendue_par_la_touche_ou_l_octet_haut() {
        for v in [0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E] {
            assert_eq!(
                evenement(v, 0x0000, false),
                Evenement::Touche {
                    vk: v,
                    scan: 0,
                    etendue: true,
                    relachee: false
                },
                "{v:#X}"
            );
        }
        for v in [0x20, 0x29, 0x2C, 0x2F, 0xA3] {
            assert_eq!(
                evenement(v, 0x001D, true),
                Evenement::Touche {
                    vk: v,
                    scan: 0x1D,
                    etendue: false,
                    relachee: true
                },
                "{v:#X}"
            );
        }
        assert_eq!(
            evenement(0xA3, 0xE01D, false),
            Evenement::Touche {
                vk: 0xA3,
                scan: 0x1D,
                etendue: true,
                relachee: false
            }
        );
        assert_eq!(
            evenement(0x13, 0xE11D, false),
            Evenement::Touche {
                vk: 0x13,
                scan: 0x1D,
                etendue: false,
                relachee: false
            }
        );
    }

    // Le lot entier, par ses valeurs : Ctrl gauche, Maj gauche, Alt gauche, la touche, puis les
    // relâchements dans l'ordre inverse, chacun avec son balayage et son extension.
    #[test]
    fn lot_complet_par_ses_valeurs() {
        let c = analyser_touche("Alt+Maj+Ctrl+Haut").unwrap();
        let t = |vk, scan, etendue, relachee| Evenement::Touche {
            vk,
            scan,
            etendue,
            relachee,
        };
        assert_eq!(
            lot_combinaison(&c, vk::HAUT, &mut balayage_essai),
            vec![
                t(0xA2, 0x1D, false, false),
                t(0xA0, 0x2A, false, false),
                t(0xA4, 0x38, false, false),
                t(0x26, 0x48, true, false),
                t(0x26, 0x48, true, true),
                t(0xA4, 0x38, false, true),
                t(0xA0, 0x2A, false, true),
                t(0xA2, 0x1D, false, true),
            ]
        );
        let c = analyser_touche("Entree").unwrap();
        assert_eq!(
            lot_combinaison(&c, vk::RETOUR_CHARIOT, &mut balayage_essai),
            vec![t(0x0D, 0x1C, false, false), t(0x0D, 0x1C, false, true)]
        );
    }

    // Le lot d'une liste de touches, par ses valeurs : les touches enfoncées dans l'ordre donné,
    // qui n'est pas celui de leurs codes, puis relâchées dans l'ordre inverse, chacune avec son
    // balayage et son extension. Le balayage se lit une fois par touche, à l'enfoncement, et le
    // relâchement le reprend.
    #[test]
    fn lot_touches_par_ses_valeurs() {
        let t = |vk, scan, etendue, relachee| Evenement::Touche {
            vk,
            scan,
            etendue,
            relachee,
        };
        let mut lus = Vec::new();
        let mut note = |v| {
            lus.push(v);
            balayage_essai(v)
        };
        assert_eq!(
            lot_touches(&[vk::MAJ_G, vk::ALT_D, vk::HAUT], &mut note),
            vec![
                t(0xA0, 0x2A, false, false),
                t(0xA5, 0x38, true, false),
                t(0x26, 0x48, true, false),
                t(0x26, 0x48, true, true),
                t(0xA5, 0x38, true, true),
                t(0xA0, 0x2A, false, true),
            ]
        );
        assert_eq!(lus, [0xA0, 0xA5, 0x26]);
    }
}
