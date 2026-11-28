//! Les gardes que Windows tient : le verrou des gestes (`Local\pilote-gestes`), et les trois faits
//! qui disent si le poste est verrouillé (ADR-0005, « Le poste verrouillé »).

use super::Poignee;
use super::jusqu_au_nul;
use super::taille;
use crate::domaine::bureau::Duree;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::bureau::EtatSession;
use crate::domaine::bureau::EtatVerrou;
use core::ffi::c_void;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::WAIT_ABANDONED;
use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::Foundation::WAIT_TIMEOUT;
use windows::Win32::System::RemoteDesktop::WTS_CURRENT_SERVER_HANDLE;
use windows::Win32::System::RemoteDesktop::WTS_CURRENT_SESSION;
use windows::Win32::System::RemoteDesktop::WTSFreeMemory;
use windows::Win32::System::RemoteDesktop::WTSINFOEXW;
use windows::Win32::System::RemoteDesktop::WTSQuerySessionInformationW;
use windows::Win32::System::RemoteDesktop::WTSSessionInfoEx;
use windows::Win32::System::StationsAndDesktops::CloseDesktop;
use windows::Win32::System::StationsAndDesktops::DESKTOP_CONTROL_FLAGS;
use windows::Win32::System::StationsAndDesktops::DESKTOP_READOBJECTS;
use windows::Win32::System::StationsAndDesktops::GetUserObjectInformationW;
use windows::Win32::System::StationsAndDesktops::OpenInputDesktop;
use windows::Win32::System::StationsAndDesktops::UOI_NAME;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::System::Threading::ReleaseMutex;
use windows::Win32::System::Threading::WaitForSingleObject;
use windows::core::PWSTR;
use windows::core::w;

/// Le seul niveau de `WTSINFOEXW` que Windows définit, celui de `WTSInfoExLevel1`.
const NIVEAU_1: u32 = 1;

/// La plus longue attente que `WaitForSingleObject` admet sans la lire comme infinie.
const ATTENTE_MAX: u32 = u32::MAX - 1;

/// La longueur du tampon du nom du bureau d'entrée, en unités UTF-16.
const NOM_MAX: usize = 256;

/// Le verrou des gestes : son handle, et s'il est à `pilote`. À sa sortie de portée, il est
/// relâché (`ReleaseMutex`) s'il est pris, puis fermé, du fil qui l'a pris.
pub struct Verrou {
    poignee: Poignee,
    pris: bool,
}

impl Drop for Verrou {
    fn drop(&mut self) {
        if self.pris {
            // SAFETY: le mutex est à ce fil, qui l'a pris par `WaitForSingleObject` ; le handle se
            // ferme ensuite avec la `Poignee`.
            let _ = unsafe { ReleaseMutex(self.poignee.0) };
        }
    }
}

/// Prend le verrou des gestes en attendant au plus `delai` : `WAIT_OBJECT_0` le donne pris,
/// `WAIT_ABANDONED` repris d'un détenteur mort, qui en donne aussi la propriété, `WAIT_TIMEOUT`
/// occupé. Le verrou rendu se tient jusqu'à sa sortie de portée.
pub fn prendre_verrou(delai: Duree) -> Result<(EtatVerrou, Verrou), ErreurSysteme> {
    // SAFETY: le nom est une constante terminée par NUL ; le handle appartient à la `Poignee`.
    let poignee = Poignee(
        unsafe { CreateMutexW(None, false, w!("Local\\pilote-gestes")) }
            .map_err(|e| ErreurSysteme(format!("CreateMutexW : {e}")))?,
    );
    let attente = u32::try_from(delai.0).map_or(ATTENTE_MAX, |a| a.min(ATTENTE_MAX));
    // SAFETY: le handle est ouvert, et le reste pendant l'attente.
    let issue = unsafe { WaitForSingleObject(poignee.0, attente) };
    let (etat, pris) = match issue {
        WAIT_OBJECT_0 => (EtatVerrou::Pris, true),
        WAIT_ABANDONED => (EtatVerrou::Repris, true),
        WAIT_TIMEOUT => (EtatVerrou::Occupe, false),
        autre => {
            return Err(ErreurSysteme(format!(
                "WaitForSingleObject : {} ({})",
                autre.0,
                windows::core::Error::from_thread()
            )));
        }
    };
    Ok((etat, Verrou { poignee, pris }))
}

/// Les faits qui disent si le poste est verrouillé : le nom du bureau d'entrée, puis les drapeaux
/// et l'état de la session. Chaque lecture ratée donne `None`.
pub fn etat_session() -> EtatSession {
    let (drapeaux, etat) = drapeaux_et_etat();
    EtatSession {
        bureau_d_entree: bureau_d_entree(),
        drapeaux,
        etat,
    }
}

/// Le nom du bureau qui reçoit les entrées (`OpenInputDesktop`, `UOI_NAME`) ; `None` s'il ne
/// s'ouvre pas, ce qu'il fait quand le poste est verrouillé, ou que son nom ne se lit pas.
fn bureau_d_entree() -> Option<String> {
    // SAFETY: le bureau ouvert se ferme par `CloseDesktop` sur tous les chemins ; `nom` porte sa
    // taille en octets.
    let bureau =
        unsafe { OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS) }.ok()?;
    let mut nom = [0u16; NOM_MAX];
    let lu = unsafe {
        GetUserObjectInformationW(
            HANDLE(bureau.0),
            UOI_NAME,
            Some(nom.as_mut_ptr().cast::<c_void>()),
            taille::<[u16; NOM_MAX]>(),
            None,
        )
    };
    let _ = unsafe { CloseDesktop(bureau) };
    lu.ok().map(|()| jusqu_au_nul(&nom))
}

/// Les drapeaux et l'état de la session courante (`WTSQuerySessionInformationW`,
/// `WTSSessionInfoEx`, `WTSInfoExLevel1`) ; `None` pour chacun s'ils ne se lisent pas. Le tampon
/// que Windows alloue se libère par `WTSFreeMemory`, sur tous les chemins.
fn drapeaux_et_etat() -> (Option<i32>, Option<i32>) {
    let mut tampon = PWSTR::null();
    let mut octets = 0u32;
    // SAFETY: `tampon` et `octets` vivent le temps de l'appel ; la structure se lit sans exiger
    // d'alignement, seulement si le tampon la contient entière, et seulement par la variante que
    // dit son niveau.
    let lu = unsafe {
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            WTS_CURRENT_SESSION,
            WTSSessionInfoEx,
            &raw mut tampon,
            &raw mut octets,
        )
    };
    let entier = usize::try_from(octets).is_ok_and(|n| n >= size_of::<WTSINFOEXW>());
    let faits = if lu.is_ok() && !tampon.is_null() && entier {
        let info = unsafe { std::ptr::read_unaligned(tampon.0.cast::<WTSINFOEXW>()) };
        if info.Level == NIVEAU_1 {
            let niveau = unsafe { info.Data.WTSInfoExLevel1 };
            (Some(niveau.SessionFlags), Some(niveau.SessionState.0))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    if !tampon.is_null() {
        unsafe { WTSFreeMemory(tampon.0.cast::<c_void>()) };
    }
    faits
}
