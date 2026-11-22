//! L'analyse des arguments de `pilote` : les 21 sous-commandes du lot, leurs options et leurs
//! bornes (décision 11 du plan maître), par clap. L'analyse ne lit aucune fenêtre. L'aide et la
//! version arrivent en `Aide` de code `Fait` ; toute autre erreur de clap, en `Aide` de code
//! `Usage`, son message traduit en français, en une ligne, le texte tapé échappé (décision 6).

use crate::domaine::sortie::Code;
use crate::domaine::sortie::echapper;
use clap::Parser;
use clap::error::ContextKind;
use clap::error::ContextValue;
use clap::error::ErrorKind;
use std::ffi::OsString;
use std::path::PathBuf;

/// La version de `pilote` : celle du paquet, puis le commit construit entre parenthèses, 12
/// chiffres hexadécimaux ou « inconnu » (`build.rs`).
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("PILOTE_COMMIT"), ")");

/// La fenêtre visée : le nom de son processus, et ce qui départage plusieurs fenêtres.
#[derive(Clone, Debug, PartialEq)]
pub struct Cible {
    pub fenetre: String,
    pub titre: Option<String>,
    pub pid: Option<u32>,
}

/// Le sélecteur d'un élément de l'automatisation ; `genre` porte l'option `--type`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Selecteur {
    pub nom: Option<String>,
    pub contient: Option<String>,
    pub genre: Option<String>,
    pub id: Option<String>,
    pub classe: Option<String>,
    pub dans: Option<String>,
    pub rang: Option<u32>,
}

/// La source du texte à taper : `--texte`, `--fichier`, ou `-`, l'entrée standard.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    Texte(String),
    Fichier(PathBuf),
    Entree,
}

/// La forme que `taille` donne à la fenêtre.
#[derive(Clone, Debug, PartialEq)]
pub enum Forme {
    Dimensions { largeur: u32, hauteur: u32 },
    Agrandir,
    Restaurer,
    Equivalent(u16),
}

/// Une commande analysée.
#[derive(Clone, Debug, PartialEq)]
pub enum Commande {
    Fenetres,
    Activer(Cible),
    Touche {
        cible: Cible,
        touches: Vec<String>,
        fois: u32,
    },
    Taper {
        cible: Cible,
        source: Source,
        comme_au_clavier: bool,
    },
    Focus {
        cible: Cible,
        selecteur: Selecteur,
        attendre: Option<u32>,
    },
    Arbre {
        cible: Cible,
        max: u32,
        dans: Option<String>,
        attendre: Option<u32>,
    },
    Trouver {
        cible: Cible,
        selecteur: Selecteur,
        attendre: Option<u32>,
    },
    Focaliser {
        cible: Cible,
        selecteur: Selecteur,
    },
    Invoquer {
        cible: Cible,
        selecteur: Selecteur,
    },
    Choisir {
        cible: Cible,
        selecteur: Selecteur,
    },
    Valeur {
        cible: Cible,
        selecteur: Selecteur,
        egale: Option<String>,
        attendre: Option<u32>,
    },
    Defiler {
        cible: Cible,
        selecteur: Selecteur,
        a: u8,
    },
    Defilables(Cible),
    Capture {
        cible: Cible,
        sortie: PathBuf,
    },
    Molette {
        cible: Cible,
        x: i32,
        y: i32,
        crans: i32,
    },
    Clic {
        cible: Cible,
        x: i32,
        y: i32,
        double: bool,
        droit: bool,
    },
    Taille {
        cible: Cible,
        forme: Forme,
    },
    Lancer {
        exe: PathBuf,
        donnees_neuves: bool,
    },
    Fermer(Cible),
    Nettoyer,
    Annonces {
        cible: Cible,
        pendant: u32,
    },
}

impl Commande {
    /// Le nom de la sous-commande, tel que l'écrit la ligne de commande.
    pub fn nom(&self) -> &'static str {
        match self {
            Commande::Fenetres => "fenetres",
            Commande::Activer(_) => "activer",
            Commande::Touche { .. } => "touche",
            Commande::Taper { .. } => "taper",
            Commande::Focus { .. } => "focus",
            Commande::Arbre { .. } => "arbre",
            Commande::Trouver { .. } => "trouver",
            Commande::Focaliser { .. } => "focaliser",
            Commande::Invoquer { .. } => "invoquer",
            Commande::Choisir { .. } => "choisir",
            Commande::Valeur { .. } => "valeur",
            Commande::Defiler { .. } => "defiler",
            Commande::Defilables(_) => "defilables",
            Commande::Capture { .. } => "capture",
            Commande::Molette { .. } => "molette",
            Commande::Clic { .. } => "clic",
            Commande::Taille { .. } => "taille",
            Commande::Lancer { .. } => "lancer",
            Commande::Fermer(_) => "fermer",
            Commande::Nettoyer => "nettoyer",
            Commande::Annonces { .. } => "annonces",
        }
    }

    /// La fenêtre que vise la commande ; `fenetres`, `lancer` et `nettoyer` n'en visent aucune.
    pub fn cible(&self) -> Option<&Cible> {
        match self {
            Commande::Fenetres | Commande::Lancer { .. } | Commande::Nettoyer => None,
            Commande::Activer(cible) | Commande::Defilables(cible) | Commande::Fermer(cible) => {
                Some(cible)
            }
            Commande::Touche { cible, .. }
            | Commande::Taper { cible, .. }
            | Commande::Focus { cible, .. }
            | Commande::Arbre { cible, .. }
            | Commande::Trouver { cible, .. }
            | Commande::Focaliser { cible, .. }
            | Commande::Invoquer { cible, .. }
            | Commande::Choisir { cible, .. }
            | Commande::Valeur { cible, .. }
            | Commande::Defiler { cible, .. }
            | Commande::Capture { cible, .. }
            | Commande::Molette { cible, .. }
            | Commande::Clic { cible, .. }
            | Commande::Taille { cible, .. }
            | Commande::Annonces { cible, .. } => Some(cible),
        }
    }
}

/// Ce que l'analyse rend au lieu d'une commande : l'aide ou la version (`Fait`, sur la sortie
/// standard), ou une erreur d'usage (`Usage`, sur la sortie d'erreur).
#[derive(Clone, Debug, PartialEq)]
pub struct Aide {
    pub code: Code,
    pub texte: String,
}

/// La commande que disent `args`, le nom du programme en tête.
pub fn analyser(args: &[OsString]) -> Result<Commande, Aide> {
    let brut = Brut::try_parse_from(args).map_err(|e| traduire(&e))?;
    Ok(brut.commande.convertir())
}

/// L'aide que rend une erreur de clap : l'aide et la version, avec le texte de clap ; toute autre
/// erreur, traduite. `<arg>` est le nom que clap donne à l'argument, tel quel, sauf le texte tapé
/// (une option ou une sous-commande inconnues), qui s'échappe.
fn traduire(e: &clap::Error) -> Aide {
    let arg = || contexte(e, ContextKind::InvalidArg);
    let texte = match e.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            return Aide {
                code: Code::Fait,
                texte: e.render().to_string(),
            };
        }
        ErrorKind::UnknownArgument => format!("option inconnue : {}", echapper(&arg())),
        ErrorKind::MissingRequiredArgument => format!("option requise : {}", arg()),
        ErrorKind::InvalidValue if contexte(e, ContextKind::InvalidValue).is_empty() => {
            format!("valeur manquante : {}", arg())
        }
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => {
            format!("valeur refusée : {}", arg())
        }
        ErrorKind::ArgumentConflict => format!("options incompatibles : {}", arg()),
        ErrorKind::InvalidSubcommand => format!(
            "sous-commande inconnue : {}",
            echapper(&contexte(e, ContextKind::InvalidSubcommand))
        ),
        ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            "sous-commande manquante".to_string()
        }
        ErrorKind::TooManyValues | ErrorKind::WrongNumberOfValues => {
            format!("trop de valeurs : {}", arg())
        }
        _ => {
            let rendu = e.render().to_string();
            format!(
                "usage : {}",
                echapper(rendu.lines().next().unwrap_or_default())
            )
        }
    };
    Aide {
        code: Code::Usage,
        texte,
    }
}

/// Le contexte `genre` de l'erreur : une valeur seule, ou une liste dont les éléments se joignent
/// par une espace ; vide s'il manque.
fn contexte(e: &clap::Error, genre: ContextKind) -> String {
    match e.get(genre) {
        Some(ContextValue::String(valeur)) => valeur.clone(),
        Some(ContextValue::Strings(valeurs)) => valeurs.join(" "),
        _ => String::new(),
    }
}

/// Le nom de processus de `--fenetre` : sans caractère de contrôle, 260 caractères au plus.
fn fenetre_valide(texte: &str) -> Result<String, String> {
    if texte.chars().count() > 260 || texte.chars().any(char::is_control) {
        Err("un caractère de contrôle, ou plus de 260 caractères".to_string())
    } else {
        Ok(texte.to_string())
    }
}

/// Les crans de `--crans` : de -50 à 50, sauf 0.
fn crans_valides(texte: &str) -> Result<i32, String> {
    let crans: i32 = texte.parse().map_err(|_| "un nombre entier".to_string())?;
    if crans == 0 || !(-50..=50).contains(&crans) {
        Err("de -50 à 50, sauf 0".to_string())
    } else {
        Ok(crans)
    }
}

// La ligne de commande telle que clap la lit, avant sa conversion en `Commande`. Les types que clap
// dérive ici portent des commentaires simples : un commentaire de documentation y deviendrait
// l'aide de clap. Ceux de leurs champs et de leurs variantes sont cette aide.
#[derive(Parser)]
#[command(name = "pilote", version = VERSION, disable_help_subcommand = true)]
struct Brut {
    #[command(subcommand)]
    commande: Sous,
}

// La fenêtre visée, telle que clap la lit.
#[derive(clap::Args)]
struct ArgsCible {
    /// Le nom du processus de la fenêtre visée.
    #[arg(long, value_parser = fenetre_valide)]
    fenetre: String,
    /// Le titre qui départage plusieurs fenêtres du processus.
    #[arg(long, allow_hyphen_values = true)]
    titre: Option<String>,
    /// Le PID qui départage plusieurs fenêtres du processus.
    #[arg(long)]
    pid: Option<u32>,
}

impl ArgsCible {
    fn convertir(self) -> Cible {
        Cible {
            fenetre: self.fenetre,
            titre: self.titre,
            pid: self.pid,
        }
    }
}

// Le sélecteur d'un élément, tel que clap le lit.
#[derive(clap::Args)]
struct ArgsSelecteur {
    /// Le nom de l'élément.
    #[arg(long, allow_hyphen_values = true)]
    nom: Option<String>,
    /// Un texte que le nom de l'élément contient.
    #[arg(long, allow_hyphen_values = true)]
    contient: Option<String>,
    /// Le type de l'élément.
    #[arg(long = "type", value_name = "TYPE")]
    genre: Option<String>,
    /// L'identifiant de l'élément.
    #[arg(long, allow_hyphen_values = true)]
    id: Option<String>,
    /// La classe de l'élément.
    #[arg(long, allow_hyphen_values = true)]
    classe: Option<String>,
    /// Le nom d'un ancêtre de l'élément.
    #[arg(long, allow_hyphen_values = true)]
    dans: Option<String>,
    /// Le rang de l'élément parmi ceux qui répondent.
    #[arg(long)]
    rang: Option<u32>,
}

impl ArgsSelecteur {
    fn convertir(self) -> Selecteur {
        Selecteur {
            nom: self.nom,
            contient: self.contient,
            genre: self.genre,
            id: self.id,
            classe: self.classe,
            dans: self.dans,
            rang: self.rang,
        }
    }
}

// L'attente de `focus`, `arbre`, `trouver` et `valeur`, telle que clap la lit : sa borne
// (décision 11 du plan maître) s'écrit ici seulement.
#[derive(clap::Args)]
struct ArgsAttendre {
    /// L'attente, en millisecondes, 30 000 au plus.
    #[arg(long, value_parser = clap::value_parser!(u32).range(..=30_000))]
    attendre: Option<u32>,
}

impl ArgsAttendre {
    fn convertir(self) -> Option<u32> {
        self.attendre
    }
}

// Les sous-commandes, telles que clap les lit.
#[derive(clap::Subcommand)]
enum Sous {
    /// Liste les fenêtres.
    Fenetres,
    /// Active la fenêtre visée.
    Activer(ArgsCible),
    /// Frappe des touches ou des combinaisons dans la fenêtre visée.
    Touche {
        #[command(flatten)]
        cible: ArgsCible,
        /// Les touches ou les combinaisons, dans l'ordre.
        #[arg(required = true)]
        touches: Vec<String>,
        /// Le nombre de frappes, de 1 à 100.
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=100))]
        fois: u32,
    },
    /// Tape un texte dans la fenêtre visée.
    #[command(group(
        clap::ArgGroup::new("source").required(true).args(["texte", "fichier", "entree"])
    ))]
    Taper {
        #[command(flatten)]
        cible: ArgsCible,
        /// Le texte à taper.
        #[arg(long, allow_hyphen_values = true)]
        texte: Option<String>,
        /// Le fichier dont le texte se tape.
        #[arg(long)]
        fichier: Option<PathBuf>,
        /// `-` : le texte se lit sur l'entrée standard.
        #[arg(value_parser = ["-"])]
        entree: Option<String>,
        /// Chaque caractère se tape par la disposition du clavier.
        #[arg(long)]
        comme_au_clavier: bool,
    },
    /// Dit l'élément qui a le focus dans la fenêtre visée.
    Focus {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
        #[command(flatten)]
        attendre: ArgsAttendre,
    },
    /// Montre l'arbre des éléments de la fenêtre visée.
    Arbre {
        #[command(flatten)]
        cible: ArgsCible,
        /// Le nombre de lignes, de 1 à 10 000.
        #[arg(
            long,
            default_value_t = 400,
            value_parser = clap::value_parser!(u32).range(1..=10_000)
        )]
        max: u32,
        /// Le nom du groupe à montrer.
        #[arg(long, allow_hyphen_values = true)]
        dans: Option<String>,
        #[command(flatten)]
        attendre: ArgsAttendre,
    },
    /// Trouve les éléments que désigne le sélecteur.
    Trouver {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
        #[command(flatten)]
        attendre: ArgsAttendre,
    },
    /// Donne le focus à l'élément que désigne le sélecteur.
    Focaliser {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
    },
    /// Invoque l'élément que désigne le sélecteur.
    Invoquer {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
    },
    /// Coche ou choisit l'élément que désigne le sélecteur.
    Choisir {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
    },
    /// Lit la valeur de l'élément que désigne le sélecteur.
    Valeur {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
        /// La valeur attendue.
        #[arg(long, allow_hyphen_values = true)]
        egale: Option<String>,
        #[command(flatten)]
        attendre: ArgsAttendre,
    },
    /// Fait défiler l'élément que désigne le sélecteur.
    Defiler {
        #[command(flatten)]
        cible: ArgsCible,
        #[command(flatten)]
        selecteur: ArgsSelecteur,
        /// La position du défilement, en pour cent, de 0 à 100.
        #[arg(long, value_parser = clap::value_parser!(u8).range(..=100))]
        a: u8,
    },
    /// Liste les éléments défilables de la fenêtre visée.
    Defilables(ArgsCible),
    /// Capture la fenêtre visée.
    Capture {
        #[command(flatten)]
        cible: ArgsCible,
        /// Le fichier de la capture.
        #[arg(long)]
        sortie: PathBuf,
    },
    /// Tourne la molette dans la fenêtre visée.
    Molette {
        #[command(flatten)]
        cible: ArgsCible,
        /// L'abscisse, dans la fenêtre.
        #[arg(long)]
        x: i32,
        /// L'ordonnée, dans la fenêtre.
        #[arg(long)]
        y: i32,
        /// Les crans, de -50 à 50, sauf 0.
        #[arg(long, allow_negative_numbers = true, value_parser = crans_valides)]
        crans: i32,
    },
    /// Clique dans la fenêtre visée.
    Clic {
        #[command(flatten)]
        cible: ArgsCible,
        /// L'abscisse, dans la fenêtre.
        #[arg(long)]
        x: i32,
        /// L'ordonnée, dans la fenêtre.
        #[arg(long)]
        y: i32,
        /// Un double clic.
        #[arg(long)]
        double: bool,
        /// Le bouton droit.
        #[arg(long)]
        droit: bool,
    },
    /// Change la taille de la fenêtre visée.
    #[command(group(
        clap::ArgGroup::new("forme")
            .required(true)
            .args(["largeur", "agrandir", "restaurer", "equivalent"])
    ))]
    Taille {
        #[command(flatten)]
        cible: ArgsCible,
        /// La largeur, avec `--hauteur`.
        #[arg(long, requires = "hauteur")]
        largeur: Option<u32>,
        /// La hauteur, avec `--largeur`.
        #[arg(long)]
        hauteur: Option<u32>,
        /// Agrandit la fenêtre.
        #[arg(long)]
        agrandir: bool,
        /// Restaure la fenêtre.
        #[arg(long)]
        restaurer: bool,
        /// L'équivalent d'une mise à l'échelle de l'affichage, en pour cent, de 100 à 400.
        #[arg(long, value_parser = clap::value_parser!(u16).range(100..=400))]
        equivalent: Option<u16>,
    },
    /// Lance une application.
    Lancer {
        /// L'exécutable.
        exe: PathBuf,
        /// Avec un dossier de données neuf.
        #[arg(long)]
        donnees_neuves: bool,
    },
    /// Ferme la fenêtre visée.
    Fermer(ArgsCible),
    /// Efface les dossiers de données que `lancer` a créés.
    Nettoyer,
    /// Écoute les annonces de la fenêtre visée.
    Annonces {
        #[command(flatten)]
        cible: ArgsCible,
        /// La durée, en secondes, de 1 à 300.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..=300))]
        pendant: u32,
    },
}

impl Sous {
    /// La commande que dit la ligne lue.
    fn convertir(self) -> Commande {
        match self {
            Sous::Fenetres => Commande::Fenetres,
            Sous::Activer(cible) => Commande::Activer(cible.convertir()),
            Sous::Touche {
                cible,
                touches,
                fois,
            } => Commande::Touche {
                cible: cible.convertir(),
                touches,
                fois,
            },
            Sous::Taper {
                cible,
                texte,
                fichier,
                entree: _,
                comme_au_clavier,
            } => Commande::Taper {
                cible: cible.convertir(),
                source: match (texte, fichier) {
                    (Some(texte), _) => Source::Texte(texte),
                    (None, Some(fichier)) => Source::Fichier(fichier),
                    (None, None) => Source::Entree,
                },
                comme_au_clavier,
            },
            Sous::Focus {
                cible,
                selecteur,
                attendre,
            } => Commande::Focus {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
                attendre: attendre.convertir(),
            },
            Sous::Arbre {
                cible,
                max,
                dans,
                attendre,
            } => Commande::Arbre {
                cible: cible.convertir(),
                max,
                dans,
                attendre: attendre.convertir(),
            },
            Sous::Trouver {
                cible,
                selecteur,
                attendre,
            } => Commande::Trouver {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
                attendre: attendre.convertir(),
            },
            Sous::Focaliser { cible, selecteur } => Commande::Focaliser {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
            },
            Sous::Invoquer { cible, selecteur } => Commande::Invoquer {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
            },
            Sous::Choisir { cible, selecteur } => Commande::Choisir {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
            },
            Sous::Valeur {
                cible,
                selecteur,
                egale,
                attendre,
            } => Commande::Valeur {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
                egale,
                attendre: attendre.convertir(),
            },
            Sous::Defiler {
                cible,
                selecteur,
                a,
            } => Commande::Defiler {
                cible: cible.convertir(),
                selecteur: selecteur.convertir(),
                a,
            },
            Sous::Defilables(cible) => Commande::Defilables(cible.convertir()),
            Sous::Capture { cible, sortie } => Commande::Capture {
                cible: cible.convertir(),
                sortie,
            },
            Sous::Molette { cible, x, y, crans } => Commande::Molette {
                cible: cible.convertir(),
                x,
                y,
                crans,
            },
            Sous::Clic {
                cible,
                x,
                y,
                double,
                droit,
            } => Commande::Clic {
                cible: cible.convertir(),
                x,
                y,
                double,
                droit,
            },
            Sous::Taille {
                cible,
                largeur,
                hauteur,
                agrandir,
                restaurer: _,
                equivalent,
            } => Commande::Taille {
                cible: cible.convertir(),
                forme: match (largeur, hauteur, equivalent) {
                    (Some(largeur), Some(hauteur), _) => Forme::Dimensions { largeur, hauteur },
                    (_, _, Some(equivalent)) => Forme::Equivalent(equivalent),
                    _ if agrandir => Forme::Agrandir,
                    _ => Forme::Restaurer,
                },
            },
            Sous::Lancer {
                exe,
                donnees_neuves,
            } => Commande::Lancer {
                exe,
                donnees_neuves,
            },
            Sous::Fermer(cible) => Commande::Fermer(cible.convertir()),
            Sous::Nettoyer => Commande::Nettoyer,
            Sous::Annonces { cible, pendant } => Commande::Annonces {
                cible: cible.convertir(),
                pendant,
            },
        }
    }
}

/// Les constructeurs des tests, que `journal.rs` et `gestes.rs` réemploient.
#[cfg(test)]
pub(crate) mod essai {
    use super::Cible;
    use std::ffi::OsString;

    /// Les arguments de `s`, découpé sur les espaces, `pilote` en tête s'il n'y est pas déjà.
    pub fn a(s: &str) -> Vec<OsString> {
        let mots: Vec<&str> = s.split(' ').collect();
        let tete = if mots.first() == Some(&"pilote") {
            None
        } else {
            Some("pilote")
        };
        tete.into_iter().chain(mots).map(OsString::from).collect()
    }

    /// Une cible sans titre ni PID.
    pub fn cible(f: &str) -> Cible {
        Cible {
            fenetre: f.to_string(),
            titre: None,
            pid: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::essai::{a, cible};
    use super::*;

    #[test]
    fn usage_inconnu_sort_en_64() {
        let e = analyser(&a("pilote focus --fenetre x --bidule")).unwrap_err();
        assert_eq!(e.code, Code::Usage);
        assert!(
            e.texte.starts_with("option inconnue : --bidule"),
            "{}",
            e.texte
        );
    }
    #[test]
    fn aide_et_version_sortent_en_0() {
        assert_eq!(analyser(&a("pilote --help")).unwrap_err().code, Code::Fait);
        let v = analyser(&a("pilote --version")).unwrap_err();
        assert_eq!(v.code, Code::Fait);
        assert_eq!(v.texte.trim_end(), format!("pilote {VERSION}"));
    }
    #[test]
    fn version_porte_le_commit_ou_inconnu() {
        let (num, commit) = VERSION.split_once(" (").unwrap();
        assert_eq!(num, env!("CARGO_PKG_VERSION"));
        let c = commit.strip_suffix(')').unwrap();
        assert!(
            c == "inconnu" || (c.len() == 12 && c.chars().all(|x| x.is_ascii_hexdigit())),
            "{c}"
        );
    }
    #[test]
    fn sans_sous_commande_sort_en_64() {
        let e = analyser(&a("pilote")).unwrap_err();
        assert_eq!(e.code, Code::Usage);
        assert_eq!(e.texte, "sous-commande manquante");
    }
    #[test]
    fn sous_commande_inconnue() {
        assert!(
            analyser(&a("pilote help"))
                .unwrap_err()
                .texte
                .starts_with("sous-commande inconnue : help")
        );
    }
    #[test]
    fn toutes_les_sous_commandes_s_analysent() {
        for s in [
            "fenetres",
            "activer --fenetre x",
            "touche --fenetre x Tab",
            "taper --fenetre x --texte t",
            "taper --fenetre x -",
            "taper --fenetre x --fichier f",
            "focus --fenetre x",
            "arbre --fenetre x",
            "trouver --fenetre x --nom n",
            "focaliser --fenetre x --nom n",
            "invoquer --fenetre x --nom n",
            "choisir --fenetre x --nom n",
            "valeur --fenetre x --nom n",
            "defiler --fenetre x --nom n --a 50",
            "defilables --fenetre x",
            "capture --fenetre x --sortie c.png",
            "molette --fenetre x --x 1 --y 2 --crans 3",
            "clic --fenetre x --x 1 --y 2",
            "taille --fenetre x --agrandir",
            "lancer e.exe",
            "fermer --fenetre x",
            "nettoyer",
            "annonces --fenetre x --pendant 5",
        ] {
            let c = analyser(&a(&format!("pilote {s}")))
                .unwrap_or_else(|e| panic!("{s} : {}", e.texte));
            assert_eq!(c.nom(), s.split(' ').next().unwrap());
        }
    }
    #[test]
    fn bornes_des_options() {
        for (s, ok) in [
            ("touche --fenetre x Tab --fois 100", true),
            ("touche --fenetre x Tab --fois 101", false),
            ("touche --fenetre x Tab --fois 0", false),
            ("focus --fenetre x --attendre 30000", true),
            ("focus --fenetre x --attendre 30001", false),
            ("arbre --fenetre x --attendre 30000", true),
            ("arbre --fenetre x --attendre 30001", false),
            ("trouver --fenetre x --nom n --attendre 30000", true),
            ("trouver --fenetre x --nom n --attendre 30001", false),
            ("valeur --fenetre x --nom n --attendre 30000", true),
            ("valeur --fenetre x --nom n --attendre 30001", false),
            ("arbre --fenetre x --max 10000", true),
            ("arbre --fenetre x --max 10001", false),
            ("arbre --fenetre x --max 0", false),
            ("molette --fenetre x --x 0 --y 0 --crans 50", true),
            ("molette --fenetre x --x 0 --y 0 --crans 51", false),
            ("molette --fenetre x --x 0 --y 0 --crans -50", true),
            ("molette --fenetre x --x 0 --y 0 --crans -51", false),
            ("molette --fenetre x --x 0 --y 0 --crans 0", false),
            ("annonces --fenetre x --pendant 1", true),
            ("annonces --fenetre x --pendant 0", false),
            ("annonces --fenetre x --pendant 300", true),
            ("annonces --fenetre x --pendant 301", false),
            ("taille --fenetre x --equivalent 100", true),
            ("taille --fenetre x --equivalent 99", false),
            ("taille --fenetre x --equivalent 400", true),
            ("taille --fenetre x --equivalent 401", false),
            ("defiler --fenetre x --nom n --a 101", false),
        ] {
            assert_eq!(analyser(&a(&format!("pilote {s}"))).is_ok(), ok, "{s}");
        }
    }
    #[test]
    fn max_vaut_400_par_defaut() {
        assert!(matches!(
            analyser(&a("pilote arbre --fenetre x")).unwrap(),
            Commande::Arbre { max: 400, .. }
        ));
    }
    #[test]
    fn fois_vaut_1_par_defaut() {
        assert!(matches!(
            analyser(&a("pilote touche --fenetre x Tab")).unwrap(),
            Commande::Touche { fois: 1, .. }
        ));
    }
    #[test]
    fn molette_accepte_des_crans_negatifs() {
        assert!(matches!(
            analyser(&a("pilote molette --fenetre x --x 1 --y 2 --crans -3")).unwrap(),
            Commande::Molette { crans: -3, .. }
        ));
    }
    #[test]
    fn taper_accepte_un_texte_qui_commence_par_un_tiret() {
        assert_eq!(
            analyser(&a("pilote taper --fenetre x --texte -abc")).unwrap(),
            Commande::Taper {
                cible: cible("x"),
                source: Source::Texte("-abc".into()),
                comme_au_clavier: false
            }
        );
    }
    #[test]
    fn taper_tiret_lit_l_entree() {
        assert!(matches!(
            analyser(&a("pilote taper --fenetre x -")).unwrap(),
            Commande::Taper {
                source: Source::Entree,
                ..
            }
        ));
    }
    #[test]
    fn une_seule_source_de_texte() {
        assert!(
            analyser(&a("pilote taper --fenetre x --texte t -"))
                .unwrap_err()
                .texte
                .starts_with("options incompatibles : ")
        );
        assert!(analyser(&a("pilote taper --fenetre x")).is_err());
    }
    #[test]
    fn taille_exige_une_seule_forme() {
        assert!(analyser(&a("pilote taille --fenetre x --agrandir --restaurer")).is_err());
        assert!(analyser(&a("pilote taille --fenetre x")).is_err());
    }
    #[test]
    fn fenetre_refuse_un_controle_et_plus_de_260_caracteres() {
        let args: Vec<OsString> = ["pilote", "activer", "--fenetre", "a\tb"]
            .map(Into::into)
            .into();
        assert_eq!(analyser(&args).unwrap_err().code, Code::Usage);
        assert!(analyser(&a(&format!("pilote activer --fenetre {}", "x".repeat(260)))).is_ok());
        assert!(analyser(&a(&format!("pilote activer --fenetre {}", "x".repeat(261)))).is_err());
    }
    #[test]
    fn erreurs_de_clap_en_francais() {
        assert!(
            analyser(&a("pilote activer"))
                .unwrap_err()
                .texte
                .starts_with("option requise : --fenetre")
        );
    }
    #[test]
    fn valeur_manquante_se_dit() {
        assert!(
            analyser(&a("pilote activer --fenetre"))
                .unwrap_err()
                .texte
                .starts_with("valeur manquante : --fenetre")
        );
    }

    // Le texte tapé s'échappe dans chaque message, et chaque bras de la traduction des erreurs de
    // clap a son test.
    #[test]
    fn argument_inconnu_echappe() {
        let args: Vec<OsString> = ["pilote", "activer", "--fenetre", "x", "a\nb"]
            .map(Into::into)
            .into();
        assert_eq!(
            analyser(&args).unwrap_err().texte,
            r"option inconnue : a\nb"
        );
        let args: Vec<OsString> = ["pilote", "a\nb"].map(Into::into).into();
        assert!(!analyser(&args).unwrap_err().texte.contains('\n'));
    }
    #[test]
    fn valeur_refusee_se_dit() {
        assert!(
            analyser(&a("pilote touche --fenetre x Tab --fois 101"))
                .unwrap_err()
                .texte
                .starts_with("valeur refusée : --fois")
        );
    }
    #[test]
    fn trop_de_valeurs_se_dit() {
        assert!(
            analyser(&a("pilote taille --fenetre x --agrandir=oui"))
                .unwrap_err()
                .texte
                .starts_with("trop de valeurs : --agrandir")
        );
    }
    #[test]
    fn taper_n_admet_que_le_tiret_en_positionnel() {
        assert!(
            analyser(&a("pilote taper --fenetre x foo"))
                .unwrap_err()
                .texte
                .starts_with("valeur refusée : ")
        );
    }
    #[test]
    fn cible_de_la_commande() {
        assert_eq!(
            analyser(&a("pilote activer --fenetre x")).unwrap().cible(),
            Some(&cible("x"))
        );
        for s in ["fenetres", "lancer e.exe", "nettoyer"] {
            assert_eq!(
                analyser(&a(&format!("pilote {s}"))).unwrap().cible(),
                None,
                "{s}"
            );
        }
    }
    // Le repli « usage : » : un argument qui n'est pas de l'UTF-8 y tombe. Sous Windows, un tel
    // argument ne se forme que par `std::os::windows::ffi::OsStringExt`, que le contrôle statique
    // refuse dans le domaine (« windows:: ») : l'erreur InvalidUtf8 de clap se forme ici sans
    // lui, et l'analyse de bout en bout se vérifie sous Linux, en CI.
    #[test]
    fn usage_pour_les_autres_erreurs() {
        assert_eq!(
            traduire(&clap::Error::new(clap::error::ErrorKind::InvalidUtf8)),
            Aide {
                code: Code::Usage,
                texte: "usage : error: invalid UTF-8 was detected in one or more arguments".into()
            }
        );
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let args: Vec<OsString> = vec![
                "pilote".into(),
                "activer".into(),
                "--fenetre".into(),
                OsString::from_vec(vec![0xFF]),
            ];
            let e = analyser(&args).unwrap_err();
            assert_eq!(e.code, Code::Usage);
            assert!(e.texte.starts_with("usage : "), "{}", e.texte);
        }
    }

    // Chaque option du sélecteur et chaque forme de `taille` se lisent dans la commande analysée,
    // la garde d'`agrandir` comprise, puis la source `--fichier` et l'option
    // `--comme-au-clavier` : une conversion qui perdrait une valeur ferait échouer ces tests.
    #[test]
    fn selecteur_porte_ses_options() {
        assert_eq!(
            analyser(&a(concat!(
                "pilote trouver --fenetre x --nom n --contient c --type t",
                " --id i --classe k --dans d --rang 2"
            )))
            .unwrap(),
            Commande::Trouver {
                cible: cible("x"),
                selecteur: Selecteur {
                    nom: Some("n".into()),
                    contient: Some("c".into()),
                    genre: Some("t".into()),
                    id: Some("i".into()),
                    classe: Some("k".into()),
                    dans: Some("d".into()),
                    rang: Some(2),
                },
                attendre: None,
            }
        );
    }
    #[test]
    fn taille_rend_sa_forme() {
        for (s, forme) in [
            (
                "--largeur 800 --hauteur 600",
                Forme::Dimensions {
                    largeur: 800,
                    hauteur: 600,
                },
            ),
            ("--agrandir", Forme::Agrandir),
            ("--restaurer", Forme::Restaurer),
            ("--equivalent 200", Forme::Equivalent(200)),
        ] {
            assert_eq!(
                analyser(&a(&format!("pilote taille --fenetre x {s}"))).unwrap(),
                Commande::Taille {
                    cible: cible("x"),
                    forme
                },
                "{s}"
            );
        }
    }
    #[test]
    fn taper_un_fichier_comme_au_clavier() {
        assert_eq!(
            analyser(&a(
                "pilote taper --fenetre x --fichier f.txt --comme-au-clavier"
            ))
            .unwrap(),
            Commande::Taper {
                cible: cible("x"),
                source: Source::Fichier(PathBuf::from("f.txt")),
                comme_au_clavier: true,
            }
        );
    }

    // Chaque valeur que la conversion porte se lit, pas seulement le nom de la sous-commande ni le
    // succès de l'analyse : `--titre` et `--pid` départagent les fenêtres et dirigent la frappe, et
    // les touches et `--fois` disent ce qui se frappe. Un titre qui commence par un tiret éprouve
    // `allow_hyphen_values`.
    #[test]
    fn touche_porte_sa_cible_ses_touches_et_ses_fois() {
        assert_eq!(
            analyser(&a(
                "pilote touche --fenetre x --titre -T --pid 42 Ctrl+A Tab --fois 3"
            ))
            .unwrap(),
            Commande::Touche {
                cible: Cible {
                    fenetre: "x".into(),
                    titre: Some("-T".into()),
                    pid: Some(42),
                },
                touches: vec!["Ctrl+A".into(), "Tab".into()],
                fois: 3,
            }
        );
    }
    #[test]
    fn activer_et_taper_portent_le_titre_et_le_pid() {
        let attendue = Cible {
            fenetre: "x".into(),
            titre: Some("-T".into()),
            pid: Some(42),
        };
        assert_eq!(
            analyser(&a("pilote activer --fenetre x --titre -T --pid 42")).unwrap(),
            Commande::Activer(attendue.clone())
        );
        assert_eq!(
            analyser(&a("pilote taper --pid 42 --fenetre x --texte t --titre -T")).unwrap(),
            Commande::Taper {
                cible: attendue,
                source: Source::Texte("t".into()),
                comme_au_clavier: false,
            }
        );
    }
    // `--attendre` porte sa valeur, ou rien, dans chacune des quatre sous-commandes qui l'ont.
    #[test]
    fn attendre_porte_sa_valeur() {
        for s in [
            "focus --fenetre x",
            "arbre --fenetre x",
            "trouver --fenetre x --nom n",
            "valeur --fenetre x --nom n",
        ] {
            let attendre = |ligne: String| match analyser(&a(&ligne)).unwrap() {
                Commande::Focus { attendre, .. }
                | Commande::Arbre { attendre, .. }
                | Commande::Trouver { attendre, .. }
                | Commande::Valeur { attendre, .. } => attendre,
                autre => panic!("{ligne} : {autre:?}"),
            };
            assert_eq!(
                attendre(format!("pilote {s} --attendre 1234")),
                Some(1234),
                "{s}"
            );
            assert_eq!(attendre(format!("pilote {s}")), None, "{s}");
        }
    }
}
