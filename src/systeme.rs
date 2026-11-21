//! Le système : le seul module qui résout les chemins de données, ouvre le journal, lit l'horloge,
//! la date, la session et l'environnement (ADR-0003). Hors de lui, le contrôle statique,
//! `tests/statique.rs`, refuse les chemins devinés, l'horloge, l'environnement et le hasard.
