//! L'application d'essai de `pilote` : la cible des essais de bout en bout, sous Windows. Elle
//! reçoit `<dossier WebView2> <fichier d'événements>`, puis ses options, et ouvre la fenêtre
//! « Appli d'essai », dont la page dit chaque événement dans le fichier, une ligne par événement,
//! `<titre de la fenêtre>\t<JSON>` ; la boucle y dit de même chaque touche que reçoit une fenêtre
//! elle-même, hors de sa page. `appli-essai --tenir-le-verrou <fichier>` est un mode à part,
//! sans fenêtre, qui tient le verrou des gestes. Ailleurs que sous Windows, elle dit qu'elle n'y
//! tourne pas, et sort en 64.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("appli-essai ne tourne que sous Windows");
    std::process::ExitCode::from(64)
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    fenetre::lancer()
}

/// Les fenêtres, la page et l'IPC (wry, tao), les options, et le mode du teneur.
#[cfg(windows)]
mod fenetre {
    use super::regles::{self, Compteur, Message, Mode, Options};
    use std::{
        borrow::Cow,
        cell::RefCell,
        collections::HashMap,
        ffi::OsString,
        fs::File,
        io::{Read, Write},
        path::{Path, PathBuf},
        process::ExitCode,
        rc::Rc,
        time::{Duration, Instant},
    };
    use tao::event::{ElementState, Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
    use tao::keyboard::Key;
    use tao::platform::windows::WindowExtWindows;
    use tao::window::{Window, WindowBuilder, WindowId};
    use windows::Win32::Foundation::{HWND, WAIT_ABANDONED, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{CreateMutexW, INFINITE, WaitForSingleObject};
    use windows::Win32::UI::WindowsAndMessaging::{
        ASFW_ANY, AllowSetForegroundWindow, GetForegroundWindow, SetForegroundWindow,
    };
    use windows::core::w;
    use wry::http::{Request, Response, header::CONTENT_TYPE};
    use wry::{WebContext, WebView, WebViewBuilder, WebViewBuilderExtWindows};

    /// La page, servie par le protocole propre `essai://` : `with_html` donnerait une origine
    /// nulle, où l'IPC peut se perdre sans bruit.
    const PAGE: &str = include_str!("page.html");

    /// Le titre de la fenêtre principale.
    const TITRE: &str = "Appli d'essai";

    /// Le titre du dialogue, une seconde fenêtre sans propriétaire.
    const TITRE_DIALOGUE: &str = "Dialogue d'essai";

    /// Le pas du voleur : il sonde le fichier de sa cible, puis demande le premier plan, à ce pas.
    const PAS: Duration = Duration::from_millis(20);

    /// Le plus long que le voleur demande le premier plan.
    const VOL_MAX: Duration = Duration::from_secs(5);

    /// Le fichier d'événements, un `File` non tamponné que les fenêtres partagent sur le fil de la
    /// boucle.
    type Journal = Rc<RefCell<File>>;

    /// Une fenêtre et sa vue, que la boucle garde vivantes jusqu'à leur fermeture.
    type Ouverte = (Window, WebView);

    /// Ce que la page demande à la boucle, par l'IPC.
    enum Ordre {
        /// Ouvrir le dialogue.
        Ouvrir,
        /// Dormir tant de secondes sur le fil de la boucle : l'hôte ne répond plus.
        Geler(u64),
        /// Un `keydown` reçu : le droit au premier plan, tombé à cette entrée, se renouvelle.
        Touche,
    }

    /// Lit les arguments et lance le mode qu'ils demandent ; un refus de l'analyse sort en 64,
    /// l'échec d'un appel au système en 1.
    pub fn lancer() -> ExitCode {
        let args: Vec<OsString> = std::env::args_os().skip(1).collect();
        match regles::analyser(&args) {
            Ok(Mode::Teneur(fichier)) => tenir_le_verrou(&fichier),
            Ok(Mode::Fenetres(options)) => fenetres(options),
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(64)
            }
        }
    }

    /// Le mode du teneur, sans boucle tao : il prend `Local\pilote-gestes`, écrit `verrou pris`
    /// dès que l'attente le donne, libre ou abandonné, puis dort sans fin. Le handle ne se ferme
    /// jamais : le verrou tombe abandonné à la mort du processus.
    fn tenir_le_verrou(fichier: &Path) -> ExitCode {
        let mut sortie = match File::create(fichier) {
            Ok(f) => f,
            Err(e) => return echec(&format!("{} : {e}", fichier.display())),
        };
        // SAFETY: le nom est une constante terminée par NUL ; le handle reste ouvert jusqu'à la
        // mort du processus.
        let poignee = match unsafe { CreateMutexW(None, false, w!("Local\\pilote-gestes")) } {
            Ok(p) => p,
            Err(e) => return echec(&format!("CreateMutexW : {e}")),
        };
        // SAFETY: le handle est ouvert.
        let issue = unsafe { WaitForSingleObject(poignee, INFINITE) };
        if issue != WAIT_OBJECT_0 && issue != WAIT_ABANDONED {
            return echec(&format!("WaitForSingleObject : {}", issue.0));
        }
        if let Err(e) = sortie.write_all(b"verrou pris\n") {
            return echec(&format!("{} : {e}", fichier.display()));
        }
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }

    /// Dit `message`, et rend le code 1.
    fn echec(message: &str) -> ExitCode {
        eprintln!("{message}");
        ExitCode::from(1)
    }

    /// Les fenêtres : la principale, le dialogue s'il est demandé, puis la boucle tao, que la
    /// dernière fenêtre fermée arrête.
    fn fenetres(options: Options) -> ExitCode {
        let journal: Journal = match File::create(&options.evenements) {
            Ok(f) => Rc::new(RefCell::new(f)),
            Err(e) => return echec(&format!("{} : {e}", options.evenements.display())),
        };
        let permettre = options.permettre;
        if permettre {
            permettre_le_premier_plan(&journal);
        }
        let event_loop = EventLoopBuilder::<Ordre>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let mut contexte = WebContext::new(Some(options.dossier.clone()));
        let focus = options.focus;
        let visible = options.voler.is_none();
        let principale = match ouvrir(
            TITRE,
            visible,
            &event_loop,
            &mut contexte,
            &proxy,
            &journal,
            &focus,
        ) {
            Ok(o) => o,
            Err(e) => return echec(&format!("{TITRE} : {e}")),
        };
        if options.reduite {
            principale.0.set_minimized(true);
        }
        let id_principale = principale.0.id();
        let mut vol = options
            .voler
            .map(|(cible, n)| Vol::nouveau(cible, n, hwnd_de(&principale.0)));
        let mut ouvertes: HashMap<WindowId, Ouverte> = HashMap::new();
        ouvertes.insert(id_principale, principale);
        if options.dialogue {
            match ouvrir(
                TITRE_DIALOGUE,
                true,
                &event_loop,
                &mut contexte,
                &proxy,
                &journal,
                &focus,
            ) {
                Ok(o) => {
                    ouvertes.insert(o.0.id(), o);
                }
                Err(e) => return echec(&format!("{TITRE_DIALOGUE} : {e}")),
            }
        }
        event_loop.run(move |evenement, cible, flot| {
            *flot = ControlFlow::Wait;
            match evenement {
                Event::UserEvent(Ordre::Ouvrir) => match ouvrir(
                    TITRE_DIALOGUE,
                    true,
                    cible,
                    &mut contexte,
                    &proxy,
                    &journal,
                    &focus,
                ) {
                    Ok(o) => {
                        ouvertes.insert(o.0.id(), o);
                    }
                    Err(e) => eprintln!("{TITRE_DIALOGUE} : {e}"),
                },
                Event::UserEvent(Ordre::Geler(secondes)) => {
                    std::thread::sleep(Duration::from_secs(secondes));
                }
                Event::UserEvent(Ordre::Touche) if permettre => {
                    permettre_le_premier_plan(&journal);
                }
                // Une touche reçue par la fenêtre elle-même, avant que wry ait porté le focus dans la
                // WebView2 : la page ne la voit jamais, et seule cette ligne la dit. Une touche que
                // tao fabrique à un changement de focus, d'après l'état du clavier, n'a été reçue
                // par personne, et ne s'écrit pas.
                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::KeyboardInput {
                            event,
                            is_synthetic: false,
                            ..
                        },
                    ..
                } => {
                    if let Some((fenetre, _)) = ouvertes.get(&window_id) {
                        let corps = regles::touche_fenetre(
                            &nom_de_touche(&event.logical_key),
                            event.state == ElementState::Pressed,
                        );
                        ecrire(&journal, &regles::ligne(&fenetre.title(), &corps));
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    ouvertes.remove(&window_id);
                    if ouvertes.is_empty() {
                        *flot = ControlFlow::Exit;
                        return;
                    }
                }
                _ => {}
            }
            let principale = ouvertes.get(&id_principale).map(|(f, _)| f);
            if let Some(v) = vol.as_mut()
                && let Some(prochain) = v.avancer(Instant::now(), principale, &journal)
            {
                *flot = ControlFlow::WaitUntil(prochain);
            }
        })
    }

    /// Ouvre la fenêtre `titre`, visible ou cachée, et sa vue, qui charge la page par `essai://`,
    /// l'élément `focus` focalisé. Le gestionnaire de l'IPC traduit `ouvrir` et `geler <s>` en
    /// ordres, et écrit tout autre message dans le fichier d'événements, précédé du titre ; un
    /// `keydown` envoie de plus `Ordre::Touche`. `Window::set_focus()` n'est jamais appelé : il
    /// injecterait Alt par `SendInput`.
    fn ouvrir(
        titre: &str,
        visible: bool,
        cible: &EventLoopWindowTarget<Ordre>,
        contexte: &mut WebContext,
        proxy: &EventLoopProxy<Ordre>,
        journal: &Journal,
        focus: &str,
    ) -> Result<Ouverte, String> {
        let fenetre = WindowBuilder::new()
            .with_title(titre)
            .with_visible(visible)
            .build(cible)
            .map_err(|e| e.to_string())?;
        let (proxy, journal, titre) = (proxy.clone(), Rc::clone(journal), titre.to_owned());
        let vue = WebViewBuilder::new_with_web_context(contexte)
            .with_custom_protocol("essai".into(), |_id, _requete| {
                Response::builder()
                    .header(CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Cow::Borrowed(PAGE.as_bytes()))
                    .unwrap()
            })
            .with_url(format!("essai://localhost/?focus={focus}"))
            .with_browser_accelerator_keys(false)
            .with_default_context_menus(false)
            .with_devtools(false)
            .with_ipc_handler(move |requete: Request<String>| {
                let corps = requete.body();
                match regles::classer(corps) {
                    Message::Ouvrir => {
                        let _ = proxy.send_event(Ordre::Ouvrir);
                    }
                    Message::Geler(secondes) => {
                        let _ = proxy.send_event(Ordre::Geler(secondes));
                    }
                    Message::Rien => {}
                    Message::Evenement { touche } => {
                        if touche {
                            let _ = proxy.send_event(Ordre::Touche);
                        }
                        ecrire(&journal, &regles::ligne(&titre, corps));
                    }
                }
            })
            .build(&fenetre)
            .map_err(|e| e.to_string())?;
        Ok((fenetre, vue))
    }

    /// Écrit `texte`, une ligne entière, en un seul `write_all` sur le `File` non tamponné : les
    /// essais lisent le fichier pendant que l'application tourne, et aucun tampon ne la retient.
    fn ecrire(journal: &Journal, texte: &str) {
        let _ = journal.borrow_mut().write_all(texte.as_bytes());
    }

    /// `AllowSetForegroundWindow(ASFW_ANY)`, puis la ligne `permis` qui dit son résultat : le droit
    /// tombe à chaque entrée du clavier, et se renouvelle à chaque `keydown` reçu.
    fn permettre_le_premier_plan(journal: &Journal) {
        // SAFETY: l'appel ne prend qu'un entier.
        let accorde = unsafe { AllowSetForegroundWindow(ASFW_ANY) }.is_ok();
        ecrire(journal, &regles::ligne(TITRE, &regles::permis(accorde)));
    }

    /// Le nom de la touche logique `touche` : son texte pour un caractère, son nom pour une touche
    /// nommée (`Tab`, `Enter`, `Escape`), ou ce que tao en dit sinon.
    fn nom_de_touche(touche: &Key<'static>) -> String {
        match touche {
            Key::Character(texte) => (*texte).to_string(),
            autre => format!("{autre:?}"),
        }
    }

    /// Le handle Win32 de `fenetre`.
    fn hwnd_de(fenetre: &Window) -> HWND {
        HWND(fenetre.hwnd() as *mut core::ffi::c_void)
    }

    /// Où en est le voleur.
    #[derive(Clone, Copy)]
    enum EtatVol {
        /// Il sonde le fichier de sa cible.
        Guette,
        /// Montré, il demande le premier plan jusqu'à `fin`.
        Demande { fin: Instant },
        /// Il a écrit sa dernière ligne.
        Fini,
    }

    /// Le voleur du premier plan (`--voler-premier-plan <fichier> <n>`) : caché au départ, il
    /// sonde à chaque pas le fichier d'événements de sa cible ; dès la n-ième ligne de saisie, il se
    /// montre et demande le premier plan à chaque pas, jusqu'à ce que `GetForegroundWindow` le
    /// rende, 5 s au plus. Il écrit alors `vole`, ou `vol manqué` si les 5 s passent sans lui.
    struct Vol {
        cible: PathBuf,
        /// Le fichier de la cible, ouvert à la première sonde qui le trouve, puis lu à la suite.
        fichier: Option<File>,
        compteur: Compteur,
        n: usize,
        hwnd: HWND,
        etat: EtatVol,
        /// L'instant du pas suivant.
        prochain: Instant,
    }

    impl Vol {
        fn nouveau(cible: PathBuf, n: usize, hwnd: HWND) -> Vol {
            Vol {
                cible,
                fichier: None,
                compteur: Compteur::default(),
                n,
                hwnd,
                etat: EtatVol::Guette,
                prochain: Instant::now(),
            }
        }

        /// Fait le pas, s'il est l'heure, et rend l'instant du suivant ; `None` quand le vol est
        /// fini. `principale` est la fenêtre à montrer, tant qu'elle n'est pas fermée.
        fn avancer(
            &mut self,
            maintenant: Instant,
            principale: Option<&Window>,
            journal: &Journal,
        ) -> Option<Instant> {
            match self.etat {
                EtatVol::Fini => return None,
                _ if maintenant < self.prochain => return Some(self.prochain),
                EtatVol::Guette => {
                    if self.saisies() >= self.n {
                        if let Some(f) = principale {
                            f.set_visible(true);
                        }
                        self.demander();
                        self.etat = EtatVol::Demande {
                            fin: maintenant + VOL_MAX,
                        };
                    }
                }
                EtatVol::Demande { fin } => {
                    // SAFETY: l'appel ne prend rien.
                    if unsafe { GetForegroundWindow() } == self.hwnd {
                        ecrire(journal, "vole\n");
                        self.etat = EtatVol::Fini;
                        return None;
                    }
                    if maintenant >= fin {
                        ecrire(journal, "vol manqué\n");
                        self.etat = EtatVol::Fini;
                        return None;
                    }
                    self.demander();
                }
            }
            self.prochain = maintenant + PAS;
            Some(self.prochain)
        }

        /// Le compte des saisies de la cible, après lecture de ce qu'elle a écrit depuis la sonde
        /// d'avant ; un fichier encore absent ou illisible ne compte rien de plus.
        fn saisies(&mut self) -> usize {
            if self.fichier.is_none() {
                self.fichier = File::open(&self.cible).ok();
            }
            let mut lu = Vec::new();
            if let Some(f) = self.fichier.as_mut() {
                let _ = f.read_to_end(&mut lu);
            }
            self.compteur.lire(&lu)
        }

        /// Demande le premier plan ; Windows peut refuser sans le dire, d'où la répétition.
        fn demander(&self) {
            // SAFETY: le handle est celui de la fenêtre principale ; un handle mort fait refuser.
            let _ = unsafe { SetForegroundWindow(self.hwnd) };
        }
    }
}

/// Les règles de l'application qui ne demandent aucune fenêtre : la lecture des arguments, la
/// forme des lignes du fichier d'événements, le tri des messages de la page et le compte des
/// saisies que guette le voleur. Elles se compilent sans Windows ; leurs tests portent `#[ignore]`,
/// comme tout test hors du domaine, et ne tournent qu'à l'étape de la sentinelle du job `windows`.
#[cfg(any(windows, test))]
mod regles {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::slice::Iter;

    /// Ce que dit une ligne de commande sans `<dossier> <fichier>`.
    pub const USAGE: &str = "usage : appli-essai <dossier> <fichier> [options], ou appli-essai \
                             --tenir-le-verrou <fichier>";

    /// Ce que demande la ligne de commande.
    #[derive(Debug, PartialEq)]
    pub enum Mode {
        /// `--tenir-le-verrou <fichier>` : tenir `Local\pilote-gestes`, sans fenêtre.
        Teneur(PathBuf),
        /// `<dossier> <fichier>` puis les options : les fenêtres.
        Fenetres(Options),
    }

    /// Les fenêtres demandées.
    #[derive(Debug, PartialEq)]
    pub struct Options {
        pub dossier: PathBuf,
        pub evenements: PathBuf,
        pub dialogue: bool,
        pub reduite: bool,
        pub focus: String,
        pub permettre: bool,
        pub voler: Option<(PathBuf, usize)>,
    }

    /// L'élément qui a le focus au chargement, sans `--focus`.
    const FOCUS_PAR_DEFAUT: &str = "depart";

    /// Ce que porte le JSON d'un `keydown` de la page.
    const TOUCHE: &str = "\"genre\":\"touche\"";

    /// Ce que porte le JSON d'une saisie de la page.
    const SAISIE: &[u8] = b"\"genre\":\"saisie\"";

    /// Lit les arguments, le nom de l'exécutable ôté. `--tenir-le-verrou`, en premier argument
    /// seulement, ouvre le mode du teneur : le fichier suit, et rien d'autre n'est admis. Sinon,
    /// `<dossier> <fichier>` viennent d'abord, puis les options, chacune une fois au plus. Chaque
    /// refus rend son message, que l'application dit avant de sortir en 64.
    pub fn analyser(args: &[OsString]) -> Result<Mode, String> {
        let mut reste = args.iter();
        let premier = reste.next();
        if premier.is_some_and(|a| a == "--tenir-le-verrou") {
            let fichier = valeur(&mut reste, "--tenir-le-verrou")?;
            if let Some(autre) = reste.next() {
                return Err(inconnue(autre));
            }
            return Ok(Mode::Teneur(PathBuf::from(fichier)));
        }
        let (Some(dossier), Some(evenements)) = (premier, reste.next()) else {
            return Err(USAGE.to_string());
        };
        if est_option(dossier) || est_option(evenements) {
            return Err(USAGE.to_string());
        }
        let mut options = Options {
            dossier: PathBuf::from(dossier),
            evenements: PathBuf::from(evenements),
            dialogue: false,
            reduite: false,
            focus: FOCUS_PAR_DEFAUT.to_string(),
            permettre: false,
            voler: None,
        };
        let mut focus_lu = false;
        while let Some(arg) = reste.next() {
            match arg.to_str() {
                Some(o @ "--dialogue") => drapeau(&mut options.dialogue, o)?,
                Some(o @ "--reduite") => drapeau(&mut options.reduite, o)?,
                Some(o @ "--permettre-premier-plan") => drapeau(&mut options.permettre, o)?,
                Some(o @ "--focus") => {
                    drapeau(&mut focus_lu, o)?;
                    let id = valeur(&mut reste, o)?;
                    options.focus = identifiant(id).ok_or_else(|| refusee(o, id))?;
                }
                Some(o @ "--voler-premier-plan") => {
                    if options.voler.is_some() {
                        return Err(repetee(o));
                    }
                    let cible = valeur(&mut reste, o)?;
                    let n = valeur(&mut reste, o)?;
                    let compte = n
                        .to_str()
                        .and_then(|t| t.parse::<usize>().ok())
                        .filter(|n| *n >= 1)
                        .ok_or_else(|| refusee(o, n))?;
                    options.voler = Some((PathBuf::from(cible), compte));
                }
                _ => return Err(inconnue(arg)),
            }
        }
        Ok(Mode::Fenetres(options))
    }

    /// Vrai si `arg` commence par `--` : une option, jamais un chemin ni une valeur.
    fn est_option(arg: &OsStr) -> bool {
        arg.as_encoded_bytes().starts_with(b"--")
    }

    /// La valeur qui suit `option` ; une option à sa place, ou rien, la laisse manquante.
    fn valeur<'a>(reste: &mut Iter<'a, OsString>, option: &str) -> Result<&'a OsStr, String> {
        match reste.next() {
            Some(v) if !est_option(v) => Ok(v),
            _ => Err(format!("valeur manquante : {option}")),
        }
    }

    /// Lève le drapeau d'`option`, qu'une seconde fois refuse.
    fn drapeau(leve: &mut bool, option: &str) -> Result<(), String> {
        if *leve {
            return Err(repetee(option));
        }
        *leve = true;
        Ok(())
    }

    /// L'identifiant d'un élément de la page : des lettres et des chiffres ASCII, `-` et `_`, rien
    /// qui changerait l'adresse où il passe (`?focus=<id>`).
    fn identifiant(id: &OsStr) -> Option<String> {
        let texte = id.to_str()?;
        let admis = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        (!texte.is_empty() && texte.chars().all(admis)).then(|| texte.to_string())
    }

    fn inconnue(arg: &OsStr) -> String {
        format!("option inconnue : {}", arg.to_string_lossy())
    }

    fn repetee(option: &str) -> String {
        format!("option répétée : {option}")
    }

    fn refusee(option: &str, valeur: &OsStr) -> String {
        format!("valeur refusée : {option} {}", valeur.to_string_lossy())
    }

    /// Une ligne du fichier d'événements : le titre de la fenêtre, une tabulation, le corps, puis
    /// le saut de ligne qui la termine.
    pub fn ligne(titre: &str, corps: &str) -> String {
        format!("{titre}\t{corps}\n")
    }

    /// Le corps de la ligne qui dit le résultat d'`AllowSetForegroundWindow`.
    pub fn permis(accorde: bool) -> String {
        let valeur = if accorde { "vrai" } else { "faux" };
        format!("{{\"genre\":\"permis\",\"valeur\":\"{valeur}\"}}")
    }

    /// Le corps de la ligne qui dit une touche reçue par une fenêtre elle-même, hors de sa page :
    /// `{"genre":"touche-fenetre","key":<touche>,"etat":"enfoncee"}`, ou `"relachee"`, la touche
    /// échappée comme `JSON.stringify` échappe les lignes de la page.
    pub fn touche_fenetre(touche: &str, enfoncee: bool) -> String {
        let etat = if enfoncee { "enfoncee" } else { "relachee" };
        format!(
            "{{\"genre\":\"touche-fenetre\",\"key\":{},\"etat\":\"{etat}\"}}",
            chaine_json(touche)
        )
    }

    /// `texte` en chaîne JSON, guillemets compris, comme l'écrit `JSON.stringify` : le guillemet et
    /// la barre oblique inverse précédés d'une barre, cinq caractères de contrôle par leur lettre
    /// (`\b`, `\f`, `\n`, `\r`, `\t`), tout autre caractère de contrôle C0 en `\u00xx`, en
    /// minuscules, et le reste tel quel.
    fn chaine_json(texte: &str) -> String {
        let mut sortie = String::from("\"");
        for c in texte.chars() {
            match c {
                '"' => sortie.push_str("\\\""),
                '\\' => sortie.push_str("\\\\"),
                '\u{8}' => sortie.push_str("\\b"),
                '\u{c}' => sortie.push_str("\\f"),
                '\n' => sortie.push_str("\\n"),
                '\r' => sortie.push_str("\\r"),
                '\t' => sortie.push_str("\\t"),
                c if u32::from(c) < 0x20 => sortie.push_str(&format!("\\u{:04x}", u32::from(c))),
                c => sortie.push(c),
            }
        }
        sortie.push('"');
        sortie
    }

    /// Un message de la page, reçu par l'IPC.
    #[derive(Debug, PartialEq)]
    pub enum Message {
        /// `ouvrir` : le bouton qui ouvre le dialogue.
        Ouvrir,
        /// `geler <s>` : le bouton qui fige l'hôte `s` secondes.
        Geler(u64),
        /// Un événement de la page, en JSON, à écrire ; `touche` s'il dit un `keydown`.
        Evenement { touche: bool },
        /// Un `geler` dont la durée ne se lit pas : ni ordre, ni ligne.
        Rien,
    }

    /// Le sens du message `corps`.
    pub fn classer(corps: &str) -> Message {
        if corps == "ouvrir" {
            return Message::Ouvrir;
        }
        if let Some(duree) = corps.strip_prefix("geler ") {
            return duree.parse().map_or(Message::Rien, Message::Geler);
        }
        Message::Evenement {
            touche: corps.contains(TOUCHE),
        }
    }

    /// Le compte des saisies d'un fichier d'événements que sa cible écrit encore, lu par morceaux :
    /// une ligne compte quand son saut de ligne est arrivé, et seulement si elle dit une saisie.
    #[derive(Default)]
    pub struct Compteur {
        /// Le début d'une ligne dont le saut de ligne n'est pas encore lu.
        reste: Vec<u8>,
        saisies: usize,
    }

    impl Compteur {
        /// Ajoute `octets`, lus à la suite des précédents, et rend le compte des saisies.
        pub fn lire(&mut self, octets: &[u8]) -> usize {
            self.reste.extend_from_slice(octets);
            while let Some(fin) = self.reste.iter().position(|o| *o == b'\n') {
                let ligne: Vec<u8> = self.reste.drain(..=fin).collect();
                if ligne.windows(SAISIE.len()).any(|f| f == SAISIE) {
                    self.saisies += 1;
                }
            }
            self.saisies
        }
    }

    /// Ces tests ne tournent qu'en CI, dans l'étape de la sentinelle du job `windows`, par
    /// `--include-ignored` : hors du domaine de `pilote` et de son contrôle statique, chaque test
    /// porte un `#[ignore = …]`, que `tests/statique.rs` exige. Un test de l'application, qui
    /// dépend de la crate `windows`, ne part ainsi jamais sur le poste par un simple `cargo test`.
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::path::Path;

        fn args(mots: &[&str]) -> Vec<OsString> {
            mots.iter().map(OsString::from).collect()
        }

        /// Les options des fenêtres que rend `analyser`, sinon une panique qui dit ce qu'il a rendu.
        fn fenetres(mots: &[&str]) -> Options {
            match analyser(&args(mots)) {
                Ok(Mode::Fenetres(options)) => options,
                autre => panic!("{mots:?} : {autre:?}"),
            }
        }

        fn refus(mots: &[&str]) -> String {
            match analyser(&args(mots)) {
                Err(message) => message,
                autre => panic!("{mots:?} : {autre:?}"),
            }
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn teneur_reconnu_en_premier_argument() {
            match analyser(&args(&["--tenir-le-verrou", "verrou.txt"])) {
                Ok(Mode::Teneur(fichier)) => assert_eq!(fichier, Path::new("verrou.txt")),
                autre => panic!("{autre:?}"),
            }
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn teneur_ne_lit_ni_dossier_ni_option() {
            assert_eq!(
                refus(&["--tenir-le-verrou"]),
                "valeur manquante : --tenir-le-verrou"
            );
            assert_eq!(
                refus(&["--tenir-le-verrou", "verrou.txt", "--dialogue"]),
                "option inconnue : --dialogue"
            );
            assert_eq!(
                refus(&["--tenir-le-verrou", "verrou.txt", "evenements.txt"]),
                "option inconnue : evenements.txt"
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn dossier_et_fichier_seuls_donnent_les_defauts() {
            let o = fenetres(&["profil", "evenements.txt"]);
            assert_eq!(
                (
                    o.dossier,
                    o.evenements,
                    o.dialogue,
                    o.reduite,
                    o.focus,
                    o.permettre,
                    o.voler
                ),
                (
                    PathBuf::from("profil"),
                    PathBuf::from("evenements.txt"),
                    false,
                    false,
                    "depart".to_string(),
                    false,
                    None
                )
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn chaque_option_se_lit() {
            let o = fenetres(&[
                "profil",
                "evenements.txt",
                "--dialogue",
                "--reduite",
                "--focus",
                "champ",
                "--permettre-premier-plan",
                "--voler-premier-plan",
                "cible.txt",
                "100",
            ]);
            assert_eq!(
                (
                    o.dossier,
                    o.evenements,
                    o.dialogue,
                    o.reduite,
                    o.focus,
                    o.permettre,
                    o.voler
                ),
                (
                    PathBuf::from("profil"),
                    PathBuf::from("evenements.txt"),
                    true,
                    true,
                    "champ".to_string(),
                    true,
                    Some((PathBuf::from("cible.txt"), 100))
                )
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn sans_dossier_ni_fichier_dit_l_usage() {
            assert_eq!(refus(&[]), USAGE);
            assert_eq!(refus(&["profil"]), USAGE);
            assert_eq!(refus(&["--dialogue", "profil", "evenements.txt"]), USAGE);
            assert_eq!(refus(&["profil", "--dialogue"]), USAGE);
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn une_option_inconnue_est_refusee() {
            assert_eq!(
                refus(&["profil", "evenements.txt", "--azerty"]),
                "option inconnue : --azerty"
            );
            assert_eq!(
                refus(&["profil", "evenements.txt", "--tenir-le-verrou", "v.txt"]),
                "option inconnue : --tenir-le-verrou"
            );
            assert_eq!(
                refus(&["profil", "evenements.txt", "en-trop"]),
                "option inconnue : en-trop"
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn une_valeur_manquante_est_refusee() {
            assert_eq!(
                refus(&["profil", "evenements.txt", "--focus"]),
                "valeur manquante : --focus"
            );
            assert_eq!(
                refus(&["profil", "evenements.txt", "--focus", "--reduite"]),
                "valeur manquante : --focus"
            );
            assert_eq!(
                refus(&[
                    "profil",
                    "evenements.txt",
                    "--voler-premier-plan",
                    "cible.txt"
                ]),
                "valeur manquante : --voler-premier-plan"
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn une_valeur_refusee_se_dit() {
            assert_eq!(
                refus(&["profil", "evenements.txt", "--focus", "champ&x=1"]),
                "valeur refusée : --focus champ&x=1"
            );
            assert_eq!(
                refus(&[
                    "profil",
                    "evenements.txt",
                    "--voler-premier-plan",
                    "c.txt",
                    "0"
                ]),
                "valeur refusée : --voler-premier-plan 0"
            );
            assert_eq!(
                refus(&[
                    "profil",
                    "evenements.txt",
                    "--voler-premier-plan",
                    "c.txt",
                    "dix"
                ]),
                "valeur refusée : --voler-premier-plan dix"
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn une_option_repetee_est_refusee() {
            for (mots, option) in [
                (&["--dialogue", "--dialogue"][..], "--dialogue"),
                (&["--reduite", "--reduite"][..], "--reduite"),
                (
                    &["--permettre-premier-plan", "--permettre-premier-plan"][..],
                    "--permettre-premier-plan",
                ),
                (&["--focus", "champ", "--focus", "zone"][..], "--focus"),
                (
                    &[
                        "--voler-premier-plan",
                        "a.txt",
                        "1",
                        "--voler-premier-plan",
                        "b.txt",
                        "2",
                    ][..],
                    "--voler-premier-plan",
                ),
            ] {
                let mut tout = vec!["profil", "evenements.txt"];
                tout.extend_from_slice(mots);
                assert_eq!(refus(&tout), format!("option répétée : {option}"));
            }
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn une_ligne_porte_le_titre_une_tabulation_et_le_corps() {
            assert_eq!(
                ligne("Appli d'essai", "{\"genre\":\"pret\"}"),
                "Appli d'essai\t{\"genre\":\"pret\"}\n"
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn permis_dit_vrai_ou_faux() {
            assert_eq!(permis(true), "{\"genre\":\"permis\",\"valeur\":\"vrai\"}");
            assert_eq!(permis(false), "{\"genre\":\"permis\",\"valeur\":\"faux\"}");
        }

        /// Une touche reçue par la fenêtre se dit comme la page dit les siennes : le genre, la
        /// touche échappée comme par `JSON.stringify`, puis son état.
        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn une_touche_de_la_fenetre_se_dit_en_json() {
            assert_eq!(
                touche_fenetre("a", true),
                "{\"genre\":\"touche-fenetre\",\"key\":\"a\",\"etat\":\"enfoncee\"}"
            );
            assert_eq!(
                touche_fenetre("Tab", false),
                "{\"genre\":\"touche-fenetre\",\"key\":\"Tab\",\"etat\":\"relachee\"}"
            );
            assert_eq!(
                touche_fenetre("\"\\\u{8}\u{c}\n\r\t\u{1}é", true),
                "{\"genre\":\"touche-fenetre\",\"key\":\"\\\"\\\\\\b\\f\\n\\r\\t\\u0001é\",\
                 \"etat\":\"enfoncee\"}"
            );
        }

        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn les_messages_de_la_page_se_trient() {
            assert!(matches!(classer("ouvrir"), Message::Ouvrir));
            assert!(matches!(classer("geler 8"), Message::Geler(8)));
            assert!(matches!(classer("geler huit"), Message::Rien));
            assert!(matches!(
                classer("{\"genre\":\"touche\",\"id\":\"champ\",\"key\":\"a\"}"),
                Message::Evenement { touche: true }
            ));
            assert!(matches!(
                classer("{\"genre\":\"saisie\",\"id\":\"champ\",\"valeur\":\"a\"}"),
                Message::Evenement { touche: false }
            ));
            assert!(matches!(
                classer("{\"genre\":\"pret\"}"),
                Message::Evenement { touche: false }
            ));
        }

        /// Le fichier de la cible se lit par morceaux, pendant qu'elle l'écrit : une ligne compte
        /// quand elle est complète, une fois, et seule une saisie compte.
        #[ignore = "hors du domaine de pilote : la sentinelle de la CI le lance"]
        #[test]
        fn le_compteur_ne_compte_que_les_saisies_des_lignes_completes() {
            let mut c = Compteur::default();
            assert_eq!(c.lire(b"Appli d'essai\t{\"genre\":\"saisie\""), 0);
            assert_eq!(
                c.lire(b",\"id\":\"champ\"}\nAppli d'essai\t{\"genre\":\"touche\"}\n"),
                1
            );
            assert_eq!(c.lire(b""), 1);
            assert_eq!(
                c.lire(b"Appli d'essai\t{\"genre\":\"saisie\"}\nAppli d'essai\t{\"genre\":\"saisie\"}\n"),
                3
            );
        }
    }
}
