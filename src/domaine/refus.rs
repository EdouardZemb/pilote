//! La liste de refus (ADR-0005, « La liste de refus ») : les fenêtres où `pilote` ne fait rien. Une
//! fenêtre est refusée quand son processus, ou le processus d'une de ses fenêtres enfants qu'un
//! autre processus porte, est l'un de `REFUSES` ; et le cadre d'une application du Store
//! (`CLASSE_CADRE`) l'est quand son AUMID est celui des Paramètres, ou quand il n'en a pas et
//! qu'aucun enfant d'un autre processus ne se lit : réduit, le cadre des Paramètres ne garde que ses
//! propres filles. Rien ne configure cette liste. KeePassXC a sa propre règle, que `gestes` juge
//! pour `touche` et `taper` (décision 3).

use crate::domaine::fenetre::FenetreLue;
use crate::domaine::processus::Processus;
use crate::domaine::processus::meme_nom;
use crate::domaine::processus::processus_de;
use crate::domaine::processus::sans_exe;

/// Les processus refusés, nommés sans `.exe` : le consentement de l'UAC, l'écran de connexion, la
/// demande d'identifiants, les Paramètres.
pub const REFUSES: [&str; 4] = ["consent", "LogonUI", "CredentialUIBroker", "SystemSettings"];

/// L'AUMID des Paramètres, que porte leur cadre.
pub const AUMID_PARAMETRES: &str =
    "windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel";

/// La classe du cadre qui héberge une application du Store.
pub const CLASSE_CADRE: &str = "ApplicationFrameWindow";

/// Vrai si `nom_exe` est l'un de `REFUSES`, sans égard à la casse ni au suffixe `.exe`.
pub fn processus_refuse(nom_exe: &str) -> bool {
    REFUSES.iter().any(|refuse| meme_nom(nom_exe, refuse))
}

/// Vrai si `nom_exe` est KeePassXC, sans égard à la casse ni au suffixe `.exe`.
pub fn est_keepassxc(nom_exe: &str) -> bool {
    meme_nom(nom_exe, "KeePassXC")
}

/// Le nom, sans `.exe`, qui refuse `f` : celui de son processus s'il est refusé ; sinon celui du
/// processus refusé d'un de ses enfants qu'un autre processus porte ; sinon, pour un cadre de classe
/// `CLASSE_CADRE`, `SystemSettings` si son AUMID est celui des Paramètres, ou s'il n'a pas d'AUMID
/// (absent ou vide) et qu'aucun enfant d'un autre processus ne se lit. `None` si rien ne la refuse.
/// La classe et l'AUMID se comparent sans égard à la casse : en cas de doute, le cadre est refusé.
pub fn fenetre_refusee(f: &FenetreLue, tous: &[Processus]) -> Option<String> {
    let refuse = |p: &Processus| processus_refuse(&p.nom).then(|| sans_exe(&p.nom).to_string());
    if let Some(nom) = processus_de(f.pid, tous).and_then(refuse) {
        return Some(nom);
    }
    if let Some(nom) = enfants_d_autres(f, tous).find_map(refuse) {
        return Some(nom);
    }
    let parametres = f.classe.eq_ignore_ascii_case(CLASSE_CADRE)
        && match f.aumid.as_deref().filter(|aumid| !aumid.is_empty()) {
            Some(aumid) => aumid.eq_ignore_ascii_case(AUMID_PARAMETRES),
            None => enfants_d_autres(f, tous).next().is_none(),
        };
    parametres.then(|| "SystemSettings".to_string())
}

/// Vrai si le processus de `f`, ou celui d'un de ses enfants qu'un autre processus porte, est
/// KeePassXC.
pub fn keepassxc_dans(f: &FenetreLue, tous: &[Processus]) -> bool {
    processus_de(f.pid, tous)
        .into_iter()
        .chain(enfants_d_autres(f, tous))
        .any(|p| est_keepassxc(&p.nom))
}

/// Les processus lus des fenêtres enfants de `f` qu'un autre processus que le sien porte ; un
/// enfant dont le processus ne se lit pas n'y paraît pas.
fn enfants_d_autres<'a>(
    f: &'a FenetreLue,
    tous: &'a [Processus],
) -> impl Iterator<Item = &'a Processus> + 'a {
    f.pids_enfants
        .iter()
        .filter(move |&&pid| pid != f.pid)
        .filter_map(move |&pid| processus_de(pid, tous))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domaine::fenetre::essai::f;
    use crate::domaine::fenetre::essai::p;

    /// L'AUMID de la Calculatrice, une autre application hébergée par un cadre.
    const AUMID_CALCULATRICE: &str = "Microsoft.WindowsCalculator_8wekyb3d8bbwe!App";

    /// Le cadre `id` du processus 5, `ApplicationFrameHost.exe`, de classe `CLASSE_CADRE`.
    fn cadre(id: u64, titre: &str) -> FenetreLue {
        let mut c = f(id, 5, titre);
        c.classe = CLASSE_CADRE.into();
        c
    }

    #[test]
    fn refuse_les_quatre_processus_sans_egard_a_la_casse() {
        for n in [
            "consent.exe",
            "LOGONUI.EXE",
            "CredentialUIBroker.exe",
            "systemsettings.exe",
        ] {
            assert!(processus_refuse(n), "{n}");
        }
        assert!(!processus_refuse("SystemSettingsBroker.exe"));
    }
    #[test]
    fn refuse_une_fenetre_dont_un_enfant_est_refuse() {
        let t = [
            p(10, 0, "appli.exe", Some(1)),
            p(7, 0, "SystemSettings.exe", Some(1)),
        ];
        let mut w = f(1, 10, "A");
        w.pids_enfants = vec![7];
        assert_eq!(fenetre_refusee(&w, &t), Some("SystemSettings".into()));
    }
    #[test]
    fn cadre_des_parametres_refuse_par_son_aumid() {
        let t = [p(5, 0, "ApplicationFrameHost.exe", Some(1))];
        let mut c = f(1, 5, "Paramètres");
        c.classe = CLASSE_CADRE.into();
        c.aumid = Some(AUMID_PARAMETRES.into());
        c.pids_enfants = vec![5];
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
    }
    #[test]
    fn cadre_sans_enfant_ni_aumid_lisible_refuse() {
        let t = [p(5, 0, "ApplicationFrameHost.exe", Some(1))];
        let c = cadre(1, "Paramètres");
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
    }
    #[test]
    fn cadre_avec_ses_seules_filles_sans_aumid_refuse() {
        let t = [p(5, 0, "ApplicationFrameHost.exe", Some(1))];
        let mut c = cadre(1, "Paramètres");
        c.pids_enfants = vec![5];
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
        c.reduite = true;
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
    }
    #[test]
    fn cadre_d_une_autre_application_passe() {
        let t = [
            p(5, 0, "ApplicationFrameHost.exe", Some(1)),
            p(30, 0, "CalculatorApp.exe", Some(2)),
        ];
        let mut c = cadre(1, "Calculatrice");
        c.aumid = Some(AUMID_CALCULATRICE.into());
        c.pids_enfants = vec![5, 30];
        assert_eq!(fenetre_refusee(&c, &t), None);
    }
    #[test]
    fn parametres_reduits_restent_refuses() {
        let t = [p(5, 0, "ApplicationFrameHost.exe", Some(1))];
        let mut c = cadre(1, "Paramètres");
        c.reduite = true;
        c.pids_enfants = vec![5];
        c.aumid = Some(AUMID_PARAMETRES.into());
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
    }
    #[test]
    fn keepassxc_par_p_ou_par_un_enfant() {
        let t = [
            p(10, 0, "KeePassXC.exe", Some(1)),
            p(20, 0, "explorer.exe", Some(1)),
            p(30, 0, "KeePassXC.exe", Some(2)),
        ];
        assert!(keepassxc_dans(&f(1, 10, "Base.kdbx - KeePassXC"), &t));
        let mut w = f(2, 20, "Ouvrir");
        w.pids_enfants = vec![20, 30];
        assert!(keepassxc_dans(&w, &t));
        w.pids_enfants = vec![20];
        assert!(!keepassxc_dans(&w, &t));
    }

    // Le nom rendu est celui de l'exécutable du processus, sans `.exe`, dans sa casse.
    #[test]
    fn le_nom_rendu_est_celui_de_l_executable() {
        let t = [p(7, 0, "logonui.EXE", Some(1))];
        assert_eq!(
            fenetre_refusee(&f(1, 7, "Connexion"), &t),
            Some("logonui".into())
        );
    }
    // Une fenêtre ordinaire, sans AUMID ni enfant, d'un processus permis, passe : la règle du cadre
    // ne vise que la classe `CLASSE_CADRE`.
    #[test]
    fn fenetre_ordinaire_sans_enfant_ni_aumid_passe() {
        let t = [p(10, 0, "appli.exe", Some(1))];
        assert_eq!(fenetre_refusee(&f(1, 10, "A"), &t), None);
    }
    // Les valeurs que fixe le plan, lues par leur texte et non par les constantes : la classe du
    // cadre et l'AUMID des Paramètres refusent, sans égard à la casse, même quand un enfant lisible
    // d'un processus permis s'y trouve.
    #[test]
    fn cadre_et_aumid_des_parametres_par_leur_texte() {
        let t = [
            p(5, 0, "ApplicationFrameHost.exe", Some(1)),
            p(30, 0, "CalculatorApp.exe", Some(2)),
        ];
        for (classe, aumid) in [
            (
                "ApplicationFrameWindow",
                "windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel",
            ),
            (
                "applicationframewindow",
                "Windows.ImmersiveControlPanel_cw5n1h2txyewy!Microsoft.Windows.ImmersiveControlPanel",
            ),
        ] {
            let mut c = f(1, 5, "Paramètres");
            c.classe = classe.into();
            c.aumid = Some(aumid.into());
            c.pids_enfants = vec![5, 30];
            assert_eq!(
                fenetre_refusee(&c, &t),
                Some("SystemSettings".into()),
                "{classe} {aumid}"
            );
        }
    }
    // Un AUMID vide, ce que rend la lecture d'une fenêtre sans identifiant, compte comme absent.
    #[test]
    fn cadre_d_aumid_vide_sans_enfant_lisible_refuse() {
        let t = [p(5, 0, "ApplicationFrameHost.exe", Some(1))];
        let mut c = cadre(1, "Paramètres");
        c.aumid = Some(String::new());
        c.pids_enfants = vec![5];
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
    }
    // Sans AUMID, un enfant d'un autre processus qui se lit et que rien ne refuse laisse passer le
    // cadre ; un enfant dont le processus ne se lit pas ne compte pas.
    #[test]
    fn cadre_sans_aumid_passe_par_un_enfant_lisible_et_permis() {
        let t = [
            p(5, 0, "ApplicationFrameHost.exe", Some(1)),
            p(30, 0, "CalculatorApp.exe", Some(2)),
        ];
        let mut c = cadre(1, "Calculatrice");
        c.pids_enfants = vec![5, 30];
        assert_eq!(fenetre_refusee(&c, &t), None);
        c.pids_enfants = vec![5, 77];
        assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into()));
    }
    // KeePassXC n'est pas dans la liste de refus : `activer` vers lui reste permis, et seuls
    // `touche` et `taper` le refusent, par `keepassxc_dans`.
    #[test]
    fn keepassxc_n_est_pas_dans_la_liste_de_refus() {
        let t = [p(10, 0, "KeePassXC.exe", Some(1))];
        assert!(!processus_refuse("KeePassXC.exe"));
        assert_eq!(fenetre_refusee(&f(1, 10, "Base"), &t), None);
    }
    #[test]
    fn est_keepassxc_sans_egard_a_la_casse_exe_ou_non() {
        for n in ["KeePassXC.exe", "keepassxc", "KEEPASSXC.EXE"] {
            assert!(est_keepassxc(n), "{n}");
        }
        for n in ["KeePass.exe", "KeePassXC-Browser.exe", "keepassxc-cli"] {
            assert!(!est_keepassxc(n), "{n}");
        }
    }
}
