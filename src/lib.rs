//! `pilote` : piloter une application de bureau de Windows, le clavier d'abord, sûr dès le premier
//! jour. Le domaine, pur et portable, décide ; sous Windows, la couche Windows lit et agit sans
//! décider, et `systeme` ouvre le journal, lit l'horloge, la date et l'environnement (ADR-0003).

pub mod domaine;

#[cfg(windows)]
mod bureau;
#[cfg(windows)]
mod systeme;

/// Lance l'outil et rend son code de sortie. Jusqu'à la Tâche 9, rien n'est livré : 64.
#[cfg(windows)]
pub fn lancer() -> u8 {
    64
}
