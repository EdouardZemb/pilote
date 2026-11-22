//! Le module commun des essais de bout en bout : leurs dossiers et leurs profils jetables,
//! l'application d'essai et ses copies, le teneur du verrou des gestes, `pilote` et les deux
//! shells, et les lectures du bureau dont les essais jugent. Ces essais pilotent le bureau, et seule
//! la CI les lance (décision 13 du plan maître) : chaque fonction publique, sauf `exiger_la_ci`
//! elle-même, et chaque constructeur commencent par `exiger_la_ci`. Chaque processus lancé reçoit
//! un profil jetable à lui. C'est le seul fichier d'essai qui lance un processus, ce que
//! `tests/statique.rs` garde.

#![cfg(windows)]
#![allow(
    dead_code,
    reason = "chaque fichier d'essai n'emploie qu'une partie du module commun"
)]

use pilote::domaine::refus::AUMID_PARAMETRES;
use pilote::domaine::refus::CLASSE_CADRE;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::Peekable;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::str::Chars;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
use windows::Win32::System::Com::CoInitializeEx;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::StructuredStorage::PropVariantClear;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::System::Threading::OpenMutexW;
use windows::Win32::System::Threading::OpenProcess;
use windows::Win32::System::Threading::PROCESS_NAME_WIN32;
use windows::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
use windows::Win32::System::Threading::QueryFullProcessImageNameW;
use windows::Win32::System::Threading::SYNCHRONIZATION_SYNCHRONIZE;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP;
use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL;
use windows::Win32::UI::Input::KeyboardAndMouse::VkKeyScanExW;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::Shell::PropertiesSystem::SHGetPropertyStoreForWindow;
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GUITHREADINFO;
use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use windows::Win32::UI::WindowsAndMessaging::GetGUIThreadInfo;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::IsIconic;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;
use windows::Win32::UI::WindowsAndMessaging::ShowWindow;
use windows::core::BOOL;
use windows::core::PWSTR;
use windows::core::w;

/// Le pas des sondages.
const PAS: Duration = Duration::from_millis(50);

/// Le plus long qu'un événement ou un état attendu met à paraître.
pub const DELAI: Duration = Duration::from_secs(5);

/// Le temps laissé aux événements pour arriver, avant de juger qu'aucun ne paraît, ou de lire la
/// dernière valeur reçue.
pub const CALME: Duration = Duration::from_millis(500);

/// Le plus long qu'une application d'essai met à dire `pret`.
const PRET_MAX: Duration = Duration::from_secs(20);

/// Le plus long que le teneur met à dire `verrou pris`.
const VERROU_MAX: Duration = Duration::from_secs(10);

/// Le plus long que la fenêtre focalisée met à passer dans la WebView2, après l'activation.
const FOCUS_MAX: Duration = Duration::from_secs(2);

/// Le plus long que Verr. Maj met à changer d'état.
const VERR_MAJ_MAX: Duration = Duration::from_secs(2);

/// La disposition française (France), et ce qu'y donne `VkKeyScanExW` : `@` sur AltGr et 0
/// (l'état 6, la touche 0x30), `/` sur Maj et `:` (l'état 1, la touche 0xBF).
const HKL_FRANCAIS: usize = 0x040C_040C;
const AROBASE_FRANCAIS: i16 = 0x0630;
const BARRE_FRANCAISE: i16 = 0x01BF;

/// Le processus de la WebView2, dont le fil porte la fenêtre focalisée de l'application d'essai.
const WEBVIEW2: &str = "msedgewebview2.exe";

/// L'exécutable des Paramètres, qu'un enfant du cadre porte tant que le cadre n'est pas réduit.
const PARAMETRES: &str = "SystemSettings.exe";

/// Le message qui dit que l'application d'essai n'est pas construite.
const APPLI_MANQUE: &str = "appli-essai.exe manque à côté de pilote : cargo build -p appli-essai";

/// Le rang de chaque dossier jetable, qui rend chacun unique dans les essais d'un même processus.
static RANG: AtomicU32 = AtomicU32::new(0);

/// Panique si `GITHUB_ACTIONS` ne vaut pas `true` : ces essais pilotent le bureau, et seule la CI
/// les lance, jamais le poste.
pub fn exiger_la_ci() {
    let ci = std::env::var("GITHUB_ACTIONS").is_ok_and(|valeur| valeur == "true");
    assert!(
        ci,
        "GITHUB_ACTIONS ne vaut pas « true » : ces essais pilotent le bureau, seule la CI les lance"
    );
}

/// Un dossier neuf, `%RUNNER_TEMP%\pilote-essais\<nom>-<pid>-<n>`, `<pid>` celui des essais et
/// `<n>` le rang de l'appel : vidé s'il existait, puis créé. L'étape de dépôt de la CI emporte
/// `pilote-essais` quand un essai échoue.
pub fn dossier_jetable(nom: &str) -> PathBuf {
    exiger_la_ci();
    let racine = std::env::var_os("RUNNER_TEMP").expect("RUNNER_TEMP manque");
    let rang = RANG.fetch_add(1, Ordering::Relaxed);
    let dossier = PathBuf::from(racine)
        .join("pilote-essais")
        .join(format!("{nom}-{}-{rang}", std::process::id()));
    if dossier.exists() {
        fs::remove_dir_all(&dossier).unwrap_or_else(|e| panic!("{} : {e}", dossier.display()));
    }
    fs::create_dir_all(&dossier).unwrap_or_else(|e| panic!("{} : {e}", dossier.display()));
    dossier
}

/// Un profil jetable : `racine` tient lieu de `USERPROFILE`, avec ses dossiers `AppData\Roaming`,
/// `AppData\Local` et `Temp`.
pub struct Profil {
    pub racine: PathBuf,
}

impl Profil {
    /// Un profil neuf, dans un dossier jetable nommé d'après `nom`.
    pub fn neuf(nom: &str) -> Profil {
        exiger_la_ci();
        let racine = dossier_jetable(nom);
        for dossier in [r"AppData\Roaming", r"AppData\Local", "Temp"] {
            let chemin = racine.join(dossier);
            fs::create_dir_all(&chemin).unwrap_or_else(|e| panic!("{} : {e}", chemin.display()));
        }
        Profil { racine }
    }

    /// Les variables d'un processus lancé dans ce profil : `USERPROFILE`, `APPDATA`,
    /// `LOCALAPPDATA`, `TEMP` et `TMP`, puis `PATH`, qui commence par le dossier de `pilote`, où se
    /// trouve aussi `appli-essai.exe`.
    pub fn variables(&self) -> Vec<(&'static str, OsString)> {
        exiger_la_ci();
        let temp = self.racine.join("Temp").into_os_string();
        let mut chemins = dossier_de_pilote().into_os_string();
        if let Some(ancien) = std::env::var_os("PATH") {
            chemins.push(";");
            chemins.push(ancien);
        }
        vec![
            ("USERPROFILE", self.racine.clone().into_os_string()),
            (
                "APPDATA",
                self.racine.join(r"AppData\Roaming").into_os_string(),
            ),
            (
                "LOCALAPPDATA",
                self.racine.join(r"AppData\Local").into_os_string(),
            ),
            ("TEMP", temp.clone()),
            ("TMP", temp),
            ("PATH", chemins),
        ]
    }
}

/// Le dossier de `pilote`, où cargo construit aussi `appli-essai.exe`.
fn dossier_de_pilote() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_pilote"))
        .parent()
        .expect("le dossier de pilote")
        .to_path_buf()
}

/// `appli-essai.exe`, à côté de `pilote` ; panique s'il manque.
fn appli_essai() -> PathBuf {
    let exe = dossier_de_pilote().join("appli-essai.exe");
    assert!(exe.is_file(), "{APPLI_MANQUE}");
    exe
}

/// La sortie d'erreur d'un processus lancé dans `profil` : le fichier `erreur.txt` de sa racine,
/// que l'étape de dépôt emporte.
fn fichier_d_erreur(profil: &Profil) -> Stdio {
    let chemin = profil.racine.join("erreur.txt");
    File::create(&chemin)
        .map(Stdio::from)
        .unwrap_or_else(|e| panic!("{} : {e}", chemin.display()))
}

/// Vrai si `critere` devient vrai avant `delai`, sondé tout de suite, puis toutes les 50 ms, puis
/// une dernière fois à la borne.
pub fn sonder(delai: Duration, mut critere: impl FnMut() -> bool) -> bool {
    exiger_la_ci();
    let debut = Instant::now();
    loop {
        if critere() {
            return true;
        }
        if debut.elapsed() >= delai {
            return false;
        }
        thread::sleep(PAS);
    }
}

/// Une application d'essai lancée : son nom de processus, sans `.exe`, son PID et son fichier
/// d'événements. Elle tourne dans son propre profil, et `Drop` la tue puis l'attend.
pub struct Appli {
    pub nom: String,
    pub pid: u32,
    pub evenements: PathBuf,
    enfant: Child,
    profil: Profil,
}

impl Appli {
    /// Vrai si une ligne du fichier d'événements contient `motif` avant `delai`, sondé toutes les
    /// 50 ms.
    pub fn attendre(&self, motif: &str, delai: Duration) -> bool {
        exiger_la_ci();
        sonder(delai, || self.lignes().iter().any(|l| l.contains(motif)))
    }

    /// Les lignes du fichier d'événements, telles qu'elles sont maintenant ; aucune s'il n'existe
    /// pas encore.
    pub fn lignes(&self) -> Vec<String> {
        exiger_la_ci();
        fs::read(&self.evenements)
            .map(|octets| {
                String::from_utf8_lossy(&octets)
                    .lines()
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Les événements de la page, dans l'ordre du fichier : chaque ligne `<titre>\t<JSON>`, lue.
    pub fn evenements(&self) -> Vec<Evenement> {
        exiger_la_ci();
        self.lignes()
            .iter()
            .filter_map(|l| Evenement::lire(l))
            .collect()
    }

    /// La valeur de la dernière saisie de l'élément `id`, s'il en a reçu une.
    pub fn derniere_saisie(&self, id: &str) -> Option<String> {
        exiger_la_ci();
        self.evenements()
            .into_iter()
            .rev()
            .find(|e| e.genre == "saisie" && e.id == id)
            .map(|e| e.valeur)
    }
}

impl Drop for Appli {
    fn drop(&mut self) {
        let _ = self.enfant.kill();
        let _ = self.enfant.wait();
    }
}

/// Lance l'application d'essai sous le nom de processus `nom` : `appli-essai` telle quelle, tout
/// autre nom par une copie `<nom>.exe` dans son profil. Elle reçoit le dossier de sa WebView2 et
/// son fichier d'événements dans ce profil, puis `options`. Panique si `appli-essai.exe` manque, ou
/// si la ligne `pret` ne paraît pas en 20 s, avec les lignes lues.
pub fn lancer_appli(nom: &str, options: &[&str]) -> Appli {
    exiger_la_ci();
    let source = appli_essai();
    let profil = Profil::neuf(nom);
    let exe = if nom == "appli-essai" {
        source
    } else {
        let copie = profil.racine.join(format!("{nom}.exe"));
        fs::copy(&source, &copie).unwrap_or_else(|e| panic!("{} : {e}", copie.display()));
        copie
    };
    let evenements = profil.racine.join("evenements.txt");
    let enfant = Command::new(&exe)
        .arg(profil.racine.join("webview2"))
        .arg(&evenements)
        .args(options)
        .envs(profil.variables())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(fichier_d_erreur(&profil))
        .spawn()
        .unwrap_or_else(|e| panic!("lancement de {} : {e}", exe.display()));
    let appli = Appli {
        nom: nom.to_string(),
        pid: enfant.id(),
        evenements,
        enfant,
        profil,
    };
    assert!(
        appli.attendre("\"genre\":\"pret\"", PRET_MAX),
        "{nom} n'a pas dit « pret » en 20 s ({}) ; lignes lues : {:?}",
        appli.profil.racine.display(),
        appli.lignes()
    );
    appli
}

/// Un événement de la page, lu d'une ligne du fichier d'événements : le titre de sa fenêtre, puis
/// les champs de son JSON, chacun vide s'il manque.
#[derive(Clone, Debug)]
pub struct Evenement {
    pub titre: String,
    pub genre: String,
    pub id: String,
    pub key: String,
    pub code: String,
    pub valeur: String,
}

impl Evenement {
    /// L'événement que dit `ligne`, `<titre>\t<JSON>` ; `None` pour une ligne d'une autre forme
    /// (`vole`, `vol manqué`, `verrou pris`).
    fn lire(ligne: &str) -> Option<Evenement> {
        let (titre, json) = ligne.split_once('\t')?;
        let champs = objet_plat(json)?;
        let champ = |cle: &str| {
            champs
                .iter()
                .find(|(c, _)| c == cle)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        Some(Evenement {
            titre: titre.to_string(),
            genre: champ("genre"),
            id: champ("id"),
            key: champ("key"),
            code: champ("code"),
            valeur: champ("valeur"),
        })
    }
}

/// Les champs d'un objet JSON plat et sans blancs, `{"clé":valeur,…}`, tel que l'écrit
/// `JSON.stringify`, dans l'ordre : chaque valeur est une chaîne, décodée, ou un littéral (`true`,
/// `false`, un nombre), tel quel. `None` pour tout autre texte.
fn objet_plat(json: &str) -> Option<Vec<(String, String)>> {
    let mut lu = json.chars().peekable();
    let mut champs = Vec::new();
    if lu.next()? != '{' {
        return None;
    }
    if lu.next_if_eq(&'}').is_some() {
        return lu.next().is_none().then_some(champs);
    }
    loop {
        if lu.next()? != '"' {
            return None;
        }
        let cle = chaine_json(&mut lu)?;
        if lu.next()? != ':' {
            return None;
        }
        let valeur = if lu.next_if_eq(&'"').is_some() {
            chaine_json(&mut lu)?
        } else {
            let mut litteral = String::new();
            while let Some(c) = lu.next_if(|c| *c != ',' && *c != '}') {
                litteral.push(c);
            }
            litteral
        };
        champs.push((cle, valeur));
        match lu.next()? {
            ',' => {}
            '}' => return lu.next().is_none().then_some(champs),
            _ => return None,
        }
    }
}

/// La chaîne JSON qui suit son guillemet ouvrant, lue jusqu'à son guillemet fermant, et décodée :
/// les échappements, `\uXXXX` compris, dont une paire de substitution forme un seul caractère.
fn chaine_json(lu: &mut Peekable<Chars>) -> Option<String> {
    let mut unites: Vec<u16> = Vec::new();
    let mut tampon = [0u16; 2];
    loop {
        let c = match lu.next()? {
            '"' => return Some(String::from_utf16_lossy(&unites)),
            '\\' => match lu.next()? {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{8}',
                'f' => '\u{c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => {
                    let chiffres = (0..4).map(|_| lu.next()).collect::<Option<String>>()?;
                    unites.push(u16::from_str_radix(&chiffres, 16).ok()?);
                    continue;
                }
                _ => return None,
            },
            c => c,
        };
        unites.extend_from_slice(c.encode_utf16(&mut tampon));
    }
}

/// Le teneur du verrou des gestes : `appli-essai --tenir-le-verrou`, son profil, et un handle de
/// `Local\pilote-gestes` ouvert par les essais. Ce handle garde le mutex en vie quand le teneur
/// meurt : abandonné, il attend `pilote`, qui le reçoit en `WAIT_ABANDONED`. `Drop` tue le teneur
/// s'il vit encore, l'attend, puis ferme le handle.
pub struct Teneur {
    enfant: Child,
    profil: Profil,
    verrou: HANDLE,
}

impl Teneur {
    /// Tue le teneur et l'attend : le verrou tombe abandonné.
    pub fn tuer(&mut self) {
        exiger_la_ci();
        let _ = self.enfant.kill();
        self.enfant.wait().expect("l'attente du teneur");
    }
}

impl Drop for Teneur {
    fn drop(&mut self) {
        if let Ok(None) = self.enfant.try_wait() {
            let _ = self.enfant.kill();
        }
        let _ = self.enfant.wait();
        if !self.verrou.is_invalid() {
            // SAFETY: le handle a été ouvert par `OpenMutexW` pour ce seul teneur, qui le ferme une
            // fois.
            let _ = unsafe { CloseHandle(self.verrou) };
        }
    }
}

/// Lance exactement `appli-essai --tenir-le-verrou <profil.racine>\verrou.txt` dans un profil neuf,
/// attend la ligne `verrou pris` dans ce fichier, 10 s au plus, par son contenu (le teneur le crée
/// vide avant de prendre le verrou), puis ouvre `Local\pilote-gestes` sans jamais l'attendre.
/// Panique, avec ce qu'il a lu, si la ligne ne paraît pas ; panique si le mutex ne s'ouvre pas.
pub fn tenir_le_verrou() -> Teneur {
    exiger_la_ci();
    let exe = appli_essai();
    let profil = Profil::neuf("teneur");
    let fichier = profil.racine.join("verrou.txt");
    let enfant = Command::new(&exe)
        .arg("--tenir-le-verrou")
        .arg(&fichier)
        .envs(profil.variables())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(fichier_d_erreur(&profil))
        .spawn()
        .unwrap_or_else(|e| panic!("lancement de {} : {e}", exe.display()));
    let mut teneur = Teneur {
        enfant,
        profil,
        verrou: HANDLE::default(),
    };
    let lire = || fs::read_to_string(&fichier).unwrap_or_default();
    let pris = sonder(VERROU_MAX, || lire().lines().any(|l| l == "verrou pris"));
    assert!(
        pris,
        "le teneur n'a pas écrit « verrou pris » en 10 s ({}) ; lu : {:?}",
        teneur.profil.racine.display(),
        lire()
    );
    // SAFETY: le nom est une constante terminée par NUL ; le handle rendu appartient au teneur, qui
    // le ferme.
    teneur.verrou = unsafe {
        OpenMutexW(
            SYNCHRONIZATION_SYNCHRONIZE,
            false,
            w!("Local\\pilote-gestes"),
        )
    }
    .unwrap_or_else(|e| panic!("OpenMutexW : {e}"));
    teneur
}

/// Ce qu'a rendu un processus : son code de sortie (-1 sans code), ses deux sorties et sa durée,
/// du lancement à la fin.
pub struct Sortie {
    pub code: i32,
    pub sortie: Vec<u8>,
    pub erreur: Vec<u8>,
    pub duree: Duration,
}

impl Sortie {
    /// La sortie d'erreur, en texte, sans son saut de ligne final.
    pub fn message(&self) -> String {
        exiger_la_ci();
        String::from_utf8_lossy(&self.erreur)
            .trim_end_matches(['\r', '\n'])
            .to_string()
    }

    /// Les lignes de la sortie standard.
    pub fn lignes(&self) -> Vec<String> {
        exiger_la_ci();
        String::from_utf8_lossy(&self.sortie)
            .lines()
            .map(String::from)
            .collect()
    }
}

/// Lance `pilote` sur `args`, dans `profil`, `entree` sur son entrée standard (rien sinon), et rend
/// ce qu'il a rendu.
pub fn pilote(profil: &Profil, args: &[&str], entree: Option<&[u8]>) -> Sortie {
    exiger_la_ci();
    let mut commande = Command::new(env!("CARGO_BIN_EXE_pilote"));
    commande.args(args).envs(profil.variables());
    executer(commande, entree)
}

/// Un shell, lancé par son chemin absolu : `Command::new` chercherait d'abord dans `System32`, où le
/// `bash.exe` de WSL répond.
pub enum Shell {
    /// `%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe`.
    Powershell51,
    /// `%ProgramFiles%\Git\bin\bash.exe`.
    GitBash,
}

/// Lance `script` par `shell`, dans `profil` (`powershell.exe -NoProfile -Command <script>`, ou
/// `bash.exe -c <script>`), et rend ce qu'il a rendu ; panique si le shell manque. Le script
/// vérifie lui-même quel shell lui répond.
pub fn shell(shell: Shell, script: &str, profil: &Profil) -> Sortie {
    exiger_la_ci();
    let (programme, options) = match shell {
        Shell::Powershell51 => (
            dossier_de_la_variable("SystemRoot")
                .join(r"System32\WindowsPowerShell\v1.0\powershell.exe"),
            &["-NoProfile", "-Command"][..],
        ),
        Shell::GitBash => (
            dossier_de_la_variable("ProgramFiles").join(r"Git\bin\bash.exe"),
            &["-c"][..],
        ),
    };
    assert!(programme.is_file(), "{} manque", programme.display());
    let mut commande = Command::new(&programme);
    commande.args(options).arg(script).envs(profil.variables());
    executer(commande, None)
}

/// Le dossier que nomme la variable d'environnement `nom` ; panique si elle manque.
fn dossier_de_la_variable(nom: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(nom).unwrap_or_else(|| panic!("{nom} manque")))
}

/// Lance `commande`, ses deux sorties recueillies, `entree` écrite puis fermée sur son entrée
/// standard (rien sinon), attend sa fin, et rend ce qu'elle a rendu.
fn executer(mut commande: Command, entree: Option<&[u8]>) -> Sortie {
    commande
        .stdin(if entree.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let debut = Instant::now();
    let mut enfant = commande
        .spawn()
        .unwrap_or_else(|e| panic!("lancement de {:?} : {e}", commande.get_program()));
    if let Some(octets) = entree {
        let mut flux = enfant.stdin.take().expect("l'entrée standard de l'enfant");
        flux.write_all(octets)
            .expect("l'écriture de l'entrée standard de l'enfant");
    }
    let fin = enfant.wait_with_output().expect("l'attente de l'enfant");
    Sortie {
        code: fin.status.code().unwrap_or(-1),
        sortie: fin.stdout,
        erreur: fin.stderr,
        duree: debut.elapsed(),
    }
}

/// Exige que la disposition du clavier de l'application soit la française, par cinq preuves, dans
/// cet ordre, chacune avec sa panique : `pilote activer` sort en 0 ; le premier plan est à
/// l'application ; une fenêtre y est focalisée ; elle est à `msedgewebview2.exe`, dont le fil donne
/// la disposition (décision 2) ; et cette disposition est 0x040C040C, où `@` et `/` sont à leur
/// place. La fenêtre focalisée passe dans la WebView2 un instant après l'activation : la troisième
/// et la quatrième preuve se sondent, 2 s au plus. `Set-WinUserLanguageList` ne dit pas quand la
/// disposition vaut pour les processus lancés ensuite : les essais du clavier l'exigent avant de
/// juger.
pub fn exiger_la_disposition_francaise(appli: &Appli, profil: &Profil) {
    exiger_la_ci();
    let activer = pilote(profil, &["activer", "--fenetre", &appli.nom], None);
    assert!(
        activer.code == 0,
        "activer a rendu {} : {}",
        activer.code,
        activer.message()
    );
    // SAFETY: `GetForegroundWindow` ne prend rien.
    let premier_plan = pid_de(unsafe { GetForegroundWindow() });
    assert!(
        premier_plan == appli.pid,
        "premier plan sur le PID {premier_plan}, pas sur l'appli {}",
        appli.pid
    );
    let mut focus = HWND::default();
    let dans_la_webview2 = sonder(FOCUS_MAX, || {
        focus = fenetre_focalisee();
        !focus.is_invalid()
            && nom_du_processus(pid_de(focus)).is_some_and(|n| n.eq_ignore_ascii_case(WEBVIEW2))
    });
    assert!(!focus.is_invalid(), "aucune fenêtre focalisée dans l'appli");
    assert!(
        dans_la_webview2,
        "fenêtre focalisée hors de msedgewebview2 : {} {}",
        classe(focus),
        nom_du_processus(pid_de(focus)).unwrap_or_default()
    );
    // SAFETY: lectures d'une fenêtre, de la disposition de son fil, puis de cette disposition.
    let fil = unsafe { GetWindowThreadProcessId(focus, None) };
    let hkl = unsafe { GetKeyboardLayout(fil) };
    let arobase = unsafe { VkKeyScanExW(u16::from(b'@'), hkl) };
    let barre = unsafe { VkKeyScanExW(u16::from(b'/'), hkl) };
    let disposition = hkl.0 as usize;
    assert!(
        disposition == HKL_FRANCAIS && arobase == AROBASE_FRANCAIS && barre == BARRE_FRANCAISE,
        "disposition non française sur le runner : HKL 0x{disposition:08X}, '@' 0x{:04X}, '/' 0x{:04X}",
        arobase as u16,
        barre as u16
    );
}

/// Vrai si une fenêtre de premier niveau du processus `pid`, visible et titrée, est réduite.
pub fn est_reduite(pid: u32) -> bool {
    exiger_la_ci();
    fenetres_de_premier_niveau().into_iter().any(|h| {
        // SAFETY: lecture d'une fenêtre ; une fenêtre détruite depuis rend faux.
        pid_de(h) == pid && visible(h) && !titre(h).is_empty() && unsafe { IsIconic(h) }.as_bool()
    })
}

/// Trouve le cadre des Paramètres, une fenêtre de premier niveau visible et titrée de classe
/// `ApplicationFrameWindow`, par son AUMID ou par son enfant `SystemSettings.exe`, qui se détache
/// une fois le cadre réduit ; lit son titre, le réduit (`ShowWindow`, `SW_MINIMIZE`), et rend ce
/// titre. `None` s'il ne le trouve pas.
pub fn reduire_les_parametres() -> Option<String> {
    exiger_la_ci();
    // L'AUMID se lit par une interface COM, qui demande COM sur ce fil ; S_FALSE et
    // RPC_E_CHANGED_MODE disent COM déjà là, et un échec laisse l'AUMID illisible.
    // SAFETY: aucun argument réservé.
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let cadre = fenetres_de_premier_niveau().into_iter().find(|&h| {
        visible(h)
            && classe(h).eq_ignore_ascii_case(CLASSE_CADRE)
            && !titre(h).is_empty()
            && (aumid(h).is_some_and(|a| a.eq_ignore_ascii_case(AUMID_PARAMETRES))
                || enfants(h).into_iter().any(|e| {
                    nom_du_processus(pid_de(e)).is_some_and(|n| n.eq_ignore_ascii_case(PARAMETRES))
                }))
    })?;
    let lu = titre(cadre);
    // SAFETY: une fenêtre détruite depuis rend faux, sans suite.
    let _ = unsafe { ShowWindow(cadre, SW_MINIMIZE) };
    Some(lu)
}

/// Vrai si une fenêtre de premier niveau visible et sans titre est au processus `processus`, nommé
/// sans `.exe` : une liste déroulante ouverte dans la WebView2 en est une.
pub fn fenetre_sans_titre_de(processus: &str) -> bool {
    exiger_la_ci();
    let exe = format!("{processus}.exe");
    fenetres_de_premier_niveau().into_iter().any(|h| {
        visible(h)
            && titre(h).is_empty()
            && nom_du_processus(pid_de(h)).is_some_and(|n| n.eq_ignore_ascii_case(&exe))
    })
}

/// Verr. Maj allumée le temps d'un essai : `Drop` l'éteint.
pub struct VerrMaj;

impl VerrMaj {
    /// Allume Verr. Maj, si elle ne l'est pas, et attend qu'elle le soit, 2 s au plus.
    pub fn allumer() -> VerrMaj {
        exiger_la_ci();
        if !verr_maj_allumee() {
            basculer_verr_maj();
        }
        assert!(
            sonder(VERR_MAJ_MAX, verr_maj_allumee),
            "Verr. Maj ne s'allume pas"
        );
        VerrMaj
    }
}

impl Drop for VerrMaj {
    fn drop(&mut self) {
        if verr_maj_allumee() {
            basculer_verr_maj();
        }
    }
}

/// Vrai si Verr. Maj est allumée, lue par `GetKeyState` sur un fil neuf : la file d'entrée d'un fil
/// naît avec l'état du clavier de ce moment, et celle d'un fil déjà vivant ne le suit qu'en lisant
/// ses messages, ce que les essais ne font pas.
fn verr_maj_allumee() -> bool {
    // SAFETY: lecture de l'état d'une touche.
    thread::spawn(|| unsafe { GetKeyState(i32::from(VK_CAPITAL.0)) } & 1 != 0)
        .join()
        .unwrap_or(false)
}

/// Frappe Verr. Maj, enfoncée puis relâchée, d'un seul envoi.
fn basculer_verr_maj() {
    let touche = |drapeaux: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_CAPITAL,
                wScan: 0x3A,
                dwFlags: drapeaux,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let lot = [touche(KEYBD_EVENT_FLAGS(0)), touche(KEYEVENTF_KEYUP)];
    let taille = i32::try_from(size_of::<INPUT>()).expect("la taille d'un INPUT");
    // SAFETY: chaque `INPUT` porte la variante du clavier que dit son type ; la taille passée est
    // celle d'un `INPUT`.
    let inseres = unsafe { SendInput(&lot, taille) };
    assert_eq!(inseres, 2, "SendInput n'a pas frappé Verr. Maj");
}

/// La fenêtre focalisée du fil au premier plan (`GetGUIThreadInfo`) ; nulle si elle ne se lit pas.
fn fenetre_focalisee() -> HWND {
    let mut info = GUITHREADINFO {
        cbSize: u32::try_from(size_of::<GUITHREADINFO>()).expect("la taille de GUITHREADINFO"),
        ..Default::default()
    };
    // SAFETY: `info` porte sa taille dans `cbSize` ; en erreur, ses fenêtres restent nulles.
    match unsafe { GetGUIThreadInfo(0, &raw mut info) } {
        Ok(()) => info.hwndFocus,
        Err(_) => HWND::default(),
    }
}

/// Les fenêtres de premier niveau, dans l'ordre d'`EnumWindows`.
fn fenetres_de_premier_niveau() -> Vec<HWND> {
    let mut liste: Vec<HWND> = Vec::new();
    // SAFETY: `collecter` ne reçoit que le pointeur vers `liste`, qui vit jusqu'au retour
    // d'`EnumWindows`, lequel n'appelle le rappel que pendant l'appel.
    unsafe { EnumWindows(Some(collecter), LPARAM(&raw mut liste as isize)) }
        .unwrap_or_else(|e| panic!("EnumWindows : {e}"));
    liste
}

/// Les fenêtres enfants de `h`, à toute profondeur ; l'énumération part de `h`, jamais du bureau.
fn enfants(h: HWND) -> Vec<HWND> {
    let mut liste: Vec<HWND> = Vec::new();
    // SAFETY: comme pour `fenetres_de_premier_niveau` ; le retour ne porte rien.
    let _ = unsafe { EnumChildWindows(Some(h), Some(collecter), LPARAM(&raw mut liste as isize)) };
    liste
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

/// Le processus de la fenêtre `h` ; 0 si elle n'existe plus.
fn pid_de(h: HWND) -> u32 {
    let mut pid = 0u32;
    // SAFETY: `pid` vit le temps de l'appel.
    unsafe { GetWindowThreadProcessId(h, Some(&raw mut pid)) };
    pid
}

/// Vrai si la fenêtre `h` est visible.
fn visible(h: HWND) -> bool {
    // SAFETY: lecture d'une fenêtre ; une fenêtre détruite rend faux.
    unsafe { IsWindowVisible(h) }.as_bool()
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
    let mut tampon = [0u16; 257];
    // SAFETY: `tampon` porte sa longueur, que `GetClassNameW` respecte.
    let lus = unsafe { GetClassNameW(h, &mut tampon) };
    let lus = usize::try_from(lus).unwrap_or(0).min(tampon.len());
    String::from_utf16_lossy(&tampon[..lus])
}

/// Le nom de l'exécutable du processus `pid`, `.exe` compris ; `None` s'il ne se lit pas.
fn nom_du_processus(pid: u32) -> Option<String> {
    // SAFETY: le handle ouvert se ferme sur tous les chemins ; `tampon` porte sa longueur dans
    // `longueur`, que `QueryFullProcessImageNameW` respecte et réécrit.
    let processus = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut tampon = [0u16; 1024];
    let mut longueur = u32::try_from(tampon.len()).unwrap_or(0);
    let lu = unsafe {
        QueryFullProcessImageNameW(
            processus,
            PROCESS_NAME_WIN32,
            PWSTR(tampon.as_mut_ptr()),
            &raw mut longueur,
        )
    };
    let _ = unsafe { CloseHandle(processus) };
    lu.ok()?;
    let longueur = usize::try_from(longueur).unwrap_or(0).min(tampon.len());
    let chemin = PathBuf::from(String::from_utf16_lossy(&tampon[..longueur]));
    chemin
        .file_name()
        .map(|nom| nom.to_string_lossy().into_owned())
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
