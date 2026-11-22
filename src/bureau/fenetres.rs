//! Les fenêtres : celles de premier niveau (`EnumWindows`), et pour chacune ce que le domaine juge
//! (visibilité, masquage de DWM, style d'outil, titre, classe, propriétaire, réduction, processus,
//! AUMID, processus des fenêtres enfants) ; la fenêtre au premier plan et la chaîne de ses
//! propriétaires ; la restauration et la demande du premier plan.

use super::hwnd_de;
use super::id_de;
use super::id_non_nul;
use super::taille;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::fenetre::FenetreLue;
use crate::domaine::fenetre::IdFenetre;
use crate::domaine::fenetre::PremierPlan;
use core::ffi::c_void;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Graphics::Dwm::DWMWA_CLOAKED;
use windows::Win32::Graphics::Dwm::DwmGetWindowAttribute;
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::StructuredStorage::PropVariantClear;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::Shell::PropertiesSystem::SHGetPropertyStoreForWindow;
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GW_OWNER;
use windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE;
use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindow;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::IsIconic;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::Win32::UI::WindowsAndMessaging::SW_RESTORE;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::ShowWindowAsync;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;
use windows::core::BOOL;

/// La plus longue chaîne de propriétaires lue au premier plan.
const PROPRIETAIRES_MAX: usize = 16;

/// La plus longue classe de fenêtre que Windows admet, NUL compris.
const CLASSE_MAX: usize = 257;

/// Les fenêtres de premier niveau, dans l'ordre d'`EnumWindows`, chacune lue.
pub fn fenetres() -> Result<Vec<FenetreLue>, ErreurSysteme> {
    let mut hwnds: Vec<HWND> = Vec::new();
    // SAFETY: `collecter` ne reçoit que le pointeur vers `hwnds`, qui vit jusqu'au retour
    // d'`EnumWindows`, lequel n'appelle le rappel que pendant l'appel.
    unsafe { EnumWindows(Some(collecter), LPARAM(&raw mut hwnds as isize)) }
        .map_err(|e| ErreurSysteme(format!("EnumWindows : {e}")))?;
    Ok(hwnds.into_iter().map(lire).collect())
}

/// Le rappel d'`EnumWindows` et d'`EnumChildWindows` : ajoute la fenêtre à la liste que porte
/// `liste`, et rend toujours vrai, pour que l'énumération aille à son terme.
unsafe extern "system" fn collecter(h: HWND, liste: LPARAM) -> BOOL {
    // SAFETY: `liste` porte le pointeur vers un `Vec<HWND>` que l'appelant tient en vie et
    // n'emprunte pas pendant l'énumération.
    let liste = unsafe { &mut *(liste.0 as *mut Vec<HWND>) };
    liste.push(h);
    BOOL::from(true)
}

/// La fenêtre `h`, lue.
fn lire(h: HWND) -> FenetreLue {
    FenetreLue {
        id: id_de(h),
        pid: pid_de(h),
        titre: titre(h),
        classe: classe(h),
        // SAFETY: `h` vient d'`EnumWindows` ; une fenêtre détruite depuis rend faux.
        visible: unsafe { IsWindowVisible(h) }.as_bool(),
        masquee: masquee(h),
        outil: outil(h),
        proprietaire: proprietaire(h),
        reduite: reduite(h),
        aumid: aumid(h),
        pids_enfants: pids_enfants(h),
    }
}

/// Le processus de la fenêtre `h` ; 0 si elle n'existe plus.
fn pid_de(h: HWND) -> u32 {
    let mut pid = 0u32;
    // SAFETY: `pid` vit le temps de l'appel.
    unsafe { GetWindowThreadProcessId(h, Some(&raw mut pid)) };
    pid
}

/// Le titre de la fenêtre `h`, vide si elle n'en a pas.
fn titre(h: HWND) -> String {
    // SAFETY: lectures d'une fenêtre ; `tampon` porte sa longueur, que `GetWindowTextW` respecte.
    let longueur = unsafe { GetWindowTextLengthW(h) };
    let mut tampon = vec![0u16; usize::try_from(longueur).unwrap_or(0) + 1];
    let lus = unsafe { GetWindowTextW(h, &mut tampon) };
    let lus = usize::try_from(lus).unwrap_or(0).min(tampon.len());
    String::from_utf16_lossy(&tampon[..lus])
}

/// La classe de la fenêtre `h`, vide si elle ne se lit pas.
fn classe(h: HWND) -> String {
    let mut tampon = [0u16; CLASSE_MAX];
    // SAFETY: `tampon` porte sa longueur, que `GetClassNameW` respecte.
    let lus = unsafe { GetClassNameW(h, &mut tampon) };
    let lus = usize::try_from(lus).unwrap_or(0).min(tampon.len());
    String::from_utf16_lossy(&tampon[..lus])
}

/// Vrai si DWM masque la fenêtre `h` (`DWMWA_CLOAKED` non nul), ou si l'attribut ne se lit pas :
/// une lecture ratée ne rend jamais une fenêtre candidate.
fn masquee(h: HWND) -> bool {
    let mut masquage = 0u32;
    // SAFETY: `masquage` est un `u32`, dont la taille est passée.
    let lu = unsafe {
        DwmGetWindowAttribute(
            h,
            DWMWA_CLOAKED,
            (&raw mut masquage).cast::<c_void>(),
            taille::<u32>(),
        )
    };
    lu.is_err() || masquage != 0
}

/// Vrai si la fenêtre `h` porte le style étendu de fenêtre d'outil.
fn outil(h: HWND) -> bool {
    // SAFETY: lecture d'un style de fenêtre ; une fenêtre détruite rend 0.
    let styles = unsafe { GetWindowLongPtrW(h, GWL_EXSTYLE) };
    styles as u32 & WS_EX_TOOLWINDOW.0 != 0
}

/// Le propriétaire de la fenêtre `h` (`GetWindow`, `GW_OWNER`) ; une erreur ou un nul, l'absence
/// comprise, vaut « sans propriétaire ». C'est la seule lecture de la propriété : la fenêtre lue et
/// la chaîne du premier plan, que le domaine compare, la lisent ainsi de la même façon.
fn proprietaire_de(h: HWND) -> Option<HWND> {
    // SAFETY: lecture d'une fenêtre.
    unsafe { GetWindow(h, GW_OWNER) }
        .ok()
        .filter(|p| !p.is_invalid())
}

/// Le propriétaire de la fenêtre `h`, s'il en a un.
fn proprietaire(h: HWND) -> Option<IdFenetre> {
    proprietaire_de(h).map(id_de)
}

/// Vrai si la fenêtre `h` est réduite.
fn reduite(h: HWND) -> bool {
    // SAFETY: lecture d'une fenêtre.
    unsafe { IsIconic(h) }.as_bool()
}

/// L'AUMID de la fenêtre `h` (`PKEY_AppUserModel_ID`) ; `None` s'il est vide ou ne se lit pas.
fn aumid(h: HWND) -> Option<String> {
    // SAFETY: le magasin se lit puis se relâche à sa sortie de portée ; la valeur lue est vidée
    // par `PropVariantClear`, et le texte alloué libéré par `CoTaskMemFree`, sur tous les chemins.
    let magasin: IPropertyStore = unsafe { SHGetPropertyStoreForWindow(h) }.ok()?;
    let mut valeur = unsafe { magasin.GetValue(&PKEY_AppUserModel_ID) }.ok()?;
    let texte = unsafe { PropVariantToStringAlloc(&raw const valeur) };
    let _ = unsafe { PropVariantClear(&raw mut valeur) };
    let texte = texte.ok()?;
    let lu = (!texte.is_null()).then(|| String::from_utf16_lossy(unsafe { texte.as_wide() }));
    unsafe { CoTaskMemFree(Some(texte.0.cast_const().cast())) };
    lu.filter(|aumid| !aumid.is_empty())
}

/// Les processus des fenêtres enfants de `h`, à toute profondeur, chacun une fois, le sien compris
/// s'il en porte. L'énumération part toujours de `h` : sans parent, elle lirait tout le bureau.
/// Un PID nul est une lecture ratée, celle d'une fille détruite entre l'énumération et la lecture
/// de son processus : il n'y paraît pas. L'instantané des processus porte en effet une entrée de
/// PID 0, le processus inactif du système, que le domaine prendrait pour le processus lisible d'un
/// enfant d'un autre processus, et le cadre d'une application du Store sans AUMID passerait alors.
fn pids_enfants(h: HWND) -> Vec<u32> {
    let mut enfants: Vec<HWND> = Vec::new();
    // SAFETY: comme pour `fenetres` ; le retour d'`EnumChildWindows` ne porte rien.
    let _ =
        unsafe { EnumChildWindows(Some(h), Some(collecter), LPARAM(&raw mut enfants as isize)) };
    let mut pids: Vec<u32> = enfants
        .into_iter()
        .map(pid_de)
        .filter(|&pid| pid != 0)
        .collect();
    pids.sort_unstable();
    pids.dedup();
    pids
}

/// La fenêtre au premier plan, son processus et la chaîne de ses propriétaires ; `None` si elle
/// est nulle.
pub fn premier_plan() -> Option<PremierPlan> {
    // SAFETY: `GetForegroundWindow` ne prend rien.
    let h = unsafe { GetForegroundWindow() };
    let id = id_non_nul(h)?;
    Some(PremierPlan {
        id,
        pid: pid_de(h),
        proprietaires: proprietaires(h),
    })
}

/// La chaîne des propriétaires de `h`, du plus proche au plus lointain, `PROPRIETAIRES_MAX` au
/// plus ; le premier « sans propriétaire » de `proprietaire_de` l'arrête, et aucun n'y est nul.
fn proprietaires(h: HWND) -> Vec<IdFenetre> {
    std::iter::successors(proprietaire_de(h), |&courante| proprietaire_de(courante))
        .take(PROPRIETAIRES_MAX)
        .map(id_de)
        .collect()
}

/// Vrai si la fenêtre `f` est réduite.
pub fn est_reduite(f: IdFenetre) -> bool {
    reduite(hwnd_de(f))
}

/// Restaure la fenêtre `f` (`ShowWindowAsync`, `SW_RESTORE`), sans attendre son fil :
/// `ShowWindow` attendrait que le fil d'une autre application traite les messages qu'il lui
/// envoie, et une application qui ne répond plus bloquerait `pilote`, le verrou des gestes tenu,
/// avant la sonde de la restauration. Le retour dit si la demande est partie, rien de la
/// restauration : une fenêtre figée reste réduite, et la sonde du domaine refuse.
pub fn restaurer(f: IdFenetre) {
    // SAFETY: une fenêtre qui n'existe plus rend faux, sans suite.
    let _ = unsafe { ShowWindowAsync(hwnd_de(f), SW_RESTORE) };
}

/// Demande le premier plan pour la fenêtre `f` (`SetForegroundWindow`) ; Windows peut refuser sans
/// le dire, et la sonde du domaine juge.
pub fn demander_premier_plan(f: IdFenetre) {
    // SAFETY: une fenêtre qui n'existe plus rend faux, sans suite.
    let _ = unsafe { SetForegroundWindow(hwnd_de(f)) };
}
