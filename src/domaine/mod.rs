//! Le domaine : les règles de `pilote`, pures et portables (ADR-0003). Il décide de tout contre le
//! trait `Bureau`, et toute lecture du poste passe par lui : il n'emploie ni l'horloge, ni
//! l'environnement, ni le système de fichiers, ni les processus, ni la crate `windows`. Il se
//! compile et se teste sous Linux comme sous Windows, sur le faux bureau, et `cargo-mutants` s'y
//! applique : zéro survivant, les mutants équivalents exclus avec leur raison. Le contrôle
//! statique, `tests/statique.rs`, le garde.
