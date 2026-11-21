//! Les processus du poste, tels que l'instantané les lit : leur nom, leur parent et leur heure de
//! création. L'arbre d'un processus suit ses descendants, mais un PID se réutilise après la mort de
//! son processus : un fils ne compte que si les deux heures de création se lisent, la sienne n'étant
//! pas antérieure à celle de son père. Un nom se compare sans égard à la casse ni au suffixe `.exe`.

use std::collections::BTreeSet;

/// Un processus lu : son PID, celui de son parent, le nom de son exécutable (`.exe` compris, tel que
/// l'instantané le rend) et son heure de création, `None` si elle ne se lit pas.
#[derive(Clone, Debug, PartialEq)]
pub struct Processus {
    pub pid: u32,
    pub parent: u32,
    pub nom: String,
    pub creation: Option<u64>,
}

/// Vrai si `nom_exe` et `demande` nomment le même exécutable : sans égard à la casse, lettres
/// accentuées comprises, et sans égard au suffixe `.exe` de l'un ou de l'autre.
pub fn meme_nom(nom_exe: &str, demande: &str) -> bool {
    sans_exe(nom_exe).to_lowercase() == sans_exe(demande).to_lowercase()
}

/// `nom_exe` sans son suffixe `.exe`, sans égard à la casse ; tel quel s'il ne le porte pas.
pub fn sans_exe(nom_exe: &str) -> &str {
    match nom_exe.len().checked_sub(4) {
        Some(debut)
            if nom_exe
                .get(debut..)
                .is_some_and(|fin| fin.eq_ignore_ascii_case(".exe")) =>
        {
            &nom_exe[..debut]
        }
        _ => nom_exe,
    }
}

/// Les racines et leurs descendants : un processus de `tous` compte si son parent est dans l'arbre
/// et que les deux heures de création se lisent, celle du fils n'étant pas antérieure à celle du
/// père. Les racines y sont toujours. Un PID ne se suit qu'à sa première entrée dans l'arbre, et
/// cette seule garde termine la descente : un parent de soi-même ne fait pas boucler.
pub fn arbre(racines: &[u32], tous: &[Processus]) -> BTreeSet<u32> {
    let mut arbre: BTreeSet<u32> = racines.iter().copied().collect();
    let mut a_suivre = racines.to_vec();
    while let Some(pere) = a_suivre.pop() {
        let Some(naissance) = processus_de(pere, tous).and_then(|p| p.creation) else {
            continue;
        };
        let fils = tous
            .iter()
            .filter(|p| p.parent == pere && p.creation.is_some_and(|c| c >= naissance));
        for p in fils {
            if arbre.insert(p.pid) {
                a_suivre.push(p.pid);
            }
        }
    }
    arbre
}

/// Le processus de PID `pid` dans `tous`, s'il s'y lit.
pub(crate) fn processus_de(pid: u32, tous: &[Processus]) -> Option<&Processus> {
    tous.iter().find(|p| p.pid == pid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::fenetre::essai::p;

    #[test]
    fn meme_nom_sans_egard_a_la_casse_ni_a_exe() {
        assert!(meme_nom("Etabli.EXE", "etabli"));
        assert!(meme_nom("etabli.exe", "etabli.exe"));
        assert!(!meme_nom("etablis.exe", "etabli"));
    }
    #[test]
    fn arbre_suit_les_descendants() {
        let t = [
            p(1, 0, "a.exe", Some(10)),
            p(2, 1, "b.exe", Some(11)),
            p(3, 2, "c.exe", Some(12)),
        ];
        assert_eq!(arbre(&[1], &t), [1, 2, 3].into());
    }
    #[test]
    fn un_pid_repris_n_est_pas_un_descendant() {
        let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", Some(5))];
        assert_eq!(arbre(&[1], &t), [1].into());
    }
    #[test]
    fn creation_illisible_n_est_pas_un_descendant() {
        let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", None)];
        assert_eq!(arbre(&[1], &t), [1].into());
    }

    // `sans_exe` n'ôte que le suffixe `.exe`, en majuscules comme en minuscules ; un nom sans lui,
    // un autre suffixe, un nom trop court, ou un nom dont les quatre derniers octets coupent un
    // caractère, se rendent tels quels, sans panique.
    #[test]
    fn sans_exe_n_ote_que_le_suffixe_exe() {
        assert_eq!(sans_exe("Etabli.EXE"), "Etabli");
        assert_eq!(sans_exe("a.exe"), "a");
        assert_eq!(sans_exe("etabli"), "etabli");
        assert_eq!(sans_exe("etabli.exe.txt"), "etabli.exe.txt");
        assert_eq!(sans_exe("exe"), "exe");
        assert_eq!(sans_exe("éabc"), "éabc");
    }
    // La casse se compare au-delà de l'ASCII : une lettre accentuée en majuscule vaut sa minuscule.
    #[test]
    fn meme_nom_replie_la_casse_des_lettres_accentuees() {
        assert!(meme_nom("Éditeur.exe", "éditeur"));
        assert!(!meme_nom("editeur.exe", "éditeur"));
    }
    // Un fils né au même instant que son père compte : seule une heure antérieure trahit un PID
    // repris.
    #[test]
    fn fils_ne_au_meme_instant_est_un_descendant() {
        let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", Some(10))];
        assert_eq!(arbre(&[1], &t), [1, 2].into());
    }
    // Un père dont l'heure de création ne se lit pas n'a aucun descendant.
    #[test]
    fn creation_du_pere_illisible_n_a_aucun_descendant() {
        let t = [p(1, 0, "a.exe", None), p(2, 1, "b.exe", Some(11))];
        assert_eq!(arbre(&[1], &t), [1].into());
    }
    // Chaque racine apporte ses descendants, et rien d'autre ; un processus qui se donne pour son
    // propre parent, ou deux processus parents l'un de l'autre, ne font pas boucler.
    #[test]
    fn plusieurs_racines_sans_boucle() {
        let t = [
            p(1, 0, "a.exe", Some(10)),
            p(2, 1, "b.exe", Some(11)),
            p(5, 5, "c.exe", Some(3)),
            p(6, 5, "d.exe", Some(4)),
            p(7, 8, "e.exe", Some(1)),
            p(8, 7, "f.exe", Some(1)),
            p(9, 0, "g.exe", Some(1)),
        ];
        assert_eq!(arbre(&[1, 5, 7], &t), [1, 2, 5, 6, 7, 8].into());
    }
    #[test]
    fn processus_de_trouve_par_le_pid() {
        let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", Some(11))];
        assert_eq!(processus_de(2, &t), Some(&t[1]));
        assert_eq!(processus_de(3, &t), None);
    }
}
