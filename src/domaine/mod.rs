//! Le domaine : les règles de `pilote`, pures et portables (ADR-0003). Il décide de tout contre le
//! trait `Bureau`, et toute lecture du poste passe par lui : il n'emploie ni l'horloge, ni
//! l'environnement, ni le système de fichiers, ni les processus, ni la crate `windows`. Il se
//! compile et se teste sous Linux comme sous Windows, sur le faux bureau, et `cargo-mutants` s'y
//! applique : zéro survivant, les mutants équivalents exclus avec leur raison. Le contrôle
//! statique, `tests/statique.rs`, le garde. C'est une recherche dans le texte : il ferme les
//! formes courantes, les chemins complets (`std::fs`), les imports groupés (`use std::{…}`) et
//! les préfixes nus (`fs::`, `env::`, `process::`), d'où un import de `std` par chemin
//! (`use std::fmt;`). Il ne ferme ni les alias (`use std as s;`), ni les blancs dans un chemin
//! (`std :: fs`) : la relecture de chaque tâche du domaine les ferme.

pub mod cli;
pub mod sortie;
pub mod touches;
