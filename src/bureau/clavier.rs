//! Le clavier : l'envoi d'un lot (`SendInput`), la disposition du fil focalisé (décision 2), relue
//! à chaque appel, les touches de modification enfoncées et Verr. Maj.

use super::hwnd_de;
use super::id_non_nul;
use crate::domaine::bureau::Evenement;
use crate::domaine::bureau::Produit;
use crate::domaine::fenetre::fil_focalise;
use crate::domaine::touches::vk;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_MOUSE;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_UNICODE;
use windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VK_TO_VSC_EX;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_MOVE;
use windows::Win32::UI::Input::KeyboardAndMouse::MOUSEINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyExW;
use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LCONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LSHIFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RMENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;
use windows::Win32::UI::Input::KeyboardAndMouse::VkKeyScanExW;
use windows::Win32::UI::WindowsAndMessaging::GUITHREADINFO;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetGUIThreadInfo;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

/// `ToUnicodeEx` laisse l'état du clavier intact (Windows 10, version 1607 et après) : une touche
/// morte sondée ne s'accroche pas au caractère que l'utilisateur tapera ensuite.
const ETAT_INTACT: u32 = 4;

/// Une touche enfoncée, dans le tableau d'état que lit `ToUnicodeEx`.
const ENFONCEE: u8 = 0x80;

/// Les touches que chaque bit de l'état de `VkKeyScanExW` enfonce pour `ToUnicodeEx` : Maj (1),
/// Ctrl (2), Alt (4), chacune générique et latérale, comme le lot les frappe.
const MODIFICATEURS_DE_L_ETAT: [(u8, [VIRTUAL_KEY; 2]); 3] = [
    (1, [VK_SHIFT, VK_LSHIFT]),
    (2, [VK_CONTROL, VK_LCONTROL]),
    (4, [VK_MENU, VK_RMENU]),
];

/// La taille du tampon de `ToUnicodeEx`, en unités UTF-16 : une touche en produit au plus
/// quelques-unes, une ligature comprise.
const PRODUIT_MAX: usize = 16;

/// La disposition du clavier du fil focalisé, relue à chaque appel : la fenêtre focalisée, à
/// défaut l'active (`GetGUIThreadInfo` du fil au premier plan), à défaut celle du premier plan
/// (`fil_focalise`), puis la disposition de son fil (`GetKeyboardLayout`). `None` quand aucune
/// fenêtre ne se lit, quand son fil ne se lit plus, ou que la disposition est nulle : jamais celle
/// de `pilote` à la place.
fn disposition() -> Option<HKL> {
    let mut fil = GUITHREADINFO {
        cbSize: super::taille::<GUITHREADINFO>(),
        ..Default::default()
    };
    // SAFETY: `fil` porte sa taille dans `cbSize` ; en erreur, ses fenêtres restent nulles.
    let lu = unsafe { GetGUIThreadInfo(0, &raw mut fil) }.is_ok();
    let (focus, active) = if lu {
        (id_non_nul(fil.hwndFocus), id_non_nul(fil.hwndActive))
    } else {
        (None, None)
    };
    let premier_plan = id_non_nul(unsafe { GetForegroundWindow() });
    let fenetre = fil_focalise(focus, active, premier_plan)?;
    // SAFETY: lectures d'une fenêtre et d'un fil ; un fil nul désignerait celui de `pilote`, et
    // s'écarte avant `GetKeyboardLayout`.
    let tid = unsafe { GetWindowThreadProcessId(hwnd_de(fenetre), None) };
    if tid == 0 {
        return None;
    }
    let hkl = unsafe { GetKeyboardLayout(tid) };
    (!hkl.is_invalid()).then_some(hkl)
}

/// `VkKeyScanExW` de l'unité `unite` dans la disposition du fil focalisé ; -1 si la disposition ne
/// se lit pas, ce qui refuse le caractère.
pub fn scan(unite: u16) -> i16 {
    match disposition() {
        // SAFETY: lecture de la disposition `hkl`.
        Some(hkl) => unsafe { VkKeyScanExW(unite, hkl) },
        None => -1,
    }
}

/// Ce que la touche virtuelle `vk` produit sous l'état `etat` (`ToUnicodeEx`, l'état du clavier
/// intact), dans la disposition du fil focalisé : l'état porte 0x80 sur Maj et Maj gauche pour le
/// bit 1, sur Ctrl et Ctrl gauche pour le bit 2, sur Alt et Alt droite pour le bit 4, et rien
/// d'autre, Verr. Maj éteinte. Un résultat négatif est une touche morte. Une disposition illisible
/// ne produit rien, ce qui refuse le caractère.
pub fn produit(vk: u8, etat: u8) -> Produit {
    let Some(hkl) = disposition() else {
        return Produit::Texte(String::new());
    };
    let mut clavier = [0u8; 256];
    for (bit, touches) in MODIFICATEURS_DE_L_ETAT {
        if etat & bit != 0 {
            for touche in touches {
                if let Some(case) = clavier.get_mut(usize::from(touche.0)) {
                    *case = ENFONCEE;
                }
            }
        }
    }
    let mut tampon = [0u16; PRODUIT_MAX];
    // SAFETY: le tableau d'état porte 256 octets ; `tampon` porte sa longueur, que `ToUnicodeEx`
    // respecte.
    let ecrits = unsafe {
        ToUnicodeEx(
            u32::from(vk),
            balayage_dans(vk, hkl),
            &clavier,
            &mut tampon,
            ETAT_INTACT,
            Some(hkl),
        )
    };
    match usize::try_from(ecrits) {
        Err(_) => Produit::Morte,
        Ok(n) => Produit::Texte(String::from_utf16_lossy(&tampon[..n.min(tampon.len())])),
    }
}

/// `MapVirtualKeyExW` (`MAPVK_VK_TO_VSC_EX`) de la touche virtuelle `vk` dans la disposition du fil
/// focalisé ; 0 si la disposition ne se lit pas.
pub fn balayage(vk: u8) -> u16 {
    disposition().map_or(0, |hkl| u16::try_from(balayage_dans(vk, hkl)).unwrap_or(0))
}

/// `MapVirtualKeyExW` (`MAPVK_VK_TO_VSC_EX`) de la touche virtuelle `vk` dans la disposition `hkl`.
fn balayage_dans(vk: u8, hkl: HKL) -> u32 {
    // SAFETY: lecture de la disposition `hkl`.
    unsafe { MapVirtualKeyExW(u32::from(vk), MAPVK_VK_TO_VSC_EX, Some(hkl)) }
}

/// Envoie `lot` d'un seul `SendInput`, et rend le nombre d'événements que Windows a insérés, 0
/// compris : UIPI bloque sans erreur.
pub fn envoyer(lot: &[Evenement]) -> u32 {
    let entrees: Vec<INPUT> = lot.iter().map(entree).collect();
    let taille = i32::try_from(size_of::<INPUT>()).unwrap_or(0);
    // SAFETY: `entrees` est un tableau d'`INPUT` dont chaque union porte la variante que dit son
    // type ; la taille passée est celle d'un `INPUT`.
    unsafe { SendInput(&entrees, taille) }
}

/// L'`INPUT` d'un événement : une touche virtuelle, son balayage et ses drapeaux ; une unité
/// UTF-16 (`KEYEVENTF_UNICODE`, sans touche virtuelle) ; ou un mouvement de souris nul.
fn entree(e: &Evenement) -> INPUT {
    match *e {
        Evenement::Touche {
            vk,
            scan,
            etendue,
            relachee,
        } => clavier(
            VIRTUAL_KEY(u16::from(vk)),
            u16::from(scan),
            drapeau(etendue, KEYEVENTF_EXTENDEDKEY) | drapeau(relachee, KEYEVENTF_KEYUP),
        ),
        Evenement::Unicode { unite, relachee } => clavier(
            VIRTUAL_KEY(0),
            unite,
            KEYEVENTF_UNICODE | drapeau(relachee, KEYEVENTF_KEYUP),
        ),
        Evenement::SourisNulle => INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: MOUSEEVENTF_MOVE,
                    ..Default::default()
                },
            },
        },
    }
}

/// Un `INPUT` du clavier.
fn clavier(touche: VIRTUAL_KEY, balayage: u16, drapeaux: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: touche,
                wScan: balayage,
                dwFlags: drapeaux,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// `valeur` si `pose`, sinon aucun drapeau.
fn drapeau(pose: bool, valeur: KEYBD_EVENT_FLAGS) -> KEYBD_EVENT_FLAGS {
    if pose { valeur } else { KEYBD_EVENT_FLAGS(0) }
}

/// Les touches de modification enfoncées, parmi `vk::MODIFICATEURS` (`GetAsyncKeyState`, le bit
/// haut).
pub fn touches_enfoncees() -> Vec<u8> {
    vk::MODIFICATEURS
        .into_iter()
        // SAFETY: lecture de l'état d'une touche.
        .filter(|&v| unsafe { GetAsyncKeyState(i32::from(v)) } < 0)
        .collect()
}

/// Vrai si Verr. Maj est allumée (`GetKeyState`, le bit bas).
pub fn verr_maj() -> bool {
    // SAFETY: lecture de l'état d'une touche.
    let etat = unsafe { GetKeyState(i32::from(VK_CAPITAL.0)) };
    etat & 1 != 0
}
