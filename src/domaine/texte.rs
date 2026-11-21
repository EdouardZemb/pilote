//! Les textes de `taper` : leur décodage, quelle que soit leur source, et leur borne (décision 5) ;
//! leur découpe en unités ; et les deux façons de les frapper. Sans la disposition, chaque
//! caractère part en unités UTF-16 (`KEYEVENTF_UNICODE`), Entrée et Tab par leurs touches. Par la
//! disposition du clavier (`--comme-au-clavier`, `barre`, `interrogation`), chaque caractère part
//! par la touche et les modificateurs que la disposition lui donne, et refuse quand elle ne le
//! produit pas exactement (décision 9). Chaque lot relâche ce qu'il enfonce, dans l'ordre inverse.

use crate::domaine::sortie::Arret;
use crate::domaine::sortie::echapper;
use crate::domaine::touches::Evenement;
use crate::domaine::touches::lot_touches;
use crate::domaine::touches::vk;

/// Le plus grand nombre de caractères qu'un texte de `taper` peut porter.
pub const LONGUEUR_MAX: usize = 10_000;

/// Le message d'un texte qui ne se décode pas.
const ILLISIBLE: &str = "texte illisible : ni UTF-8 ni UTF-16 à BOM";

/// Le texte de `taper` lu dans un fichier ou sur l'entrée standard : de l'UTF-16LE s'il commence
/// par la BOM FF FE, sinon de l'UTF-8, sa BOM EF BB BF retirée s'il la porte. Un octet qui ne se
/// décode pas, un nombre impair d'octets en UTF-16, une moitié orpheline de paire de substitution
/// sortent en 64, « texte illisible : ni UTF-8 ni UTF-16 à BOM ». Un seul saut de ligne final est
/// retiré, celui qu'ajoutent le shell ou l'éditeur : CRLF, LF ou CR seul, les trois formes que
/// `unites` frappe en Entrée. Le texte est ensuite borné (`borner`).
pub fn decoder(octets: &[u8]) -> Result<String, Arret> {
    let texte = match octets.strip_prefix(&[0xFF, 0xFE]) {
        Some(reste) => utf16le(reste),
        None => {
            let reste = octets.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(octets);
            str::from_utf8(reste).ok().map(String::from)
        }
    };
    let texte = texte.ok_or_else(|| Arret::Usage(ILLISIBLE.into()))?;
    borner(sans_saut_final(texte))
}

/// `octets` lus en UTF-16LE, ou `None` s'ils sont en nombre impair ou portent une moitié
/// orpheline de paire de substitution.
fn utf16le(octets: &[u8]) -> Option<String> {
    let (paires, reste) = octets.as_chunks::<2>();
    if !reste.is_empty() {
        return None;
    }
    char::decode_utf16(paires.iter().map(|&p| u16::from_le_bytes(p)))
        .collect::<Result<String, _>>()
        .ok()
}

/// `texte` sans son saut de ligne final, s'il en porte un : CRLF, sinon LF, sinon CR.
fn sans_saut_final(mut texte: String) -> String {
    let garde = ["\r\n", "\n", "\r"]
        .iter()
        .find_map(|fin| texte.strip_suffix(fin))
        .map(str::len);
    if let Some(longueur) = garde {
        texte.truncate(longueur);
    }
    texte
}

/// `texte`, s'il porte au plus `LONGUEUR_MAX` caractères (des valeurs scalaires d'Unicode, non
/// des octets) ; au-delà, 64, « texte trop long : <n> caractères, 10000 au plus ».
pub fn borner(texte: String) -> Result<String, Arret> {
    let n = texte.chars().count();
    if n > LONGUEUR_MAX {
        return Err(Arret::Usage(format!(
            "texte trop long : {n} caractères, {LONGUEUR_MAX} au plus"
        )));
    }
    Ok(texte)
}

/// Une unité de frappe : un caractère, Entrée ou Tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unite {
    Caractere(char),
    Entree,
    Tab,
}

/// Les unités de `texte`, dans l'ordre : CRLF, CR seul et LF seul donnent chacun une Entrée, la
/// tabulation Tab, et tout autre caractère part tel quel, sans normalisation : une lettre
/// décomposée reste deux caractères.
pub fn unites(texte: &str) -> Vec<Unite> {
    let mut sortie = Vec::new();
    let mut caracteres = texte.chars().peekable();
    while let Some(c) = caracteres.next() {
        sortie.push(match c {
            '\r' => {
                caracteres.next_if_eq(&'\n');
                Unite::Entree
            }
            '\n' => Unite::Entree,
            '\t' => Unite::Tab,
            c => Unite::Caractere(c),
        });
    }
    sortie
}

/// Le lot d'une unité frappée sans la disposition : Entrée et Tab par leurs touches virtuelles,
/// chacune avec son balayage, lu une fois dans `balayage` ; un caractère par ses unités UTF-16
/// (`KEYEVENTF_UNICODE`), chacune enfoncée puis relâchée, les deux moitiés d'une paire de
/// substitution dans le même lot.
pub fn lot_unicode(u: Unite, balayage: &mut dyn FnMut(u8) -> u16) -> Vec<Evenement> {
    let c = match u {
        Unite::Entree => return lot_touches(&[vk::RETOUR_CHARIOT], balayage),
        Unite::Tab => return lot_touches(&[vk::TAB], balayage),
        Unite::Caractere(c) => c,
    };
    c.encode_utf16(&mut [0; 2])
        .iter()
        .flat_map(|&unite| [false, true].map(|relachee| Evenement::Unicode { unite, relachee }))
        .collect()
}

/// Ce que `ToUnicodeEx` produit d'une touche dans un état : une touche morte, ou un texte, vide
/// quand la touche ne produit rien.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Produit {
    Morte,
    Texte(String),
}

/// La disposition du clavier du fil focalisé (décision 2), que la couche Windows lit.
pub trait Disposition {
    /// `VkKeyScanExW` : l'état des modificateurs (octet haut) et la touche virtuelle (octet bas)
    /// qui donnent l'unité UTF-16 `unite` ; -1 si aucune touche ne la donne.
    fn scan(&mut self, unite: u16) -> i16;
    /// `ToUnicodeEx`, l'état du clavier intact : ce que produit la touche virtuelle `vk` sous
    /// l'état `etat`, codé comme l'octet haut de `scan`.
    fn produit(&mut self, vk: u8, etat: u8) -> Produit;
    /// `MapVirtualKeyExW` (`MAPVK_VK_TO_VSC_EX`) : le code de balayage de la touche virtuelle `vk`.
    fn balayage(&mut self, vk: u8) -> u16;
}

/// Le lot qui frappe `c` par la disposition du clavier du fil focalisé (décision 9). `\n` et `\t`
/// donnent Entrée et Tab, sans modificateur et sans rien demander à la disposition que leurs
/// balayages. Tout autre caractère prend la touche et l'état que rend `scan` : l'état 0 frappe la
/// touche seule, l'état 1 Maj gauche, l'état 6 (AltGr) Ctrl gauche puis Alt droite, l'état 7 Maj
/// gauche, Ctrl gauche puis Alt droite ; puis la touche, puis les relâchements dans l'ordre
/// inverse (`touches::lot_touches`). Chaque touche lit son balayage une fois.
///
/// Il refuse en 1, « caractère absent du clavier : « <c> » », un caractère hors du plan de base,
/// que `scan` ne lit pas, puis tout autre état : -1 (aucune touche), Ctrl sans Alt (2, 3), Alt
/// sans Ctrl (4, 5), l'un des bits 8, 16 ou 32, et les bits 64 et 128, que Windows ne définit
/// pas ; puis un produit autre que le seul caractère `c`. Une touche morte refuse en 1, « touche
/// morte : « <c> » ». Rien ne part ainsi que la disposition ne produise exactement. `<c>` est
/// échappé.
pub fn lot_clavier(c: char, d: &mut dyn Disposition) -> Result<Vec<Evenement>, Arret> {
    let enfoncees = match c {
        '\n' => vec![vk::RETOUR_CHARIOT],
        '\t' => vec![vk::TAB],
        _ => par_la_disposition(c, d)?,
    };
    Ok(lot_touches(&enfoncees, &mut |v| d.balayage(v)))
}

/// Les touches virtuelles qui frappent `c` par la disposition `d`, modificateurs d'abord, dans
/// l'ordre où elles s'enfoncent ; ou le refus de `lot_clavier`.
fn par_la_disposition(c: char, d: &mut dyn Disposition) -> Result<Vec<u8>, Arret> {
    let affiche = echapper(c.encode_utf8(&mut [0; 4]));
    let absent = || Arret::Refus(format!("caractère absent du clavier : « {affiche} »"));
    let unite = u16::try_from(u32::from(c)).map_err(|_| absent())?;
    let [etat, touche] = d.scan(unite).to_be_bytes();
    let modificateurs: &[u8] = match etat {
        0 => &[],
        1 => &[vk::MAJ_G],
        6 => &[vk::CTRL_G, vk::ALT_D],
        7 => &[vk::MAJ_G, vk::CTRL_G, vk::ALT_D],
        _ => return Err(absent()),
    };
    match d.produit(touche, etat) {
        Produit::Morte => Err(Arret::Refus(format!("touche morte : « {affiche} »"))),
        Produit::Texte(t) if t.chars().eq([c]) => Ok([modificateurs, &[touche]].concat()),
        Produit::Texte(_) => Err(absent()),
    }
}

#[cfg(test)]
pub(crate) mod essai {
    use super::Disposition;
    use super::Produit;
    use crate::domaine::touches::essai::balayage_essai;

    /// Ce que `VkKeyScanExW` rend : l'AZERTY du poste, relevé en lecture seule, puis des valeurs
    /// d'essai qu'aucun AZERTY ne porte, sur la touche 0x42 : l'état 7, que `lot_clavier` frappe,
    /// puis les états 3, 4 et 5 et les bits 8 à 128, qu'il refuse.
    const SCANS: [(char, i16); 20] = [
        ('@', 0x0630),
        ('/', 0x01BF),
        ('?', 0x01BC),
        ('a', 0x0041),
        ('A', 0x0141),
        ('0', 0x0130),
        ('\n', 0x020D),
        ('\t', 0x0009),
        ('\u{7}', 0x0247),
        ('ñ', -1),
        ('^', 0x00DD),
        ('±', 0x0342),
        ('¬', 0x0442),
        ('÷', 0x0542),
        ('ß', 0x0742),
        ('¤', 0x0842),
        ('¶', 0x1042),
        ('§', 0x2042),
        ('µ', 0x4042),
        ('·', i16::from_be_bytes([0x80, 0x42])),
    ];

    /// Une disposition d'essai. `scan` lit `SCANS`, -1 pour tout autre caractère ; `produit(0xDD,
    /// 0)`, l'accent circonflexe, est une touche morte ; `produit(0x41, 0)` vaut `produit_de_a`,
    /// « a » par défaut ; tout autre produit est le caractère que `scan` sert pour cette touche et
    /// cet état, et rien sinon ; `balayage` est `balayage_essai`.
    #[derive(Clone, Debug)]
    pub(crate) struct DispositionEssai {
        pub(crate) produit_de_a: &'static str,
    }

    impl DispositionEssai {
        pub(crate) fn new() -> DispositionEssai {
            DispositionEssai { produit_de_a: "a" }
        }
    }

    impl Disposition for DispositionEssai {
        fn scan(&mut self, unite: u16) -> i16 {
            SCANS
                .iter()
                .find(|(c, _)| u32::from(*c) == u32::from(unite))
                .map_or(-1, |&(_, r)| r)
        }
        fn produit(&mut self, vk: u8, etat: u8) -> Produit {
            match (vk, etat) {
                (0xDD, 0) => Produit::Morte,
                (0x41, 0) => Produit::Texte(self.produit_de_a.into()),
                _ => Produit::Texte(
                    SCANS
                        .iter()
                        .find(|(_, r)| r.to_be_bytes() == [etat, vk])
                        .map_or_else(String::new, |(c, _)| c.to_string()),
                ),
            }
        }
        fn balayage(&mut self, vk: u8) -> u16 {
            balayage_essai(vk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::essai::DispositionEssai;
    use super::*;
    use crate::domaine::touches::essai::balayage_essai;
    use crate::domaine::touches::essai::touches;

    #[test]
    fn fichier_utf8_sans_bom() {
        assert_eq!(decoder("été".as_bytes()).unwrap(), "été");
    }
    #[test]
    fn fichier_utf8_a_bom() {
        assert_eq!(decoder(b"\xEF\xBB\xBFab").unwrap(), "ab");
    }
    #[test]
    fn fichier_utf16_a_bom() {
        assert_eq!(decoder(&[0xFF, 0xFE, b'a', 0, 0xE9, 0]).unwrap(), "aé");
    }
    #[test]
    fn texte_illisible_sort_en_64() {
        assert_eq!(
            decoder(&[0xC3, 0x28]),
            Err(Arret::Usage(
                "texte illisible : ni UTF-8 ni UTF-16 à BOM".into()
            ))
        );
        assert_eq!(
            decoder(&[0xFF, 0xFE, 0x3D, 0xD8]),
            Err(Arret::Usage(
                "texte illisible : ni UTF-8 ni UTF-16 à BOM".into()
            ))
        );
    }
    #[test]
    fn un_seul_saut_de_ligne_final_ignore() {
        assert_eq!(decoder(b"a\n\n").unwrap(), "a\n");
        assert_eq!(decoder(b"a\r\n").unwrap(), "a");
        assert_eq!(decoder(b"a").unwrap(), "a");
    }
    #[test]
    fn texte_de_10000_caracteres_passe_10001_refuse() {
        assert!(decoder("é".repeat(10_000).as_bytes()).is_ok());
        assert_eq!(
            decoder("é".repeat(10_001).as_bytes()),
            Err(Arret::Usage(
                "texte trop long : 10001 caractères, 10000 au plus".into()
            ))
        );
        assert_eq!(
            borner("x".repeat(10_001)),
            Err(Arret::Usage(
                "texte trop long : 10001 caractères, 10000 au plus".into()
            ))
        );
    }
    #[test]
    fn taper_crlf_donne_un_seul_entree() {
        assert_eq!(
            unites("a\r\nb\rc\nd"),
            vec![
                Unite::Caractere('a'),
                Unite::Entree,
                Unite::Caractere('b'),
                Unite::Entree,
                Unite::Caractere('c'),
                Unite::Entree,
                Unite::Caractere('d')
            ]
        );
    }
    #[test]
    fn tabulation_devient_tab() {
        assert_eq!(unites("\t"), vec![Unite::Tab]);
    }
    #[test]
    fn entree_par_sa_touche() {
        assert_eq!(
            touches(&lot_unicode(Unite::Entree, &mut balayage_essai)),
            vec![(0x0D, false), (0x0D, true)]
        );
    }
    #[test]
    fn taper_hors_du_plan_de_base_envoie_la_paire() {
        assert_eq!(
            lot_unicode(Unite::Caractere('😀'), &mut balayage_essai),
            vec![
                Evenement::Unicode {
                    unite: 0xD83D,
                    relachee: false
                },
                Evenement::Unicode {
                    unite: 0xD83D,
                    relachee: true
                },
                Evenement::Unicode {
                    unite: 0xDE00,
                    relachee: false
                },
                Evenement::Unicode {
                    unite: 0xDE00,
                    relachee: true
                }
            ]
        );
    }
    #[test]
    fn lettre_decomposee_part_telle_quelle() {
        assert_eq!(
            unites("e\u{301}"),
            vec![Unite::Caractere('e'), Unite::Caractere('\u{301}')]
        );
    }
    #[test]
    fn entree_et_tab_sans_modificateur() {
        assert_eq!(
            touches(&lot_clavier('\n', &mut DispositionEssai::new()).unwrap()),
            vec![(0x0D, false), (0x0D, true)]
        );
        assert_eq!(
            touches(&lot_clavier('\t', &mut DispositionEssai::new()).unwrap()),
            vec![(0x09, false), (0x09, true)]
        );
    }
    #[test]
    fn comme_au_clavier_lettre_simple() {
        assert_eq!(
            touches(&lot_clavier('a', &mut DispositionEssai::new()).unwrap()),
            vec![(0x41, false), (0x41, true)]
        );
    }
    #[test]
    fn comme_au_clavier_majuscule_par_maj() {
        assert_eq!(
            touches(&lot_clavier('A', &mut DispositionEssai::new()).unwrap()),
            vec![(0xA0, false), (0x41, false), (0x41, true), (0xA0, true)]
        );
    }
    #[test]
    fn comme_au_clavier_etat_7_par_maj_et_altgr() {
        assert_eq!(
            touches(&lot_clavier('ß', &mut DispositionEssai::new()).unwrap()),
            vec![
                (0xA0, false),
                (0xA2, false),
                (0xA5, false),
                (0x42, false),
                (0x42, true),
                (0xA5, true),
                (0xA2, true),
                (0xA0, true)
            ]
        );
    }
    #[test]
    fn comme_au_clavier_refuse_alt_sans_ctrl() {
        for c in ['¬', '÷'] {
            assert_eq!(
                lot_clavier(c, &mut DispositionEssai::new()),
                Err(Arret::Refus(format!(
                    "caractère absent du clavier : « {c} »"
                )))
            );
        }
    }
    #[test]
    fn taper_arobase_par_altgr() {
        let l = lot_clavier('@', &mut DispositionEssai::new()).unwrap();
        assert_eq!(
            touches(&l),
            vec![
                (0xA2, false),
                (0xA5, false),
                (0x30, false),
                (0x30, true),
                (0xA5, true),
                (0xA2, true)
            ]
        );
        assert_eq!(
            l[1],
            Evenement::Touche {
                vk: 0xA5,
                scan: 0x38,
                etendue: true,
                relachee: false
            }
        );
    }
    #[test]
    fn barre_par_maj_et_deux_points() {
        assert_eq!(
            touches(&lot_clavier('/', &mut DispositionEssai::new()).unwrap()),
            vec![(0xA0, false), (0xBF, false), (0xBF, true), (0xA0, true)]
        );
    }
    #[test]
    fn comme_au_clavier_refuse_un_caractere_absent() {
        assert_eq!(
            lot_clavier('ñ', &mut DispositionEssai::new()),
            Err(Arret::Refus("caractère absent du clavier : « ñ »".into()))
        );
    }
    #[test]
    fn comme_au_clavier_refuse_hors_du_plan_de_base() {
        assert_eq!(
            lot_clavier('😀', &mut DispositionEssai::new()),
            Err(Arret::Refus("caractère absent du clavier : « 😀 »".into()))
        );
    }
    #[test]
    fn comme_au_clavier_refuse_ctrl_sans_alt() {
        assert_eq!(
            lot_clavier('\u{7}', &mut DispositionEssai::new()),
            Err(Arret::Refus(
                r"caractère absent du clavier : « \u{0007} »".into()
            ))
        );
    }
    #[test]
    fn comme_au_clavier_refuse_les_bits_8_16_32() {
        for c in ['¤', '¶', '§'] {
            assert_eq!(
                lot_clavier(c, &mut DispositionEssai::new()),
                Err(Arret::Refus(format!(
                    "caractère absent du clavier : « {c} »"
                )))
            );
        }
    }
    #[test]
    fn comme_au_clavier_refuse_une_touche_morte() {
        assert_eq!(
            lot_clavier('^', &mut DispositionEssai::new()),
            Err(Arret::Refus("touche morte : « ^ »".into()))
        );
    }
    #[test]
    fn comme_au_clavier_refuse_un_produit_different() {
        let mut d = DispositionEssai::new();
        d.produit_de_a = "x";
        assert_eq!(
            lot_clavier('a', &mut d),
            Err(Arret::Refus("caractère absent du clavier : « a »".into()))
        );
    }

    /// Une disposition d'essai qui note ce qu'on lui demande : si `scan` ou `produit` ont servi,
    /// et chaque touche dont le balayage a été lu, dans l'ordre.
    struct Notee {
        essai: DispositionEssai,
        consultee: bool,
        balayages: Vec<u8>,
    }

    impl Notee {
        fn new() -> Notee {
            Notee {
                essai: DispositionEssai::new(),
                consultee: false,
                balayages: Vec::new(),
            }
        }
    }

    impl Disposition for Notee {
        fn scan(&mut self, unite: u16) -> i16 {
            self.consultee = true;
            self.essai.scan(unite)
        }
        fn produit(&mut self, vk: u8, etat: u8) -> Produit {
            self.consultee = true;
            self.essai.produit(vk, etat)
        }
        fn balayage(&mut self, vk: u8) -> u16 {
            self.balayages.push(vk);
            self.essai.balayage(vk)
        }
    }

    /// L'événement d'une touche non étendue.
    fn touche(vk: u8, scan: u8, relachee: bool) -> Evenement {
        Evenement::Touche {
            vk,
            scan,
            etendue: false,
            relachee,
        }
    }

    // L'UTF-16 se lit par paires d'octets : un octet de trop est illisible. Une BOM d'UTF-8
    // suivie d'octets faux aussi.
    #[test]
    fn utf16_impair_ou_utf8_faux_apres_bom_illisible() {
        let illisible = Err(Arret::Usage(
            "texte illisible : ni UTF-8 ni UTF-16 à BOM".into(),
        ));
        assert_eq!(decoder(&[0xFF, 0xFE, b'a', 0, b'b']), illisible);
        assert_eq!(decoder(b"\xEF\xBB\xBF\xC3\x28"), illisible);
        assert_eq!(
            decoder(&[0xFF, 0xFE, 0x3D, 0xD8, 0x00, 0xDE]).unwrap(),
            "😀"
        );
    }

    // Un retour chariot seul est un saut de ligne, comme pour `unites` : final, il est retiré, et
    // un seul saut l'est, quelle que soit sa forme.
    #[test]
    fn retour_chariot_final_seul_ignore() {
        assert_eq!(decoder(b"a\r").unwrap(), "a");
        assert_eq!(decoder(b"a\n\r").unwrap(), "a\n");
        assert_eq!(decoder(b"a\r\r\n").unwrap(), "a\r");
        assert_eq!(
            decoder(&[0xFF, 0xFE, b'a', 0, b'\r', 0, b'\n', 0]).unwrap(),
            "a"
        );
    }

    // La longueur se compte après le retrait du saut de ligne final.
    #[test]
    fn longueur_comptee_sans_le_saut_final() {
        let texte = format!("{}\r\n", "x".repeat(10_000));
        assert_eq!(decoder(texte.as_bytes()).map(|t| t.len()), Ok(10_000));
        assert_eq!(LONGUEUR_MAX, 10_000);
    }

    // Entrée et Tab partent par leur touche et leur balayage, lu une fois par touche.
    #[test]
    fn entree_et_tab_par_leur_touche_et_leur_balayage() {
        let mut lus = Vec::new();
        let mut note = |v| {
            lus.push(v);
            balayage_essai(v)
        };
        assert_eq!(
            lot_unicode(Unite::Entree, &mut note),
            vec![touche(0x0D, 0x1C, false), touche(0x0D, 0x1C, true)]
        );
        assert_eq!(
            lot_unicode(Unite::Tab, &mut note),
            vec![touche(0x09, 0x0F, false), touche(0x09, 0x0F, true)]
        );
        assert_eq!(lus, [0x0D, 0x09]);
    }

    // Un caractère du plan de base part en une seule unité UTF-16, enfoncée puis relâchée, sans
    // touche virtuelle.
    #[test]
    fn caractere_du_plan_de_base_en_une_unite() {
        assert_eq!(
            lot_unicode(Unite::Caractere('é'), &mut balayage_essai),
            vec![
                Evenement::Unicode {
                    unite: 0xE9,
                    relachee: false
                },
                Evenement::Unicode {
                    unite: 0xE9,
                    relachee: true
                }
            ]
        );
    }

    // Par la disposition, `\n` et `\t` ne lui demandent rien que leurs balayages ; un autre
    // caractère la consulte, et lit chaque balayage une fois.
    #[test]
    fn entree_et_tab_ne_consultent_pas_la_disposition() {
        for (c, v, scan) in [('\n', 0x0D, 0x1C), ('\t', 0x09, 0x0F)] {
            let mut d = Notee::new();
            assert_eq!(
                lot_clavier(c, &mut d),
                Ok(vec![touche(v, scan, false), touche(v, scan, true)])
            );
            assert!(!d.consultee, "{c:?}");
            assert_eq!(d.balayages, [v], "{c:?}");
        }
        let mut d = Notee::new();
        assert!(lot_clavier('ß', &mut d).is_ok());
        assert!(d.consultee);
        assert_eq!(d.balayages, [0xA0, 0xA2, 0xA5, 0x42]);
    }

    // Le lot d'un caractère par la disposition, par ses valeurs : chaque touche avec son balayage
    // et son extension.
    #[test]
    fn lot_clavier_par_ses_valeurs() {
        assert_eq!(
            lot_clavier('A', &mut DispositionEssai::new()),
            Ok(vec![
                touche(0xA0, 0x2A, false),
                touche(0x41, 0x10, false),
                touche(0x41, 0x10, true),
                touche(0xA0, 0x2A, true)
            ])
        );
        assert_eq!(
            lot_clavier('0', &mut DispositionEssai::new()),
            Ok(vec![
                touche(0xA0, 0x2A, false),
                touche(0x30, 0x0B, false),
                touche(0x30, 0x0B, true),
                touche(0xA0, 0x2A, true)
            ])
        );
        assert_eq!(
            lot_clavier('?', &mut DispositionEssai::new()).map(|l| touches(&l)),
            Ok(vec![
                (0xA0, false),
                (0xBC, false),
                (0xBC, true),
                (0xA0, true)
            ])
        );
    }

    // L'état 3 porte Ctrl sans Alt ; les bits 64 et 128, que Windows ne définit pas, refusent
    // aussi : seuls les états 0, 1, 6 et 7 frappent.
    #[test]
    fn comme_au_clavier_refuse_l_etat_3_et_les_bits_64_128() {
        for c in ['±', 'µ', '·'] {
            assert_eq!(
                lot_clavier(c, &mut DispositionEssai::new()),
                Err(Arret::Refus(format!(
                    "caractère absent du clavier : « {c} »"
                ))),
                "{c}"
            );
        }
    }

    // Un caractère hors du plan de base refuse avant que la disposition ne soit lue : il n'a pas
    // d'unité UTF-16 seule à donner à `scan`. Les 16 bits bas de U+10040 valent 0x0040, « @ »,
    // que l'AZERTY frappe par AltGr : les tronquer ferait chercher « @ » à sa place.
    #[test]
    fn hors_du_plan_de_base_refuse_sans_lire_la_disposition() {
        let mut d = Notee::new();
        assert_eq!(
            lot_clavier('\u{10040}', &mut d),
            Err(Arret::Refus(
                "caractère absent du clavier : « \u{10040} »".into()
            ))
        );
        assert!(!d.consultee, "la disposition a été lue");
        assert!(d.balayages.is_empty(), "{:?}", d.balayages);
    }
}
