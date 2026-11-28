# Registre des constats

Une ligne par constat. Les numéros ne se réutilisent jamais.

- **AM** : une piste d'amélioration, « il faudrait ».
- **LE** : une leçon, « on a cru que ».
- **EM** : un empêchement, « on n'a pas pu ».

Au 3e constat portant le même motif, on écrit une porte (un test, un hook, une vérification de CI), et la ligne le dit.

| N° | Famille | Date | Constat | Motif | Prévention | Critère de sortie | État |
|---|---|---|---|---|---|---|---|
| LE-1 | LE | 2026-11-22 | Tranche 1 du lot 1 (branche `feat/tranche1`) : on a cru que `MapVirtualKeyExW(vk, MAPVK_VK_TO_VSC_EX)` marquait les touches étendues ; pour les flèches, Inser, Suppr, `Debut`, `Fin`, `PageHaut` et `PageBas`, il rend le balayage du pavé numérique sans 0xE0 (relevé du poste en lecture seule : VK_UP donne 0x0048 ; seuls Ctrl droite et Alt droite portent 0xE0). | le code de balayage étendu de Windows | 1er constat : la liste des touches étendues en dur dans le domaine (`touches::evenement` : 0x21 à 0x28, 0x2D et 0x2E), éprouvée par `fleches_et_inser_sont_etendues` | un essai de bout en bout vérifie le `code` qu'une flèche frappée par `touche` donne dans la page (`ArrowDown`, jamais `Numpad2`) | ouvert |
| LE-2 | LE | 2026-11-22 | Tranche 1 du lot 1 (branche `feat/tranche1`) : on a cru que `ToUnicodeEx` prenait le balayage que rend `MAPVK_VK_TO_VSC_EX`. Le préfixe 0xE0 d'une touche étendue y pose le bit 15, que `ToUnicodeEx` lit comme « touche relâchée », et rien n'est produit (relevé du poste en lecture seule, disposition française : `ToUnicodeEx(VK_DIVIDE, 0xE035)` rend 0, `ToUnicodeEx(VK_DIVIDE, 0x0035)` rend « / »). | le code de balayage étendu de Windows | 2e constat : seul l'octet bas du balayage va à `ToUnicodeEx` (`OCTET_BAS`, dans `src/bureau/clavier.rs`) ; la couche Windows n'a pas de test, et la revue de chaque tâche la relit | la règle `ToUnicodeEx` de `tests/statique.rs` : un appel hors de `src/bureau/clavier.rs`, ou sans `OCTET_BAS` sur sa ligne ni sur l'une des trois suivantes, fait échouer `sources_respectent_les_regles` | clos |
