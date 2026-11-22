//! Les processus : l'instantané (`CreateToolhelp32Snapshot`), l'heure de création de chacun
//! (`GetProcessTimes`), et le niveau d'intégrité d'un processus, lu sur son jeton.

use super::Poignee;
use super::jusqu_au_nul;
use super::taille;
use crate::domaine::bureau::ErreurSysteme;
use crate::domaine::processus::Processus;
use windows::Win32::Foundation::FILETIME;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::GetSidSubAuthority;
use windows::Win32::Security::GetSidSubAuthorityCount;
use windows::Win32::Security::GetTokenInformation;
use windows::Win32::Security::IsValidSid;
use windows::Win32::Security::TOKEN_MANDATORY_LABEL;
use windows::Win32::Security::TOKEN_QUERY;
use windows::Win32::Security::TokenIntegrityLevel;
use windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
use windows::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W;
use windows::Win32::System::Diagnostics::ToolHelp::Process32FirstW;
use windows::Win32::System::Diagnostics::ToolHelp::Process32NextW;
use windows::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::System::Threading::GetProcessTimes;
use windows::Win32::System::Threading::OpenProcess;
use windows::Win32::System::Threading::OpenProcessToken;
use windows::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

/// L'instantané des processus : chacun avec son parent, le nom de son exécutable et son heure de
/// création. La fin du parcours arrive en erreur de `Process32NextW` ; un premier processus
/// illisible est une erreur, puisque `pilote` au moins tourne.
pub fn processus() -> Result<Vec<Processus>, ErreurSysteme> {
    // SAFETY: l'instantané se ferme à la sortie de portée de sa `Poignee` ; `entree` porte sa
    // taille dans `dwSize`, comme `Process32FirstW` l'exige.
    let instantane = Poignee(
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .map_err(|e| ErreurSysteme(format!("CreateToolhelp32Snapshot : {e}")))?,
    );
    let mut entree = PROCESSENTRY32W {
        dwSize: taille::<PROCESSENTRY32W>(),
        ..Default::default()
    };
    unsafe { Process32FirstW(instantane.0, &raw mut entree) }
        .map_err(|e| ErreurSysteme(format!("Process32FirstW : {e}")))?;
    let mut lus = Vec::new();
    loop {
        lus.push(Processus {
            pid: entree.th32ProcessID,
            parent: entree.th32ParentProcessID,
            nom: jusqu_au_nul(&entree.szExeFile),
            creation: creation(entree.th32ProcessID),
        });
        if unsafe { Process32NextW(instantane.0, &raw mut entree) }.is_err() {
            return Ok(lus);
        }
    }
}

/// L'heure de création du processus `pid`, en intervalles de 100 ns depuis 1601 ; `None` si le
/// processus ne s'ouvre pas ou que l'heure ne se lit pas.
fn creation(pid: u32) -> Option<u64> {
    let processus = ouvrir(pid)?;
    let mut temps = [FILETIME::default(); 4];
    let [naissance, sortie, noyau, utilisateur] = &mut temps;
    // SAFETY: les quatre `FILETIME` vivent le temps de l'appel ; le handle se ferme à la sortie de
    // portée de sa `Poignee`.
    unsafe { GetProcessTimes(processus.0, naissance, sortie, noyau, utilisateur) }.ok()?;
    Some((u64::from(naissance.dwHighDateTime) << 32) | u64::from(naissance.dwLowDateTime))
}

/// Le processus `pid`, ouvert pour les seules lectures limitées ; `None` s'il ne s'ouvre pas.
fn ouvrir(pid: u32) -> Option<Poignee> {
    // SAFETY: le handle rendu appartient à la `Poignee`, qui le ferme.
    unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
        .ok()
        .map(Poignee)
}

/// Le niveau d'intégrité du processus `pid`, ou de `pilote` lui-même pour `None` : le dernier
/// sous-identifiant du SID que porte son jeton (`TokenIntegrityLevel`). `None` en toute erreur.
pub fn niveau_integrite(pid: Option<u32>) -> Option<u32> {
    let ouvert = match pid {
        Some(pid) => Some(ouvrir(pid)?),
        None => None,
    };
    // SAFETY: le pseudo-handle de `GetCurrentProcess` ne se ferme pas ; un processus ouvert se
    // ferme à la sortie de portée de sa `Poignee`, le jeton de même.
    let processus = ouvert
        .as_ref()
        .map_or_else(|| unsafe { GetCurrentProcess() }, |p| p.0);
    let mut jeton = HANDLE::default();
    unsafe { OpenProcessToken(processus, TOKEN_QUERY, &raw mut jeton) }.ok()?;
    let jeton = Poignee(jeton);
    niveau_du_jeton(&jeton)
}

/// Le niveau d'intégrité que porte `jeton`, lu en deux appels : la taille, puis l'étiquette.
fn niveau_du_jeton(jeton: &Poignee) -> Option<u32> {
    let mut taille_lue = 0u32;
    // SAFETY: le premier appel ne demande que la taille et échoue par construction ; le second
    // reçoit un tampon de cette taille, qu'il porte.
    let _ =
        unsafe { GetTokenInformation(jeton.0, TokenIntegrityLevel, None, 0, &raw mut taille_lue) };
    let longueur = usize::try_from(taille_lue).ok()?;
    if longueur < size_of::<TOKEN_MANDATORY_LABEL>() {
        return None;
    }
    let mut tampon = vec![0u8; longueur];
    unsafe {
        GetTokenInformation(
            jeton.0,
            TokenIntegrityLevel,
            Some(tampon.as_mut_ptr().cast()),
            taille_lue,
            &raw mut taille_lue,
        )
    }
    .ok()?;
    // SAFETY: le tampon porte au moins une `TOKEN_MANDATORY_LABEL`, lue sans exiger
    // d'alignement ; le SID qu'elle désigne est dans le tampon, vivant jusqu'au retour, et se
    // vérifie avant toute lecture de ses sous-identifiants.
    let etiquette =
        unsafe { std::ptr::read_unaligned(tampon.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()) };
    let sid = etiquette.Label.Sid;
    if sid.is_invalid() || !unsafe { IsValidSid(sid) }.as_bool() {
        return None;
    }
    let compte = unsafe { *GetSidSubAuthorityCount(sid) };
    let dernier = compte.checked_sub(1)?;
    Some(unsafe { *GetSidSubAuthority(sid, u32::from(dernier)) })
}
