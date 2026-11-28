# Tranche 2 · la lecture, les gestes et la capture — plan d'implémentation

> **Pour les agents :** compétence requise : `superpowers:subagent-driven-development` (recommandée) ou
> `superpowers:executing-plans`, tâche par tâche. Les étapes se cochent (`- [ ]`). Chaque tâche suit le test
> d'abord : le test écrit, lancé et vu rouge, puis le code, puis le test vert, puis le commit. Sur le poste, ne
> lancer ni `pilote` ni `appli-essai` (hors `--version` et `--help`), jamais `cargo run`, jamais `cargo test` avec
> `--ignored` ou `--include-ignored` : les essais de bout en bout pilotent le bureau, et la CI seule les lance.
> `cargo build`, `cargo clippy`, `cargo fmt`, `cargo test --workspace`, `cargo doc`, `cargo mutants` sur le domaine
> et la lecture des sources des crates du registre (`~/.cargo/registry/src/`) sont permis.

Plan maître : `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md`

**But :** lire ce que lit un lecteur d'écran et agir par l'accessibilité, au clavier comme au pointeur, sans
qu'une lecture sorte de la fenêtre retenue ni qu'une valeur secrète sorte ; capturer la seule fenêtre retenue ;
rétablir la règle du champ de KeePassXC ; le prouver en CI sur WebView2, jeton restreint compris.

**Architecture :** celle de la tranche 1 (ADR-0003). Le domaine, `src/domaine/`, décide de tout contre le trait
`Bureau`, étendu à l'automatisation, à la capture et au pointeur : il reçoit l'arbre d'une fenêtre en une liste
plate, choisit l'élément, juge les gardes, et demande le geste par l'indice de l'élément. La couche Windows lit
l'arbre d'un seul tenant par le cache de l'automatisation, garde les éléments de la dernière lecture, et agit sur
eux sans rien décider.

**Pile :** celle de la tranche 1, plus `png` 0.18.1 et `unicode-normalization` 0.1.26 (portables, dans
`[dependencies]`), et les features `Win32_Graphics_Gdi`, `Win32_Storage_Xps` et `Win32_UI_Accessibility` de
`windows` 0.62.2. Aucune autre dépendance.

**Spec :** le plan maître (« Tranche 2 », décisions 3 à 7, 11 à 16 et 20, « Outils et cas limites » et « Points
de vigilance » pour la tranche 2) ; ADR-0005 et ADR-0006 ; les décisions d'Edouard du 28/11/2026 (`Ctrl+F6`,
`Ctrl+Tab`, `Ctrl+Alt+<lettre>`) ; le « Reste ouvert » de la PR de la tranche 1 (#5).

---

## Les décisions de ce plan

Le plan maître et les ADR décident ; ce plan précise ce qu'ils laissaient à la tranche 2. Chaque point où il
précise ou change le plan maître ou un ADR est listé plus bas, dans sa section à lui, pour qu'Edouard l'accepte
avec le plan.

1. **Les ADR s'amendent en place.** ADR-0005 et ADR-0006 reçoivent les règles de la tranche 2 selon
   `docs/modele/documentation.md` (v0.15.0) : une ligne `- **Amendé** le AAAA-MM-JJ : tranche 2 du lot 1 — <ce qui
   change>` dans leur en-tête, une par jour d'amendement (une tâche qui amende un jour déjà noté ajoute son point à
   la ligne de ce jour) ; leur corps dit ce qui est en vigueur, à la place de l'ancien texte ; une section
   `## Confirmation`, entre « Conséquences » et « Révision », qui nomme les tests de chaque règle par leur chemin,
   et que chaque tâche complète ; une sous-section `### Ce que la tranche 2 précise ou change`, après celle de la
   tranche 1, qui liste chaque précision, pour que la PR la redéclare. Chaque comportement qui change un ADR
   l'amende dans la tâche qui le livre, jamais plus tard. La direction ne change pas : aucun nouvel ADR.
2. **COM en MTA, l'AUMID sur un fil à part.** `lancer` initialise COM en `COINIT_MULTITHREADED` (le plan maître,
   « Tranche 2 ») : le client de l'automatisation y est sans fil d'interface, et une application figée ne bloque pas
   `pilote` sur une pompe de messages. L'AUMID se lit sur un fil neuf, initialisé en `COINIT_APARTMENTTHREADED`,
   puis `CoUninitialize` et `join` : sa lecture ne dépend pas du modèle du fil principal, que la documentation de
   `SHGetPropertyStoreForWindow` ne dit pas, et c'est celle des essais (`Parametres::regle`, dans un processus en
   STA), si bien que l'essai des Paramètres dit ce que lit `pilote`. Un fil qui ne naît pas, ou une lecture ratée,
   rend `None`, comme aujourd'hui : le domaine refuse le cadre sans enfant lisible. Limite, qu'ADR-0005 dit : faute
   d'une autre application du Store sur le runner, aucun essai ne prouve qu'un AUMID d'une autre application se lit ;
   un AUMID perdu ne refuserait à tort que le cadre réduit d'une telle application.
3. **Les modules.** Le domaine gagne `element` (le modèle d'un élément, ses types, sa ligne, le classement des
   erreurs de l'automatisation), `selecteur`, `lectures` (`focus`, `arbre`, `trouver`, `valeur`, `defilables`),
   `capture`, `automatisation` (`focaliser`, `invoquer`, `choisir`, `defiler`) et `pointeur` (`clic`, `molette`) ;
   `gestes` garde `executer`, l'ordre des gardes du clavier, la chaîne commune des gestes et la garde d'une unité
   (décision 13). La couche gagne
   `src/bureau/uia.rs` et `src/bureau/capture.rs`. C'est une précision de la décision 4 du plan maître, qui ne
   nommait que `selecteur`.
4. **L'arbre se lit d'un seul tenant, sans valeur.** `lire_l_arbre(W)` rend tous les éléments de la vue de contrôle
   sous W, W en tête (indice 0), en ordre de document, chacun avec l'indice de son parent ; la couche le lit par une
   seule requête de cache (`TreeScope_Subtree`, le filtre par défaut, celui de la vue de contrôle) et garde ses
   éléments. Le cache ne porte jamais la valeur d'un élément, et le motif LegacyIAccessible, qui la porte aussi,
   n'est jamais nommé (`UIA_ValueValuePropertyId`, `UIA_LegacyIAccessibleValuePropertyId`,
   `UIA_LegacyIAccessiblePatternId` et `IUIAutomationLegacyIAccessiblePattern`, refusés dans `src/` par le contrôle
   statique) : le domaine lit chaque valeur à part (`lire_la_valeur(i)`), jamais celle d'un élément marqué
   `IsPassword`. Une lecture ratée de `IsPassword`, en cache ou courante, ou du processus du focus, fait échouer la
   lecture entière (`ErreurUia`) : jamais un « pas un mot de passe » par défaut. Les gestes nomment l'élément par son
   indice dans la dernière lecture. Une absence (un motif non offert, aucun enfant, aucun focus) arrive en `None` ou en
   liste vide, jamais en erreur ; une erreur arrive en `ErreurUia { code, texte }`, que le domaine classe :
   `UIA_E_ELEMENTNOTAVAILABLE` (0x80040201), `RPC_E_DISCONNECTED` (0x80010108) et `CO_E_OBJNOTCONNECTED`
   (0x800401FD) en 2, « l'élément a disparu » ; `UIA_E_TIMEOUT` (0x80131505) en 3, « échec du système : délai de
   l'automatisation dépassé » ; `UIA_E_ELEMENTNOTENABLED` (0x80040200) en 3, « échec du système : l'élément est
   inactif » ; `UIA_E_NOTSUPPORTED` (0x80040204) et `UIA_E_INVALIDOPERATION` (0x80131509) en 3, « échec du
   système : l'élément refuse le geste » ; tout autre code en 3, « échec du système : <texte> ». La couche écrit
   toujours le code dans le texte, « <appel> : 0x<code> », suivi de « : <message> » si Windows en donne un non vide :
   `FormatMessage` n'a aucun texte pour `UIA_E_ELEMENTNOTENABLED`, et prête à `UIA_E_NOTSUPPORTED` celui d'un autre
   code de même numéro. Pour un `HRESULT(0)` inattendu, elle écrit « <appel> : rien rendu », jamais le message de
   Windows, qui dirait « l'opération a réussi ».
5. **L'ordre des lectures** (`focus`, `arbre`, `trouver`, `valeur`, `defilables`, `capture`) : l'analyse (64) ; le
   journal (1) ; la fenêtre retenue (1, 2, 2) ; pour `capture`, KeePassXC (1), la fenêtre réduite (1), puis la
   sensibilité de `pilote` au DPI de chaque écran (3, « échec du système : pilote n'est pas sensible au DPI ») ; la
   réponse de W, `SendMessageTimeoutW(W, WM_NULL, SMTO_ABORTIFHUNG)` en 2 000 ms au plus (3, « échec du système : la
   fenêtre ne répond pas ») ; pour l'automatisation, son ouverture (`CUIAutomation8`, connexion 2 000 ms,
   transaction 5 000 ms, `AutoSetFocus` coupé, reprise de connexion coupée) ; puis la lecture. Ni verrou, ni poste
   verrouillé, ni niveau d'intégrité, ni touche enfoncée, ni activation. La réponse de W rend l'hôte figé
   déterministe : sans elle, `ElementFromHandle(W)` pourrait rendre un arbre partiel lu par le proxy des fenêtres.
   `focus` juge le premier plan accepté avant toute lecture de l'automatisation (1, « focus hors de la fenêtre
   visée »), lit l'arbre de la fenêtre au premier plan accepté (W, ou un dialogue que W possède), puis le focus
   (`GetFocusedElement`) : il faut que son processus soit de l'arbre retenu et qu'il soit un élément de cet arbre lu
   (`CompareElements`), sinon 1, sans nom ni valeur.
6. **Le document vide et l'attente.** Un arbre est vide si son `Document` le plus extérieur, celui de la page (le
   premier, en ordre de document, qui n'a aucun ancêtre `Document`), n'a aucun enfant, ou si aucun de ses éléments
   n'est d'un type hors de `Window`, `Pane` et `TitleBar`, le sous-arbre d'une `TitleBar` ne comptant pas : un
   `Document` vide sous celui de la page (un `iframe` vide, un `role="document"`) ne rend pas l'arbre vide. Une seule
   échéance borne la commande : `max(2 000 ms, --attendre)`, comptée depuis sa première lecture de l'arbre. Une seule
   fonction du domaine (`lectures::lire_l_arbre`) relit l'arbre toutes les 100 ms tant qu'il est vide et que
   l'échéance n'est pas passée, puis rend ce qu'elle a lu, vide ou non. `--attendre` rejoue ensuite, toutes les
   100 ms, tant que `--attendre` ne s'est pas écoulé depuis la même première lecture, les seuls jugements qui
   peuvent changer : aucun élément, aucun élément au rang, l'élément focalisé qui ne répond pas, le focus hors de la
   fenêtre visée, la valeur qui diffère, le groupe de `arbre --dans` absent ; chaque reprise relit l'arbre par la
   même fonction, bornée à la même échéance, si bien que la commande ne dure jamais plus que l'échéance, au temps
   d'une lecture près. L'ambiguïté, le mot de passe, le motif absent et l'élément disparu arrêtent tout de suite. À
   la borne, l'arrêt est le dernier jugement.
7. **Le sélecteur.** Les candidats sont tous les éléments de l'arbre sauf W, en ordre de document ; les options se
   cumulent. `--nom`, `--contient`, `--dans` et `--egale` comparent après NFC, U+00A0, U+202F et U+2007 comptés
   pour des espaces, des deux côtés, sensibles à la casse ; `--id` et `--classe` en égalité exacte ; `--type`
   prend un nom de la table des 41 types de l'automatisation (`Button` … `AppBar`, 50000 à 50040), sans égard à la
   casse ASCII, et un nom inconnu sort en 64 à l'analyse (« type inconnu : <type> »). `--dans` ne compte ni W ni un
   ancêtre de type `Window` ou `Document`. `--rang` va de 1 à `u32::MAX` (clap : 0 rend « valeur refusée : --rang
   <RANG> ») ; un rang au-delà des candidats sort en 2. `focaliser`, `invoquer`, `choisir`, `defiler` et `valeur`
   exigent au moins une option parmi `--nom`, `--contient`, `--type`, `--id`, `--classe` et `--dans` (64, « sélecteur
   vide : … ») ; `trouver` sans option rend tous les candidats ; `focus` filtre l'élément focalisé, et refuse
   `--rang` (64, « options incompatibles : --rang <RANG> »).
8. **La ligne d'un élément** : `[<Type>] <nom>`, puis, dans cet ordre, `id`, `classe`, `valeur` ou `motdepasse=oui`,
   `raccourci`, `acces`, `annonce`, `etat`, `dans`, `rang`. Un nom vide n'a pas de chevrons ; un `id`, une `classe`,
   un `raccourci` ou un `acces` vides n'ont pas leur clé ; `valeur` ne paraît que pour un élément qui offre le motif
   Value, `valeur=""` s'il est vide ; `annonce` vaut `polite` (1) ou `assertive` (2), absente sinon ; `etat` suit
   l'ordre d'ADR-0006 (`coche`, `decoche` ou `indetermine` pour Toggle 1, 0 ou 2 ; `choisi` ; `inactif` ; `deplie`
   pour ExpandCollapse 1 ou 2, `replie` pour 0 ; `lecture-seule`). `focus`, `trouver`, `valeur` et `defilables`
   portent `dans` ; `trouver`, `rang` ; `arbre`, ni l'un ni l'autre : il commence par la ligne de W et indente de
   deux espaces par niveau ; `--max` compte ses lignes, et un arbre coupé sort en 0 avec « arbre coupé : <max>
   lignes sur <n> » sur la sortie d'erreur. `arbre --dans <nom>` montre les sous-arbres les plus extérieurs dont la
   racine porte ce nom (hors W, `Window` et `Document`), chacun indenté depuis sa racine ; aucun : 2.
9. **Les gestes de l'automatisation** (`focaliser`, `invoquer`, `choisir`, `defiler`) prennent les positions 1 à 10
   des gestes du clavier (le verrou, le poste, le niveau, les touches enfoncées, l'activation, le refus rejugé),
   puis la réponse de W, l'ouverture, l'arbre de W (sondé s'il est vide), l'élément choisi (2), le motif lu dans
   l'arbre (3), puis, avant chaque appel qui agit (`SetFocus`, `Invoke`, `Toggle`, `Select`, `SetScrollPercent`),
   la garde d'une unité (décision 13), dont le refus est celui d'ADR-0006, « le premier plan a changé : <k> unités
   sur <n> sont parties » : une unité y est un appel qui agit, `k` compte ceux déjà partis, `n` ceux que le geste a
   prévus à ce moment (1, et 2 pour la seconde bascule de `choisir`), si bien qu'une session sait qu'une première
   bascule a déjà changé l'état. Les sondes d'état de `choisir` ne demandent pas le premier plan : elles ne font que
   lire. Le suivi compte un geste dès avant l'appel qui agit. Un `UIA_E_TIMEOUT` rendu par l'appel qui agit, et non
   par une lecture, sort en 3, « échec du système : délai de l'automatisation dépassé pendant le geste ; il a pu
   agir : relire avant de rejouer » : un `Invoke` qui ouvre un dialogue modal peut ne rendre la main qu'à sa
   fermeture, et le délai de transaction le coupe après que le geste a agi. `choisir` : Toggle d'abord s'il est
   offert (rien si coché ; une bascule ; l'état sondé toutes les 100 ms, 2 s au plus, jusqu'à ce qu'il change, sinon
   3 ; coché, 0 ; sinon une seconde bascule, la même sonde, puis 3 s'il n'est pas coché), sinon SelectionItem (rien
   si choisi ; `Select` ; la même sonde), sinon 3. `defiler --a <n>` : `SetScrollPercent(UIA_ScrollPatternNoScroll,
   n)`, et 3 pour un élément qui ne défile pas verticalement. Aucun geste n'écrit sur la sortie standard.
10. **La capture.** `PrintWindow` en rendu complet, dans une section DIB de 32 bits en BGRA de haut en bas ;
    l'image se rogne au cadre de DWM (`DWMWA_EXTENDED_FRAME_BOUNDS`, en pixels physiques comme le reste, le
    processus étant PMv2 : la décision 5 le vérifie avant de capturer), ou reste entière s'il ne se lit pas ; une
    image vide sort en 3, une image dont chaque composante RGB vaut 0 aussi ; le PNG est RGB, 8 bits, sans alpha,
    `Compression::Fast` ; il ne s'écrit qu'après ces jugements, à la place d'un fichier existant ; un fichier qui ne s'écrit pas sort en 64 (« fichier non
    écrit : <chemin> »). La ligne : `[Capture] <chemin> largeur=<l> hauteur=<h> zone-cliente=<x>,<y>,<l>,<h>`, la
    zone cliente relative à l'image rognée, sans la clé si elle ne se lit pas.
11. **Le pointeur.** `--x` et `--y` sont des pixels physiques depuis le coin haut gauche de la zone cliente ; clap
    admet un nombre négatif, que le domaine refuse ensuite (1) ; `--crans` positif tourne vers le haut. Après la
    position 11 des gestes du clavier (les touches enfoncées relues) : la sensibilité de `pilote` au DPI de chaque
    écran (3, « échec du système : pilote n'est pas sensible au DPI ») — sans elle, la zone cliente et la fenêtre
    sous le point se liraient en pixels virtualisés, quand `SendInput` vise des pixels physiques, et le clic
    tomberait ailleurs. Puis, pour chaque unité, la garde d'une unité (décision 13), dont les lectures d'entre-deux
    sont, dans cet ordre : la zone cliente, relue (3, « échec du système : zone cliente illisible »), et le bureau
    virtuel ; le point d'écran calculé ; le point dans la zone, `[0, l) × [0, h)` (1, « point hors de la zone
    cliente : <x>,<y> pour <l>×<h> »), dans le bureau virtuel (1, « point hors de l'écran : <x>,<y> »), la fenêtre de
    premier niveau sous lui acceptée comme le premier plan, sur l'instantané de la première lecture du premier plan
    (1, « point recouvert par une autre fenêtre ») ; le niveau du processus de la fenêtre au premier plan et de
    celui de la fenêtre sous le point. Puis le premier plan relu, égal ; puis ce même point normalisé, le lot de
    l'unité bâti sur lui, l'envoi, la cadence : une fenêtre qui bouge entre deux crans emmène le point du cran
    suivant, et la garde du point recouvert juge toujours le point qui reçoit l'événement. Une unité est un clic (le
    déplacement absolu, le bouton enfoncé, relâché), un double clic (le déplacement, deux fois enfoncé et relâché,
    d'un seul envoi) ou un cran (le déplacement, la molette de ±120, positif vers le haut). Le point se normalise au
    quart de pixel, à l'échelle de 65 536 sur l'étendue : `n = min(65535, (4 × (p − origine) + 1) × 65536 / (4 ×
    étendue))`, sur chaque axe du bureau virtuel, sûr à cette échelle sous l'arrondi par défaut comme au plus proche.
    La documentation de `MOUSEINPUT` ne dit que « de 0 à 65 535, (65535, 65535) au coin bas droit » : à l'échelle de
    65 535 sur `étendue − 1`, ce point tomberait un pixel avant au-delà des trois quarts de l'étendue. L'essai
    `clic_au_bord_du_bureau_virtuel` (Tâche 14) départage : s'il voit le pointeur un pixel avant le point visé, la
    formule passe à `n = ((p − origine) × 65535 + étendue − 2) / (étendue − 1)`, avec les valeurs attendues de
    `normalisation_vise_le_quart_du_pixel` et de `clic_envoie_un_seul_lot`, et ADR-0005 dit l'échelle mesurée. Des
    boutons inversés par le réglage de la souris s'échangent dans le domaine. Le pointeur n'est pas remis en place.
12. **KeePassXC, la règle du champ.** Quand P, ou le processus d'une fenêtre enfant de W, est KeePassXC (à la
    retenue ou au rejugement) : `taper` refuse à la position 3, puis au rejugement, un texte qui porte une tabulation
    ou un saut de ligne ; après la position 12, la réponse de W et l'ouverture de l'automatisation
    (`lectures::ouvrir`) ; puis, dans la garde de chaque unité de `touche` et de `taper` (décision 13), les lectures
    d'entre-deux sont l'arbre de la fenêtre au premier plan accepté, sondé s'il est vide (`lectures::lire_l_arbre`,
    2 s), lu à la première unité et chaque fois que cette fenêtre change, puis le focus (`GetFocusedElement`) :
    illisible (une erreur), absent, ou hors de cet arbre lu (`indice` nul), refus (« KeePassXC : focus illisible ») ;
    dans un champ de mot de passe, `taper` refuse, et `touche` n'admet que Tab, Maj+Tab, Haut, Bas, Gauche, Droite,
    Debut, Fin, PageHaut, PageBas et Echap, nus. Le focus se juge par son appartenance à l'arbre lu de la fenêtre au
    premier plan, et non par son processus : un élément de WebView2 peut porter le PID de `msedgewebview2`, un
    descendant de P, et l'arbre de Chromium, qui ne se bâtit qu'à la première requête, rendrait sinon le document ou
    la fenêtre hôte pour focus, sans `IsPassword`. Un focus sur un élément né après la lecture de l'arbre refuse aussi,
    le sens sûr. La cadence y vaut 100 ms au lieu de 5, pour que le focus déplacé par une tabulation se lise avant
    l'unité suivante ; c'est une limite, que la règle borne sans la fermer. `activer` vers KeePassXC reste permis.
13. **La garde d'une unité.** Juste avant chaque envoi (une combinaison, un caractère, un clic, un cran) et chaque
    appel qui agit de l'automatisation, dans cet ordre : (a) le premier plan accepté (`premier_plan_accepte` : sa
    lecture, l'instantané des processus, sa relecture) ; (b) les lectures d'entre-deux, chacune seulement si elle a
    lieu d'être : le niveau d'intégrité de chaque processus autre que P qui tient la fenêtre au premier plan accepté
    (un dialogue que W possède) ou, pour le pointeur, la fenêtre sous le point, comparé à celui de `pilote` comme
    celui de P (1, « fenêtre plus élevée que pilote », ou « niveau d'intégrité illisible »), chaque PID une fois par
    commande ; pour le pointeur, le point (décision 11) ; pour KeePassXC, l'arbre et le focus (décision 12) ; (c) si
    (b) a lu quoi que ce soit, le premier plan accepté relu, qui doit égaler celui de (a), chaîne des propriétaires
    comprise (sinon 1, « le premier plan a changé : <k> unités sur <n> sont parties ») ; (d) l'envoi, ou l'appel,
    juste après la dernière lecture du premier plan. Rien ne s'intercale donc entre le dernier contrôle et l'envoi,
    comme ADR-0005 le veut (« Le premier plan accepté »), et une unité qui n'a rien à lire en (b) garde la seule
    lecture de la tranche 1. Au faux bureau, chaque `Envoi` et chaque `Agir` suit exactement `[PremierPlan, Processus,
    PremierPlan]` : `premier_plan_verifie_avant_chaque_unite` le garde pour `taper`, et ses jumeaux pour KeePassXC, un
    dialogue d'un autre processus, `clic`, `molette` et chaque geste de l'automatisation (Tâches 8 à 10).
14. **Ctrl+F6** (Edouard, 28/11/2026) rejoint les touches interdites : `(alt || ctrl) && f6`, quels que soient les
    autres modificateurs et l'ordre, jugé avant toute touche inconnue comme les autres interdites. `Ctrl+Tab` reste
    permis (Edouard, 28/11/2026 : les onglets des boîtes de dialogue) : ADR-0005 en dit la limite, il change de
    classeur dans Excel, et la garde du premier plan n'arrête que l'unité suivante ; sa phrase « une touche dont on
    apprend qu'elle change de fenêtre rejoint la liste » nomme cette exception. `Ctrl+Alt+<lettre>` reste admis
    (Edouard, 28/11/2026).
15. **Le contrôle statique** gagne quatre règles, chacune avec sa plantation : `CreateProcess` et `ShellExecute`
    rejoignent les mots de lancement de `tests/` ; `CreateRestrictedToken`, `CreateProcessAsUser`,
    `SetTokenInformation`, `UIA_ValueValuePropertyId`, `UIA_LegacyIAccessibleValuePropertyId`,
    `UIA_LegacyIAccessiblePatternId` et `IUIAutomationLegacyIAccessiblePattern` sont refusés dans `src/` (le code de
    la décision 14 du plan maître vit dans les essais ; la valeur ne passe ni par le cache ni par le motif
    LegacyIAccessible, qui la porte aussi) ; un `#[test]` qui n'est pas en tête de ligne est une faute, hors de
    `tests/statique.rs` ; `ToUnicodeEx(` hors de `src/bureau/clavier.rs`, ou sans
    `OCTET_BAS` dans les trois lignes qui le suivent, est une faute, ce qui donne à LE-2 un critère de sortie qui se
    ferme.
16. **L'essai de la décision 14 du plan maître.** `tests/commun/mod.rs` lance `pilote` par `CreateProcessAsUserW`
    avec un jeton tiré de celui des essais (le niveau haut du runner) : restreint (`CreateRestrictedToken`, le groupe
    Administrateurs en refus seul, `DISABLE_MAX_PRIVILEGE`, puis le niveau moyen par `SetTokenInformation`), ou
    seulement abaissé (`DuplicateTokenEx`, puis le niveau moyen). Le profil jetable se marque au niveau moyen
    (`icacls <racine> /setintegritylevel (OI)(CI)M /T /Q`), le bloc d'environnement est entier, trié, en UTF-16
    (`CREATE_UNICODE_ENVIRONMENT`), le bureau `winsta0\default`, et les sorties vont à des fichiers hérités du
    profil. `SE_GROUP_INTEGRITY` (0x20) s'y écrit en constante locale, sa source en commentaire : la constante de
    `windows` vit sous `Win32_System_SystemServices`, une feature que le dépôt ne prend pas. D'abord, `fenetres` au
    jeton restreint doit sortir en 0 et lister `processus=appli-essai` : il prouve le lancement, le bureau, le
    journal et l'énumération, sans verrou ni niveau, et sépare ces causes du refus jugé ensuite. Puis les deux essais
    attendent 1, « fenêtre plus élevée que pilote » : ADR-0005 dit, d'après la sonde du poste, que le jeton filtré
    lit le jeton d'une cible élevée du même compte, et « illisible » n'arrive que pour un autre compte (le plan
    maître attendait « illisible »). Ce que chacun prouve : le jeton restreint, la configuration du poste (le groupe
    Administrateurs en refus seul, le niveau moyen) ; le jeton abaissé, la comparaison des deux niveaux lus quand le
    groupe Administrateurs, actif, ouvre sûrement le jeton de la cible. Tout autre résultat — « niveau d'intégrité
    illisible », « journal inaccessible », « poste verrouillé », le verrou, un autre code, un lancement refusé —
    arrête l'exécutant : l'essai écrit le message reçu et le journal du profil restreint sur sa sortie d'erreur,
    l'exécutant le dit à Edouard, et aucune garde ne change pour que l'essai passe. Le repli sur le seul test du
    domaine demande l'accord daté d'Edouard, et la PR le déclare en écart.
17. **Les délais de la CI.** La Tâche 13 pose des délais provisoires, marqués « provisoire » en commentaire
    (`windows` 20 min, `mutants` 40 min), pour que les jobs finissent ; la Tâche 17 les remplace par deux fois le plus
    long passage à froid mesuré (le critère de sortie du plan maître). Un délai provisoire n'est pas une mesure : il
    peut descendre à la mesure ; un délai mesuré ne baisse jamais. Le délai de `mutants` (19 min) est déjà sous deux
    fois son passage du 22/11 (11 min 12 s) : la mesure le relève. Celui de `mutants-complet.yml` reste à 25 min
    (deux fois 12 min 17 s, le passage à froid du 22/11/2026 ; Edouard, 28/11/2026), posé par la PR de ce plan : la
    tranche ne le touche pas, et sa PR dit comment le remesurer après la fusion.
18. **Ce qui reste ouvert de la tranche 1** (PR #5) : Verr. Maj lue sur un fil neuf (Tâche 11), son état illisible
    rendu en `Option` au domaine, qui refuse (Tâche 5) ; le niveau d'une fenêtre possédée par W d'un autre processus
    (Tâche 8, décision 13) ; l'essai des Paramètres réduits qui dit quelle règle refuse (Tâche 15) ; `#[test]` hors
    de la tête de ligne et les lancements Win32 vus par le contrôle statique (Tâche 1) ; les options des sous-commandes livrées essayées avec elles, une valeur à tiret pour chaque
    option de texte du sélecteur et de `arbre`, les bornes de `--max` (1, 10 000) et de `--a` (0, 100) (Tâche 4) ;
    les renvois « (décision N) » du code remplacés par des renvois aux sections des ADR, ou de `CLAUDE.md` quand
    aucun ADR ne porte la règle (Tâche 1) ; LE-2 et son critère qui se ferme (Tâche 1). Ce qui reste : la
    disposition illisible rendue en `Option` par le trait, le domaine décidant des modificateurs de `ToUnicodeEx`,
    va au plan de la tranche 3, puisque cette tranche ne touche ni `texte.rs` ni la disposition de `clavier.rs` ;
    `--hauteur` qui exige `--largeur` va à la tranche 3, avec `taille`.

**Les messages de la tranche**, mot pour mot ; `<…>` échappé comme ADR-0006 le dit :

| Code | Message | Sous-commandes |
|---|---|---|
| 64 | `type inconnu : <type>` | celles du sélecteur |
| 64 | `sélecteur vide : --nom, --contient, --type, --id, --classe ou --dans` | `focaliser`, `invoquer`, `choisir`, `defiler`, `valeur` |
| 64 | `options incompatibles : --rang <RANG>` | `focus` |
| 64 | `fichier non écrit : <chemin>` | `capture` |
| 1 | `focus hors de la fenêtre visée` | `focus` |
| 1 | `valeur non rendue : champ de mot de passe` | `valeur` |
| 1 | `capture refusée : fenêtre de KeePassXC` | `capture` |
| 1 | `capture refusée : fenêtre réduite` | `capture` |
| 1 | `le premier plan a changé : <k> unités sur <n> sont parties` (celui d'ADR-0006, une unité étant ici un appel qui agit, un clic ou un cran) | les gestes de l'automatisation, `clic`, `molette` |
| 1 | `Verr. Maj illisible` | `taper --comme-au-clavier` |
| 1 | `point hors de la zone cliente : <x>,<y> pour <l>×<h>` | `clic`, `molette` |
| 1 | `point hors de l'écran : <x>,<y>` | `clic`, `molette` |
| 1 | `point recouvert par une autre fenêtre` | `clic`, `molette` |
| 1 | `KeePassXC : un texte qui porte une tabulation ou un saut de ligne est refusé` | `taper` |
| 1 | `KeePassXC : focus dans un champ de mot de passe ; <k> unités sur <n> sont parties` | `taper` |
| 1 | `KeePassXC : seules les touches de déplacement passent dans un champ de mot de passe ; <k> unités sur <n> sont parties` | `touche` |
| 1 | `KeePassXC : focus illisible ; <k> unités sur <n> sont parties` | `touche`, `taper` |
| 2 | `aucun élément ne répond au sélecteur` | celles du sélecteur |
| 2 | `plusieurs éléments répondent au sélecteur : <n> ; --rang départage` | id. |
| 2 | `aucun élément au rang <r> : <n> candidats` | id. |
| 2 | `l'élément focalisé ne répond pas au sélecteur` | `focus` |
| 2 | `aucun élément « <nom> » pour --dans` | `arbre` |
| 2 | `la valeur diffère de « <texte> »` | `valeur --egale` |
| 2 | `l'élément a disparu` | toutes celles de l'automatisation |
| 3 | `échec du système : la fenêtre ne répond pas` | lectures, capture, gestes de l'automatisation |
| 3 | `échec du système : délai de l'automatisation dépassé` | toutes celles de l'automatisation |
| 3 | `échec du système : délai de l'automatisation dépassé pendant le geste ; il a pu agir : relire avant de rejouer` | les gestes de l'automatisation |
| 3 | `échec du système : l'élément est inactif` ; `échec du système : l'élément refuse le geste` | toutes celles de l'automatisation |
| 3 | `échec du système : pilote n'est pas sensible au DPI` | `capture`, `clic`, `molette` |
| 3 | `échec du système : l'élément n'offre pas le motif Invoke` (de même `Value`, `Scroll`) | `invoquer`, `valeur`, `defiler` |
| 3 | `échec du système : l'élément n'offre ni Toggle ni SelectionItem` | `choisir` |
| 3 | `échec du système : l'état n'a pas changé en 2 s` | `choisir` |
| 3 | `échec du système : l'élément n'est pas coché après deux bascules` | `choisir` |
| 3 | `échec du système : l'élément ne défile pas verticalement` | `defiler` |
| 3 | `échec du système : capture vide` ; `échec du système : capture toute noire` | `capture` |
| 3 | `échec du système : zone cliente illisible` | `clic`, `molette` |
| 0 | `arbre coupé : <max> lignes sur <n>` (sortie d'erreur) | `arbre` |

Le message de KeePassXC de la tranche 1 (« KeePassXC : touche et taper sont refusés dans cette version ») disparaît.
Le refus d'un mot de passe dit « valeur non rendue », pour ne pas se confondre avec « valeur refusée : <arg> », le
64 de `clap`.

## Ce que ce plan précise ou change du plan maître et des ADR

Chaque ligne se relit contre le texte qu'elle touche ; les précisions entrent dans les ADR par la tâche qui les
livre (décision 1), et la PR les redéclare.

- **Plan maître, ligne 45, et ADR-0006, « Conséquences »** (« un changement passe par un nouvel ADR ») : les ADR
  s'amendent en place, par la règle de `docs/modele/documentation.md` v0.15.0, sans nouvel ADR (décision 1). La
  compétence, qui naît à la tranche 4, n'a rien à suivre dans cette PR.
- **Plan maître, décision 4** : six modules du domaine de plus que `selecteur`, et deux fichiers de la couche
  (décision 3).
- **Plan maître, décision 14** : le message attendu au jeton restreint est « fenêtre plus élevée que pilote », non
  « niveau d'intégrité illisible », d'après ADR-0005 ; `fenetres` au jeton restreint passe d'abord ; toute autre
  réponse arrête l'exécutant, sans qu'aucune garde change (décision 16).
- **Plan maître, décision 16** : un délai provisoire, marqué tel, peut descendre à la mesure ; seul un délai mesuré
  ne baisse jamais (décision 17).
- **Plan maître, décision 3** (« le premier plan vérifié avant … chaque appel de l'automatisation ») : avant chaque
  appel qui agit, pas avant les sondes d'état de `choisir` ni avant les lectures (décision 9).
- **Plan maître, décision 6** (« quand P est KeePassXC ») : la règle vaut aussi quand le processus d'une fenêtre
  enfant de W est KeePassXC, comme ADR-0005 le dit déjà ; le focus illisible, ou hors de l'arbre lu de la fenêtre
  au premier plan, refuse ; la cadence y vaut 100 ms (décision 12). Le refus d'un geste de l'automatisation dit
  combien d'appels qui agissent sont partis, dans la forme d'ADR-0006 (décision 9).
- **Plan maître, décisions 11 et 12, et ADR-0006, « L'attente »** (« au plus ce délai ») : la sonde du document vide
  et les reprises de `--attendre` partagent une seule échéance, `max(2 s, --attendre)`, comptée depuis la première
  lecture ; le document vide est celui de la page, le plus extérieur (décision 6).
- **ADR-0005, « Le premier plan accepté »** (« L'envoi qui suit part juste après elle ») : ce qui doit se lire avant
  une unité (le niveau d'un processus pas encore jugé, le point, l'arbre et le focus de KeePassXC) se lit entre deux
  lectures du premier plan accepté, qui doivent être égales, et l'envoi ou l'appel suit la dernière (décision 13).
- **ADR-0005, « KeePassXC »** : le refus entier de la tranche 1 cède à la règle du champ, le focus jugé sur l'arbre
  lu de la fenêtre au premier plan (décision 12) ; la conséquence « D'ici la tranche 2 du lot 1, KeePassXC ne
  reçoit ni `touche` ni `taper` » se retire.
- **ADR-0005, « Les touches interdites »** et sa précision de la tranche 1 (« F6, Ctrl+F6 et Maj+F6 restent
  permis ») : Ctrl+F6 interdite ; la limite de `Ctrl+Tab`, et l'exception qu'il fait à « une touche dont on apprend
  qu'elle change de fenêtre rejoint la liste » ; `Ctrl+Alt+<lettre>` inchangé (décision 14).
- **ADR-0005, « Ce qui précise ou change le plan de la tranche »** (`CoInitializeEx(COINIT_APARTMENTTHREADED)`) :
  COM en MTA, l'AUMID lu sur un fil neuf en STA, et la limite d'un AUMID d'une autre application que rien n'éprouve
  en CI (décision 2).
- **ADR-0005, « Ce qu'on lit reste dans la fenêtre retenue »** : l'ordre des lectures, la réponse de W en 2 000 ms,
  les délais de l'automatisation, et la règle de `focus` par `GetFocusedElement` et `CompareElements`
  (décision 5) ; une lecture ratée de `IsPassword` fait échouer la lecture (décision 4).
- **ADR-0005, « Les gestes et l'ordre des gardes »** : l'ordre des gestes de l'automatisation (décision 9) ; celui
  de `clic` et `molette`, avec la sensibilité au DPI, le point relu à chaque unité, le point hors de l'écran, la
  normalisation et son échelle, et les boutons inversés (décision 11) ; le niveau d'un dialogue d'un autre
  processus, ou de la fenêtre sous le point (décision 13) ; Verr. Maj illisible refusée (décision 18).
- **ADR-0005, « Le journal », « Après un geste »** : un appel de l'automatisation qui agit compte comme un geste
  dès avant de partir (décision 9).
- **ADR-0005, « Les captures »** : la sensibilité au DPI exigée, le rognage au cadre de DWM, l'image noire ou vide,
  l'écriture après les jugements, à la place d'un fichier existant (décisions 5 et 10).
- **ADR-0006, « La sortie »** : l'ordre des clés d'une ligne d'élément, la ligne de W en tête de `arbre`, son
  indentation, `dans` et `rang` selon la sous-commande, `defilables` (décision 8) ; la ligne `[Capture]`, et les clés
  `largeur`, `hauteur` et `zone-cliente` dans la liste des clés (décision 10).
- **ADR-0006, « Le sélecteur »** : les candidats sans W, `--type` sans égard à la casse et le type inconnu en 64,
  le sélecteur vide en 64, `--rang` borné par clap, `focus --rang` refusé, `--egale` normalisé (décision 7).
- **ADR-0006, « L'attente »** : la définition du document vide, l'échéance unique `max(2 s, --attendre)`, les
  jugements qui se rejouent et ceux qui arrêtent (décision 6).
- **ADR-0006, « Les gestes de l'automatisation »** : l'algorithme détaillé de `choisir`, `defiler` vertical, le
  délai dépassé pendant le geste (décision 9).
- **ADR-0006, « Les codes de sortie »**, ligne par ligne : au 1, la capture d'une fenêtre de KeePassXC, le premier
  plan changé avant un appel de l'automatisation, le focus illisible de KeePassXC, Verr. Maj illisible, et les trois
  refus du point à la place de « point de clic ou de molette hors de la fenêtre retenue » ; au 2, la valeur qui
  diffère, avec ou sans `--attendre`, l'élément disparu, aucun élément pour `arbre --dans` ; au 3, la fenêtre qui ne
  répond pas, la capture vide, `pilote` qui n'est pas sensible au DPI, la zone cliente illisible, l'état inchangé
  de `choisir`, l'élément inactif ou qui refuse le geste, le délai dépassé pendant le geste ; au 64, le type
  inconnu, le sélecteur vide, les options incompatibles, le fichier non écrit (décisions 4 à 12).
- **ADR-0006, « Les messages »** : les messages de la table ci-dessus, `valeur non rendue : champ de mot de passe`
  compris ; le message de KeePassXC de la tranche 1 se retire.
- **ADR-0006, « Les bornes »** : `--x` et `--y` admettent un nombre négatif à l'analyse, et sont des pixels
  physiques depuis le coin haut gauche de la zone cliente ; `--crans` positif tourne vers le haut (décision 11).
- **ADR-0006, « L'analyse »** : « pas encore livrée » ne vaut plus que pour `taille`, `lancer`, `fermer`,
  `nettoyer` et `annonces`.

## Contraintes générales

Celles du plan maître et de la tranche 1 valent toutes. En plus :

- Règles du poste, sans exception : ne jamais lancer `pilote` ni `appli-essai` (hors `--version` et `--help`),
  jamais `cargo run`, jamais `cargo test -- --ignored` ni `--include-ignored`. Le dépôt est public : aucun chemin
  d'utilisateur, aucun nom de famille ; `%USERPROFILE%`, `%LOCALAPPDATA%`, `~` à leur place.
- Le domaine ne nomme aucune interface de l'automatisation : il reçoit des `ElementLu` et des codes, et rend des
  indices et des `GesteUia`. Chaque durée (100 ms, 2 s, 2 000 ms, 5 000 ms, 100 ms de KeePassXC) est une constante
  du domaine.
- La couche ne choisit ni un élément, ni un motif, ni un code de sortie : chaque méthode lit ou agit, et rend
  `None`, une liste vide ou une erreur. Elle n'a aucun test ; les essais de bout en bout l'éprouvent.
- Aucun `SetValue`, aucune valeur dans le cache, aucune valeur d'un élément `IsPassword` lue par le domaine.
- Chaque refus d'avant l'activation se vérifie sans `Restaurer`, `Demande`, `Envoi` ni `Agir` au faux bureau ;
  chaque lecture sans `PrendreVerrou`, `EtatSession`, `Niveau`, `TouchesEnfoncees`, `Restaurer`, `Demande`, `Envoi`
  ni `Agir`.
- Chaque `Envoi` et chaque `Agir` suit exactement `[PremierPlan, Processus, PremierPlan]` au faux bureau (décision
  13) : une tâche qui ajoute un geste, ou une lecture avant une unité, ajoute le jumeau de
  `premier_plan_verifie_avant_chaque_unite` qui le vérifie.
- Le faux bureau panique sur une lecture de l'automatisation qui précède son ouverture : l'ordre se teste ainsi
  dans chaque test.
- Les tests du domaine des modules nouveaux emploient les aides de `gestes::essai` (Tâche 5) ; un test qui reprend
  le nom d'un test de `gestes.rs` le dit autrement, pour qu'aucun nom ne double.
- Une étape « voir rouge » qui nomme deux filtres les met après `--` (`cargo test --lib -- domaine::a domaine::b`) :
  avant lui, `cargo test` n'en prend qu'un, et refuse le second sans rien compiler ni lancer.
- À la fin des Tâches 2 à 10, `cargo mutants -p pilote -F '^src/domaine/<module>\.rs:'` (cargo-mutants 27.1.0 est
  sur le poste) ne laisse aucun survivant : sous l'`examine_globs` de `.cargo/mutants.toml`, `-f` ne restreint
  rien et muterait tout le domaine, quand `-F` garde les seuls mutants du module, `--lib` et `exclude_re` compris
  (`--list -F '^src/domaine/sortie\.rs:'` en rend 21, contre 354 pour `-f`, le 28/11/2026). Un survivant se tue
  par un test ; un mutant équivalent ou qui boucle s'exclut par son nom dans `.cargo/mutants.toml`, avec sa raison,
  et le commit de la tâche emporte ce fichier.
- À la fin de chaque tâche, avant son commit : `cargo fmt --all`, puis `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace` sans faute. Les tests du plan
  sont écrits en forme compacte, que rustfmt réécrit. Une invocation citée en commentaire sans `--fenetre`
  s'entend avec `--fenetre appli-essai`.
- Les essais de bout en bout se jugent par sondage, jamais par une relance ; un essai instable se dit à Edouard
  avant d'en changer le statut ; une précondition de l'environnement (le niveau haut du runner, 96 DPI) se vérifie
  d'abord et panique avec sa raison, pour qu'un rouge dise un défaut de `pilote` ou du runner, jamais un hasard.
- Chromium sert l'arbre depuis la copie du navigateur, qui suit la page en différé : dans un essai, une lecture
  qui suit un geste attend d'abord l'événement de la page qui prouve le geste (la `saisie` du texte complet, le
  `change`, le `focus`), puis `CALME`, comme les essais de la tranche 1 ; ou bien une lecture qui attend
  (`--attendre 3000`, `--egale`) précède la lecture unique qu'on juge. Le compte d'événements qui ne doivent pas
  venir se fait après `CALME`. Une preuve négative a son témoin : une précondition vérifiée, qui la rend probante.

## Points de vigilance

- Une lecture qui s'intercale entre le dernier contrôle du premier plan et l'envoi (le focus de KeePassXC, le
  niveau d'un dialogue, le point) : une fenêtre qui prend le premier plan pendant elle recevrait l'unité —
  `premier_plan_verifie_avant_chaque_unite` et ses jumeaux (Tâches 8, 9 et 10),
  `keepassxc_premier_plan_change_pendant_la_lecture_du_focus` (Tâche 8).
- Un élément qui disparaît entre la lecture et le geste, une liste rechargée : le code 2, jamais un plantage —
  `element_disparu_sort_en_2` (Tâche 9, et Tâche 14 sur une case que son clic retire) et
  `arret_uia_classe_les_codes` (Tâche 3).
- Un `Invoke` qui ouvre un dialogue modal et ne rend la main qu'à sa fermeture : le délai coupe un geste qui a
  agi, et une session qui lirait « échec » le rejouerait — `delai_pendant_le_geste_dit_qu_il_a_pu_agir` (Tâche 9).
- Un hôte qui ne répond plus, une page dont le seul JavaScript est figé : 3 sans attendre la fin du gel, puis
  l'arbre d'avant en 0 — `hote_qui_ne_repond_pas_sort_en_3` (Tâche 6), `capture_d_un_hote_qui_ne_repond_pas_sort_en_3`
  (Tâche 7), `arbre_d_un_hote_fige_sort_en_3`, `capture_d_un_hote_fige_sort_en_3` et
  `arbre_d_une_page_figee_rend_l_arbre_d_avant` (Tâche 13).
- Une valeur de mot de passe qu'une option ferait sortir par un détour (`--egale` comme oracle, `arbre`,
  `trouver`), ou une lecture ratée de `IsPassword` prise pour « pas un mot de passe » —
  `valeur_refuse_un_mot_de_passe_meme_avec_egale` et `arbre_ne_lit_jamais_la_valeur_d_un_mot_de_passe` (Tâche 6),
  la revue de la Tâche 11, `mot_de_passe_ne_sort_jamais` (Tâche 13).
- Un point de clic sur une fenêtre toujours au-dessus, ou hors de l'écran quand la fenêtre déborde, ou une fenêtre
  qui bouge entre deux crans — `clic_refuse_un_point_recouvert_dans_le_domaine`,
  `clic_refuse_un_point_hors_de_l_ecran` et `chaque_unite_relit_le_point` (Tâche 10),
  `clic_refuse_un_point_recouvert` (Tâche 14).
- Un poste qui n'est pas à 96 DPI, où `pilote` ne serait pas sensible au DPI de chaque écran : la capture rognée
  de travers, le clic hors du point jugé — `capture_sans_dpi_par_ecran_sort_en_3` (Tâche 7) et
  `pointeur_sans_dpi_par_ecran_sort_en_3` (Tâche 10).
- Une capture nommée dans un dossier absent, ou sur un fichier qui existe — `capture_dans_un_dossier_absent_sort_en_64`
  et `capture_remplace_un_fichier_existant` (Tâche 7).

## Mécanismes annoncés

- `src/lib.rs:29` — `COINIT_APARTMENTTHREADED` — COM s'initialise aujourd'hui en STA ; la Tâche 1 passe au MTA
- `src/domaine/touches.rs:150` — `(alt && porte("f6"))` — la règle de F6 que la Tâche 2 étend à Ctrl
- `src/domaine/touches.rs:707` — `"Ctrl+F6"` — Ctrl+F6, voisine permise aujourd'hui, que la Tâche 2 passe aux interdites
- `src/domaine/gestes.rs:244` — `Evenement::SourisNulle => {}` — les relâchements ignorent la souris ; la Tâche 2 y ajoute les boutons
- `src/domaine/gestes.rs:389` — `Travail::PasEncoreLivree(autre.nom())` — toute sous-commande hors de la tranche 1 sort en 64 ; les Tâches 6, 7, 9 et 10 en retirent douze
- `src/domaine/gestes.rs:450` — `refuser_keepassxc(frappe.is_some()` — le refus entier de KeePassXC, que la Tâche 8 remplace par la règle du champ
- `src/domaine/gestes.rs:535` — `fn juger_le_niveau` — le niveau ne se juge que pour P ; la Tâche 8 l'étend au processus d'un dialogue
- `src/domaine/gestes.rs:604` — `fn premier_plan_accepte` — la lecture du premier plan, l'instantané, la relecture : la garde d'une unité la fait juste avant l'envoi, et de nouveau après toute lecture d'entre-deux (décision 13)
- `src/domaine/gestes.rs:1793` — `fn premier_plan_verifie_avant_chaque_unite` — chaque envoi de `taper` suit exactement `[PremierPlan, Processus, PremierPlan]` ; les Tâches 8 à 10 lui donnent ses jumeaux
- `src/domaine/gestes.rs:797` — `fn aucun_geste` — les aides des tests de `gestes.rs` sont privées à son module de tests ; la Tâche 5 les passe dans `gestes::essai`
- `src/domaine/gestes.rs:897` — `fn lecture_n_active_rien` — un test de ce nom existe déjà : celui de la Tâche 6 se nomme autrement
- `src/domaine/gestes.rs:621` — `fn sonder<T>(` — la sonde bornée, que la sonde du document vide et celles de `choisir` reprennent
- `src/domaine/gestes.rs:2344-2348` — `"focus --fenetre x"` — l'essai du 64 « pas encore livrée » vise `focus`, que la Tâche 6 livre : il passe à `fermer`
- `src/domaine/fenetre.rs:147` — `pub fn accepte` — la règle du premier plan accepté, que la fenêtre sous le point reprend
- `src/domaine/refus.rs:59` — `pub fn keepassxc_dans` — KeePassXC se juge sur P et sur chaque fenêtre enfant de W
- `src/domaine/cli.rs:343` — `rang: Option<u32>,` — `--rang` admet 0 aujourd'hui ; la Tâche 4 le borne par clap
- `src/domaine/cli.rs:505` — `L'abscisse, dans la fenêtre.` — l'aide de `--x`, que la Tâche 4 précise, avec les nombres négatifs
- `src/domaine/cli.rs:511` — `Les crans, de -50 à 50, sauf 0.` — l'aide de `--crans`, qui ne dit pas le sens ; la Tâche 4 l'ajoute
- `src/domaine/faux.rs:453` — `lot.contains(&Evenement::SourisNulle)` — seul le mouvement nul ouvre le premier plan dans le faux ; un déplacement absolu ne l'ouvre pas
- `src/domaine/faux.rs:115` — `pub vol: Option<(usize, PremierPlan)>,` — le faux ne change le premier plan qu'après le n-ième envoi ; la Tâche 5 ajoute les vols après une lecture de l'arbre, du focus et un appel qui agit
- `src/domaine/faux.rs:145` — `pub verr_maj: bool,` — Verr. Maj sans état illisible ; la Tâche 5 la passe en `Option<bool>`
- `src/domaine/bureau.rs:95` — `fn verr_maj(&mut self) -> bool;` — la couche choisit aujourd'hui le sens d'une lecture ratée ; la Tâche 5 rend une `Option`, que le domaine juge
- `src/domaine/sortie.rs:148` — `pub fn arret` — une issue a une sortie ou une erreur ; la Tâche 3 ajoute l'issue faite avec un avis
- `tests/statique.rs:87` — `MOTS_DE_LANCEMENT` — les lancements refusés dans `tests/` ; la Tâche 1 ajoute `CreateProcess` et `ShellExecute`
- `tests/statique.rs:240` — `starts_with("#[test]")` — seul un `#[test]` en tête de ligne se voit ; la Tâche 1 refuse les autres
- `src/bureau/clavier.rs:133` — `& OCTET_BAS` — le seul appel de `ToUnicodeEx` passe l'octet bas, ce que la règle de la Tâche 1 garde
- `src/bureau/clavier.rs:232` — `pub fn verr_maj` — Verr. Maj lue sur le fil de `pilote` ; la Tâche 11 la lit sur un fil neuf
- `src/bureau/fenetres.rs:160` — `fn aumid` — l'AUMID lu sur le fil de `lancer`, en STA ; la Tâche 1 le lit sur un fil neuf en STA, le fil principal passant au MTA
- `tests/commun/mod.rs:809` — `COINIT_APARTMENTTHREADED` — le processus des essais lit l'AUMID en STA, comme le fera `pilote`
- `src/systeme.rs:103` — `SetProcessDpiAwarenessContext` — son résultat s'ignore ; la Tâche 5 ajoute au trait la sensibilité lue, que `capture`, `clic` et `molette` exigent
- `.cargo/mutants.toml:3` — `examine_globs` — sous lui, `-f` ne restreint pas la mutation : chaque tâche passe `-F '^src/domaine/<module>\.rs:'`
- `tests/commun/mod.rs:976` — `fn lire_verr_maj` — la lecture sur un fil neuf que la Tâche 11 reprend
- `docs/registre.md:14` — `ce que relit la revue de toute tranche qui en ajoute un` — le critère de LE-2, qui ne se ferme jamais ; la Tâche 1 le remplace
- `docs/registre.md:13` — `ArrowDown` — le critère de LE-1, que `touche_bas_donne_arrowdown` (Tâche 15) remplit
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:85` — `L'envoi qui suit part juste après elle` — la règle que la garde d'une unité tient, lectures d'entre-deux comprises (décision 13)
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:112` — `le refus est entier` — le refus de KeePassXC jusqu'à cette tranche
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:245` — `change de fenêtre rejoint la liste` — la phrase à laquelle `Ctrl+Tab` fait exception, que la Tâche 2 amende
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:324` — `CoInitializeEx(COINIT_APARTMENTTHREADED)` — la précision de la tranche 1 que la décision 2 amende
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:351` — `F6, Ctrl+F6 et Maj+F6 restent permis` — la précision de la tranche 1 que la décision 14 amende
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:250` — `Ctrl+Alt+<lettre>` — la limite des raccourcis globaux, qui reste
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:285` — `**Après un geste**` — la règle du journal après un geste, que la décision 9 étend à l'automatisation
- `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md:371` — `KeePassXC ne reçoit ni` — la conséquence que la Tâche 8 retire
- `docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md:106` — `KeePassXC : touche et taper sont refusés dans cette version` — la ligne du tableau que la Tâche 8 retire
- `docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md:191` — `compte à partir de 1` — le rang, que clap bornera
- `docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md:311-312` — `Un changement passe par un nouvel ADR` — la conséquence que la décision 1 amende
- `docs/modele/documentation.md:240` — `Il s'amende en place` — la règle d'amendement que suit la décision 1
- `docs/modele/documentation.md:246` — `n'a pas à recevoir sa section « Confirmation »` — un ADR d'avant la v0.15.0 reçoit sa section « Confirmation » à son prochain amendement
- `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md:45` — `un changement ultérieur passe par un nouvel ADR` — la phrase du plan maître que la décision 1 précise
- `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md:188` — `le message disant le niveau illisible` — l'attente du plan maître que la décision 16 change
- `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md:203` — `ne baisse jamais` — la règle des délais que la décision 17 précise
- `.github/workflows/ci.yml:53` — `timeout-minutes: 9` — le délai de `windows`, remesuré à la Tâche 17
- `.github/workflows/ci.yml:147` — `timeout-minutes: 19` — le délai de `mutants`, déjà sous deux fois son passage du 22/11
- `.github/workflows/ci.yml:141` — `*.exe` — l'artefact exclut les copies ; la Tâche 13 y ajoute les PNG, la Tâche 16 les sorties du jeton restreint
- `essai/appli/src/page.html:65` — `<div id="cadre">` — le cadre qui défile n'a ni rôle, ni nom, ni `tabindex` ; la Tâche 12 lui en donne
- `essai/appli/src/page.html:121` — `geler 8` — le gel de l'hôte, sans signal de début ; la Tâche 12 en ajoute un
- `essai/appli/src/main.rs:202` — `Ordre::Geler(secondes)` — l'hôte dort sans écrire ; la Tâche 12 écrit `{"genre":"gel"}` avant
- `tests/essais_gardes.rs:392` — `fn copie_keepassxc_refuse_taper_et_touche` — l'essai du refus entier, que la Tâche 15 remplace
- `tests/essais_gardes.rs:445` — `2 => assert!(!a.message().contains(&chevrons)` — l'essai des Paramètres réduits ne dit pas quelle règle refuse

## Outils et cas limites

- `clap` — une valeur qui commence par un tiret passe pour une option, sauf `allow_negative_numbers` ou `allow_hyphen_values` — `clic_accepte_un_point_negatif`, `options_de_texte_admettent_un_tiret`, Tâche 4
- `clap`, `value_parser!(u32).range(1..)` — 0 arrive en `ValueValidation`, que l'analyse dit « valeur refusée » — `rang_zero_sort_en_64`, Tâche 4
- `unicode-normalization`, la forme NFC — elle ne touche pas les espaces insécables — `nom_compte_les_insecables_pour_des_espaces`, Tâche 4
- `png` 0.18.1 — sans `set_color`, l'encodeur prend des niveaux de gris, une largeur ou une hauteur nulles refusent l'en-tête, et l'image n'est finie qu'après `finish` — `png_sans_alpha` et `capture_vide_sort_en_3`, Tâche 7
- `ElementFromHandleBuildCache` et `GetCachedChildren` — un élément sans enfant rend une erreur de code `HRESULT(0)`, pas une liste vide — `lecture_sonde_un_document_vide` (Tâche 13), le seul essai qui exerce la couche ; le domaine, qui reçoit la liste plate, rejoue la sonde par son homonyme de la Tâche 6
- l'automatisation, par `GetCachedPatternAs` — un motif non offert arrive en erreur de code `HRESULT(0)` ; un élément disparu en `UIA_E_ELEMENTNOTAVAILABLE`, un fournisseur mort en `RPC_E_DISCONNECTED` — `arret_uia_classe_les_codes` (Tâche 3), `invoquer_un_element_sans_le_geste_sort_en_3` et `element_disparu_sort_en_2` (Tâches 9 et 14, la seconde sur une case que son clic retire de la page)
- `FormatMessage`, par `Error::message()` de `windows-result` — il n'a aucun texte pour `UIA_E_ELEMENTNOTENABLED` ni `UIA_E_INVALIDOPERATION`, et prête à `UIA_E_NOTSUPPORTED` celui d'un autre code de même numéro — la couche écrit toujours le code, et `arret_uia_classe_les_codes` (Tâche 3) classe ces trois codes sans lire le texte
- `Invoke`, et `Toggle` ou `Select` passés par le proxy des fenêtres — un contrôle qui ouvre un dialogue modal peut ne rendre la main qu'à sa fermeture, et le délai de transaction le coupe après que le geste a agi — `delai_pendant_le_geste_dit_qu_il_a_pu_agir`, Tâche 9
- `IUIAutomation2` — `AutoSetFocus` vaut vrai par défaut : chaque motif déplacerait le focus avant d'agir — `SetAutoSetFocus(false)` à l'ouverture, et `defilables_et_defiler`, qui vérifie que le cadre défilé ne prend pas le focus, son témoin le lui donnant ensuite, Tâches 11 et 14
- `GetFocusedElement` — il rend le focus du système, même hors de la fenêtre retenue, et, sur Chromium avant la première requête, le document ou la fenêtre hôte — `focus_hors_de_la_fenetre_visee_sort_en_1` (Tâches 6 et 13), `keepassxc_sonde_l_arbre_avant_le_focus` (Tâche 8), `copie_keepassxc_garde_son_champ` (Tâche 15)
- `Toggle` et `Select` de Chromium — ils rendent la main avant que l'état change — `choisir_attend_l_etat_avant_de_rebasculer` (Tâche 9), puis `choisir_attend_une_case_lente` (Tâche 14)
- l'arbre de Chromium — il ne se bâtit qu'à la première requête, et le navigateur le sert depuis sa copie quand la page est figée — `lecture_sonde_un_document_vide` (Tâches 6 et 13), `arbre_d_une_page_figee_rend_l_arbre_d_avant` (Tâche 13)
- `SendMessageTimeoutW`, `SMTO_ABORTIFHUNG` — il attend tout le délai tant que Windows ne tient pas la fenêtre pour figée, environ 5 s — `hote_qui_ne_repond_pas_sort_en_3` (Tâche 6), `arbre_d_un_hote_fige_sort_en_3`, qui borne la durée à 6 s (Tâche 13)
- `PrintWindow`, avec `PW_RENDERFULLCONTENT` — une fenêtre réduite rend une image noire, l'image porte l'ombre du cadre, et l'alpha n'a pas de sens — `capture_refuse_une_fenetre_reduite`, `capture_noire_sort_en_3` et `png_sans_alpha` (Tâche 7), `capture_rogne_l_ombre` (Tâche 13)
- `DWMWA_EXTENDED_FRAME_BOUNDS` — il ne suit pas la mise à l'échelle : il est en pixels physiques, que `GetWindowRect` et `GetClientRect` ne rendent que si `pilote` est sensible au DPI de chaque écran, ce que `SetProcessDpiAwarenessContext` peut refuser sans un mot — `capture_sans_dpi_par_ecran_sort_en_3` (Tâche 7), `pointeur_sans_dpi_par_ecran_sort_en_3` (Tâche 10), `rognage_garde_le_cadre_dwm` (Tâche 7), `capture_rogne_l_ombre` (Tâche 13), à 96 DPI seulement : hors de 96 DPI, rien n'est éprouvé en CI
- `DeleteObject` et `CreateDIBSection` — une bitmap encore choisie dans un DC ne se supprime pas, et ses bits ne se lisent qu'après `GdiFlush` — la suite de `capturer` (Tâche 11), puis `capture_relue` (Tâche 13)
- `WindowFromPoint` — il rend la fenêtre fille sous le point, pas sa fenêtre de premier niveau, et rien hors de l'écran — `clic_sur_un_dialogue_de_w_passe` et `clic_refuse_un_point_hors_de_l_ecran` (Tâche 10), `clic_refuse_un_point_recouvert` (Tâche 14)
- `SendInput`, la souris — `MOUSEEVENTF_ABSOLUTE` prend des coordonnées de 0 à 65 535, (65535, 65535) au coin bas droit, sans dire ni l'échelle ni l'arrondi ; `VIRTUALDESK` vise tout le bureau virtuel ; `mouseData` est un `u32` qui porte la molette négative — `normalisation_vise_le_quart_du_pixel` (Tâche 10), `clic_clique_au_point` et `clic_au_bord_du_bureau_virtuel`, qui relisent la position du pointeur, le second sur la dernière colonne du bureau virtuel, et `molette_defile_la_roulette` (Tâche 14)
- `GetSystemMetrics(SM_SWAPBUTTON)` — les boutons inversés échangent le gauche et le droit de `SendInput`, qui parle des boutons physiques — `boutons_inverses_echangent_les_boutons` (Tâche 10), `boutons_inverses_cliquent_a_gauche` (Tâche 14, sous `SwapMouseButton`)
- `GetKeyState` — la file d'entrée d'un fil vivant ne suit l'état du clavier qu'en lisant ses messages — Verr. Maj lue sur un fil neuf (Tâche 11), et `comme_au_clavier_refuse_quand_verr_maj_est_allumee`, l'essai de la tranche 1, rejoué en CI
- `SHGetPropertyStoreForWindow` — sa lecture sous le MTA n'est ni documentée ni éprouvée : `pilote` le lit sur un fil neuf en STA, comme `Parametres::regle` dans le processus des essais — `parametres_reduits_restent_refuses`, qui dit la règle qui refuse, Tâche 15
- `OpenProcessToken`, depuis un jeton au niveau moyen — il lit le jeton d'une cible élevée du même compte (ADR-0005), et « illisible » n'arrive que pour un autre compte — `refuse_une_fenetre_plus_elevee`, qui attend « fenêtre plus élevée que pilote », Tâche 16
- `CreateProcessAsUserW` — sans bloc d'environnement Unicode, le journal tombe dans la sentinelle ; `SE_ASSIGNPRIMARYTOKEN` n'est pas exigé pour un jeton tiré de celui de l'appelant ; le niveau d'un fichier hérité ne se juge pas — `pilote_restreint_lit_les_fenetres`, `refuse_une_fenetre_plus_elevee` et `refuse_une_fenetre_plus_elevee_au_jeton_abaisse`, Tâche 16
- `CheckTokenMembership` — il exige un jeton d'emprunt, pas un jeton primaire — la précondition des essais du jeton, Tâche 16
- `icacls /setintegritylevel` — sans `(OI)(CI)`, les fichiers créés ensuite gardent le niveau haut, et sans `/T` ce qui existe déjà le garde aussi — le journal écrit par `pilote` au niveau moyen, lu par `refuse_une_fenetre_plus_elevee`, Tâche 16
- `actions/upload-artifact` — un motif de plus n'élargit rien que les exclusions retirent — l'étape de dépôt relue aux Tâches 13 et 16, ses exclusions gardées en dernier
- `cargo-mutants`, `-f` — sous l'`examine_globs` de `.cargo/mutants.toml`, il ne retire rien : `-f src/domaine/sortie.rs` liste les 354 mutants du domaine — `-F '^src/domaine/<module>\.rs:'`, dont `--list` rend les seuls mutants du module (21 pour `sortie.rs`, le 28/11/2026), à chaque passage de tâche, Tâches 2 à 10
- `cargo test`, deux filtres avant `--` — il refuse le second (« unexpected argument »), sans compiler ni lancer un test, ce qu'une étape « voir rouge » prendrait pour le rouge attendu — chaque étape qui nomme deux filtres les met après `--`, Tâches 2, 3, 4, 6 et 9

## Effets cherchés

- ADR-0005 et ADR-0006 amendés en place, avec leur section « Confirmation » — `docs/decisions/` et la compétence de la tranche 4, qui les décrira — la revue de la branche (Tâche 17) relit chaque ligne de la section des changements de ce plan contre le texte amendé, et chaque test nommé dans « Confirmation » contre le code
- COM en MTA, l'AUMID lu sur un fil neuf en STA — chaque processus `pilote`, l'AUMID et l'automatisation — `parametres_reduits_restent_refuses` et `copie_systemsettings_refusee` en CI (Tâche 15), et chaque essai de lecture de la Tâche 13
- la garde d'une unité, le premier plan relu après toute lecture d'entre-deux — chaque geste de `pilote`, au clavier, au pointeur et par l'automatisation — `premier_plan_verifie_avant_chaque_unite` et ses jumeaux (Tâches 8 à 10), puis `copie_keepassxc_garde_son_champ` et les essais des gestes en CI
- Ctrl+F6 interdite — toute session qui la frapperait, et la compétence de la tranche 4 — `ctrl_f6_est_interdite` et `ctrl_f6_avant_l_inconnu`, et la phrase d'ADR-0005 au présent
- quatre règles du contrôle statique — chaque PR à venir, des lancements, du jeton, du cache et des tests — leurs plantations dans `tests/statique.rs`, et `sources_respectent_les_regles` vert sur la tête
- LE-1 et LE-2 clos — `docs/registre.md` — LE-2 par la règle `ToUnicodeEx` de la Tâche 1 ; LE-1 par `touche_bas_donne_arrowdown`, vert dans le job `windows` (Tâche 15), son état passé à « clos » à la Tâche 17
- les PNG et les sorties du jeton restreint dans l'artefact — un essai rouge en CI — les motifs relus aux revues des Tâches 13 et 16, et le premier échec qui les montre
- les délais de `windows` et `mutants` remesurés — la CI de chaque PR — `gh run view <id> --json jobs` sur un passage à froid de la tête, et le commentaire de chaque délai
- « pas encore livrée » réservé aux sous-commandes de la tranche 3 — les sessions qui appellent `pilote` — `sous_commande_pas_encore_livree_sort_en_64` sur `fermer`, et chaque sous-commande de la tranche 2 analysée et exécutée par ses tests
- les options de fenêtre et les cibles de l'application d'essai — les essais des tranches 3 et 5, qui la reprennent — les tests `regles` de l'application, lancés par la sentinelle

## Revues

- la revue de chaque tâche, sur la tête qui part, par une session neuve : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12, Tâche 13, Tâche 14, Tâche 15, Tâche 16, Tâche 17
- la lecture de la CI de chaque envoi des essais de bout en bout, avant la revue de sa tâche : Tâche 13, Tâche 14, Tâche 15, Tâche 16
- la revue de la branche entière, par une session neuve, sur la tête commitée, avant que la PR quitte le brouillon : Tâche 17

---

## Les fichiers

```
Cargo.toml, Cargo.lock          png 0.18.1, unicode-normalization 0.1.26 ; Win32_Graphics_Gdi, Win32_Storage_Xps,
                                Win32_UI_Accessibility
src/lib.rs                      COM en MTA
src/domaine/mod.rs              + element, selecteur, lectures, capture, automatisation, pointeur
src/domaine/sortie.rs           + Issue::avis
src/domaine/touches.rs          + Evenement::{Pointeur, Bouton, Molette}, CRAN ; Ctrl+F6 interdite
src/domaine/element.rs          ElementLu, Defile, ErreurUia, la table des types, la ligne d'un élément,
                                document_vide, arret_uia ; essai::arbre_d_essai (#[cfg(test)])
src/domaine/selecteur.rs        normaliser, analyser_le_selecteur, repond, candidats, choisir_un, sous_arbres
src/domaine/bureau.rs           + Rect, Point, DelaisUia, FocusLu, EtatLu, GesteUia, Image ; quatorze méthodes ;
                                verr_maj en Option
src/domaine/lectures.rs         focus, arbre, trouver, valeur, defilables ; la réponse, l'ouverture, la sonde, l'attente
src/domaine/capture.rs          rogner, toute_noire, png, capturer
src/domaine/automatisation.rs   focaliser, invoquer, choisir, defiler
src/domaine/pointeur.rs         normaliser, lot, pointer
src/domaine/gestes.rs           Travail étendu ; Suivi partagé ; preparer_le_geste ; la garde d'une unité ; KeePassXC ;
                                Verr. Maj illisible ; essai (les aides des tests, #[cfg(test)])
src/domaine/faux.rs             l'automatisation, la capture et le pointeur simulés ; les vols ; les niveaux par PID
src/domaine/cli.rs              --rang de 1 à u32::MAX ; --x et --y négatifs ; aides
src/bureau/mod.rs               BureauWindows : le client de l'automatisation, les éléments de la dernière lecture
src/bureau/uia.rs               CUIAutomation8, ses délais, la requête de cache, la lecture, les motifs, ErreurUia
src/bureau/capture.rs           PrintWindow, la section DIB, le cadre de DWM, la zone cliente
src/bureau/fenetres.rs          + repond, zone_cliente, fenetre_sous, bureau_virtuel, sensible_au_dpi ; l'AUMID sur un
                                fil neuf en STA
src/bureau/clavier.rs           + la souris dans `entree`, boutons_inverses ; verr_maj sur un fil neuf
tests/statique.rs               les quatre règles de la décision 15
tests/commun/mod.rs             image_png, zone_cliente, cadre_dwm, position_du_pointeur, dpi, fenetre_au_point,
                                bureau_virtuel, BoutonsInverses, Parametres::regle ; pilote_restreint, Jeton
tests/essais_lecture.rs         focus, arbre, trouver, valeur, defilables, capture
tests/essais_gestes.rs          focaliser, invoquer, choisir, defiler, clic, molette
tests/essais_gardes.rs          KeePassXC, le jeton restreint, SystemSettings, les Paramètres
tests/essais_clavier.rs         touche_bas_donne_arrowdown
essai/appli/src/main.rs         --couleur, --au-dessus, --position, --taille ; la ligne `gel`
essai/appli/src/page.html       clic, double, droit, change, defile ; cible, roulette, case lente, case fugace,
                                retarder ; le cadre nommé et focalisable
.github/workflows/ci.yml        l'artefact ; les délais
docs/decisions/0005-…, 0006-…   amendés
docs/registre.md                LE-1 et LE-2
CLAUDE.md                       l'état
```

---

### Tâche 1 : COM en MTA et le contrôle statique

**Fichiers :** modifier `src/lib.rs`, `src/bureau/fenetres.rs` (`aumid`), `tests/statique.rs`, `docs/registre.md`,
`docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md`, et chaque fichier de `src/`, `tests/` et
`essai/` dont un commentaire cite « décision N ».

**Interfaces :**
- Consomme : `controler`, `mots_refuses`, `refuse`, `admet` de `tests/statique.rs`.
- Produit :
  ```rust
  // tests/statique.rs
  const MOTS_DE_LANCEMENT: &[&str] = &["CARGO_BIN_EXE", "Command::new", "std::process", "spawn", "CreateProcess", "ShellExecute"];
  /// Refusés dans `src/`, préfixes bornés à gauche : le jeton de la décision 14 vit dans les essais ; la valeur
  /// ne passe jamais par le cache, ni par le motif LegacyIAccessible, qui la porte aussi.
  const MOTS_DU_JETON_ET_DU_CACHE: &[&str] = &["CreateRestrictedToken", "CreateProcessAsUser", "SetTokenInformation",
      "UIA_ValueValuePropertyId", "UIA_LegacyIAccessibleValuePropertyId", "UIA_LegacyIAccessiblePatternId",
      "IUIAutomationLegacyIAccessiblePattern"];
  /// Le seul fichier qui appelle `ToUnicodeEx`, et la marque de l'octet bas.
  const CLAVIER: &str = "src/bureau/clavier.rs"; const OCTET_BAS: &str = "OCTET_BAS";
  fn fautes_de_la_ligne_de_test(chemin: &str, numero: usize, ligne: &str) -> Option<String>;
  fn fautes_de_to_unicode_ex(chemin: &str, lignes: &[&str]) -> Vec<String>;
  ```
  `controler` appelle les deux nouvelles fonctions ; une ligne de commentaire (`//` en tête) ne compte toujours
  pas.

- [ ] **Étape 1 : les tests, rouges.**
  ```rust
  #[test] fn refuse_un_lancement_win32_dans_un_essai() {
      refuse("tests/essais_x.rs", "let _ = CreateProcessW(None, …);", "tests/essais_x.rs:1 contient « CreateProcess »");
      refuse("tests/essais_x.rs", "ShellExecuteExW(&mut info);", "tests/essais_x.rs:1 contient « ShellExecute »");
      admet("tests/commun/mod.rs", "CreateProcessAsUserW(…);"); }
  #[test] fn refuse_le_jeton_et_la_valeur_en_cache_dans_src() { for mot in MOTS_DU_JETON_ET_DU_CACHE {
      refuse("src/bureau/x.rs", &format!("let _ = {mot}W(…);"), &format!("src/bureau/x.rs:1 contient « {mot} »")); }
      admet("tests/commun/mod.rs", "CreateRestrictedToken(…); SetTokenInformation(…);"); }
  #[test] fn refuse_un_test_hors_de_la_tete_de_ligne() {
      refuse("src/domaine/x.rs", "#[cfg(test)] #[test]\nfn a() {}", "src/domaine/x.rs:1 : #[test] hors de la tête de ligne");
      refuse("tests/essais_x.rs", "#[ignore = \"il pilote le bureau : la CI le lance\"] #[test]\nfn a() {}",
             "tests/essais_x.rs:1 : #[test] hors de la tête de ligne");
      admet("src/domaine/x.rs", "// un #[test] cité dans un commentaire");
      admet("tests/statique.rs", "let t = \"#[test]\";"); }
  #[test] fn refuse_to_unicode_ex_hors_du_clavier() {
      refuse("src/bureau/fenetres.rs", "let n = unsafe { ToUnicodeEx(vk, b & OCTET_BAS, &e, &mut t, 0, None) };",
             "src/bureau/fenetres.rs:1 : ToUnicodeEx hors de src/bureau/clavier.rs"); }
  #[test] fn refuse_to_unicode_ex_sans_l_octet_bas() {
      refuse("src/bureau/clavier.rs", "ToUnicodeEx(\n    u32::from(vk),\n    balayage,\n    &clavier,\n)",
             "src/bureau/clavier.rs:1 : ToUnicodeEx sans l'octet bas du balayage");
      admet("src/bureau/clavier.rs", "ToUnicodeEx(\n    u32::from(vk),\n    balayage & OCTET_BAS,\n)");
      admet("src/bureau/clavier.rs", "use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;"); }
  ```
  `chaque_mot_de_lancement_a_sa_plantation` couvre les deux mots ajoutés. Ajouter
  `chaque_mot_du_jeton_et_du_cache_a_sa_plantation`, sur le modèle de `chaque_mot_refuse_a_sa_plantation`.
- [ ] **Étape 2 : voir rouge** : `cargo test --test statique`.
- [ ] **Étape 3 : les règles.** Dans `mots_refuses`, `MOTS_DU_JETON_ET_DU_CACHE` pour tout chemin qui commence par
  `src/`. `fautes_de_la_ligne_de_test` : une ligne qui porte `#[test]` sans que son début, blancs ôtés, le soit, hors
  de `STATIQUE`. `fautes_de_to_unicode_ex` : chaque ligne qui porte `ToUnicodeEx(` (avec la parenthèse), hors de
  `STATIQUE` ; hors de `CLAVIER`, faute ; dans `CLAVIER`, faute si ni elle ni ses trois suivantes ne portent
  `OCTET_BAS`. Le commentaire d'en-tête du fichier dit les quatre règles.
- [ ] **Étape 4 : voir vert** : `cargo test --test statique`, `sources_respectent_les_regles` compris.
- [ ] **Étape 5 : COM en MTA, l'AUMID sur un fil neuf.** `src/lib.rs` : `COINIT_MULTITHREADED` ; le commentaire dit
  que le client de l'automatisation vit sous le MTA. `src/bureau/fenetres.rs` : `aumid` passe la fenêtre (sa valeur
  entière, un `HWND` n'étant pas `Send`) à un fil neuf (`thread::Builder::spawn`), qui appelle
  `CoInitializeEx(None, COINIT_APARTMENTTHREADED)`, lit l'AUMID comme aujourd'hui, appelle `CoUninitialize` si
  l'initialisation a réussi, et rend sa lecture par `join` ; un fil qui ne naît pas, ou qui panique, rend `None`. Le
  commentaire dit pourquoi (décision 2), et qu'un AUMID illisible vaut `None`, que le domaine juge.
- [ ] **Étape 6 : les renvois.** Chaque « (décision N) » d'un commentaire de `src/`, `tests/` et `essai/` devient le
  renvoi à la section d'ADR qui porte la règle (`ADR-0005, « Le niveau d'intégrité »`), ou, à défaut, à
  `CLAUDE.md` ; `git grep -n "décision [0-9]" -- src tests essai` ne rend rien, sauf une citation d'un plan dans
  `tests/statique.rs` qui dit pourquoi elle reste.
- [ ] **Étape 7 : ADR-0005**, premier amendement (décision 1) : la ligne `- **Amendé** le <date> : tranche 2 du lot
  1 — COM en MTA` ; la puce `CoInitializeEx(COINIT_APARTMENTTHREADED)` dit le MTA, l'AUMID lu sur un fil neuf en
  STA, et la limite d'un AUMID d'une autre application que rien n'éprouve en CI ; la section `## Confirmation` naît,
  avec `tests/statique.rs` pour « Aucun `SetValue` » et pour la valeur hors du cache et du motif LegacyIAccessible ;
  la sous-section `### Ce que la tranche 2 précise ou change` naît, avec ce point.
- [ ] **Étape 8 : LE-2.** Son critère de sortie : « la règle `ToUnicodeEx` de `tests/statique.rs` : un appel hors de
  `src/bureau/clavier.rs`, ou sans `OCTET_BAS` dans les trois lignes qui le suivent, fait échouer
  `sources_respectent_les_regles` » ; son état : « clos ».
- [ ] **Étape 9 : commit** : `git add src/lib.rs src/bureau/fenetres.rs tests/statique.rs docs/registre.md docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md`
  et chaque fichier de l'étape 6, par son chemin, puis
  `git commit -m "feat(statique): COM en MTA, AUMID en STA, lancements Win32, jeton et ToUnicodeEx gardés"`.

### Tâche 2 : Ctrl+F6, les événements du pointeur et leurs relâchements

**Fichiers :** modifier `src/domaine/touches.rs`, `src/domaine/gestes.rs` (`relachements`, `meme_touche`,
`relachee`), `src/domaine/faux.rs`, `src/bureau/clavier.rs` (`entree`), ADR-0005.

**Interfaces :**
- Consomme : `refuser_l_interdite`, `relachements`, `insertion_complete`.
- Produit :
  ```rust
  // touches.rs
  pub enum Evenement { Touche { vk: u8, scan: u8, etendue: bool, relachee: bool }, Unicode { unite: u16, relachee: bool },
      SourisNulle,
      /// Un déplacement absolu, en coordonnées normalisées du bureau virtuel (0 à 65 535).
      Pointeur { x: u16, y: u16 },
      /// Le bouton gauche, ou le droit, enfoncé ou relâché.
      Bouton { droit: bool, relache: bool },
      /// Un cran de molette : +CRAN vers le haut, -CRAN vers le bas.
      Molette { delta: i32 } }
  pub const CRAN: i32 = 120;
  ```
  Dans la couche, `entree` rend un `MOUSEINPUT` : `Pointeur` en `MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE |
  MOUSEEVENTF_VIRTUALDESK` (`dx`, `dy`) ; `Bouton` en `LEFTDOWN`, `LEFTUP`, `RIGHTDOWN` ou `RIGHTUP` ; `Molette` en
  `MOUSEEVENTF_WHEEL`, `mouseData` étant `delta as u32`. Le faux ne compte pas `Pointeur` comme le mouvement nul.

- [ ] **Étape 1 : les tests, rouges.**
  ```rust
  // touches.rs : touches_interdites_sortent_en_1 gagne "Ctrl+F6" et "ctrl+maj+f6" ;
  // interdites_dans_tout_ordre_et_avant_l_inconnu gagne "F6+Ctrl", "Maj+F6+Ctrl" et "Ctrl+F6+Bidule" ;
  // voisines_des_interdites_sont_permises perd "Ctrl+F6" et "Ctrl+Maj+F6", garde "F6", "Maj+F6", "Ctrl+Tab", "Ctrl+Maj+Tab".
  #[test] fn ctrl_f6_est_interdite() { for t in ["Ctrl+F6", "CTRL+F6", "Ctrl+Alt+F6", "Maj+Ctrl+F6"] {
      assert_eq!(analyser_touche(t), Err(Arret::Refus(format!("touche interdite : {t}"))), "{t}"); }
      for t in ["F6", "Maj+F6", "Ctrl+Tab", "Ctrl+Alt+A"] { assert!(analyser_touche(t).is_ok(), "{t}"); } }
  // gestes.rs
  #[test] fn ctrl_f6_avant_l_inconnu() { let mut b = FauxBureau::avec_appli();
      assert_eq!(x(&mut b, "touche --fenetre appli-essai Bidule Ctrl+F6"), Issue::arret(Arret::Refus("touche interdite : Ctrl+F6".into())));
      assert!(!appele(&b, |a| matches!(a, AppelFaux::Fenetres))); }
  #[test] fn relachements_relachent_les_boutons() {
      let debut = [Evenement::Pointeur { x: 1, y: 2 }, Evenement::Bouton { droit: false, relache: false }];
      assert_eq!(relachements(&debut, &mut |_| 0), vec![Evenement::Bouton { droit: false, relache: true }]);
      let droit = [Evenement::Bouton { droit: true, relache: false }, Evenement::Bouton { droit: true, relache: true },
                   Evenement::Molette { delta: -CRAN }];
      assert!(relachements(&droit, &mut |_| 0).is_empty()); }
  #[test] fn un_clic_insere_en_partie_relache_son_bouton() { /* faux : insertions [1] ; insertion_complete(&mut b, &lot,
      2), le lot étant [Pointeur, Bouton enfoncé, Bouton relâché] : Err(Echec("échec du système : SendInput a inséré 2
      événements sur 3 ; relâchement : 1 sur 1")), et l'envoi du relâchement est [Bouton { droit: false, relache: true }] */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- domaine::touches domaine::gestes`.
- [ ] **Étape 3 : le code.** `refuser_l_interdite` : `((alt || ctrl) && porte("f6"))`, son commentaire au présent.
  `relachements` : un `Bouton` enfoncé et non relâché dans le début inséré se relâche, dans l'ordre inverse des
  enfoncements, touches et boutons mêlés ; `Pointeur`, `Molette` et `SourisNulle` ne relâchent rien. `meme_touche`
  compare deux `Bouton` par `droit` ; `relachee` rend le `Bouton` relâché, et tel quel tout événement sans
  enfoncement. La couche : `entree`, comme dit plus haut.
- [ ] **Étape 4 : voir vert**, puis la mutation : `cargo mutants -p pilote -F '^src/domaine/touches\.rs:'` et
  `-F '^src/domaine/gestes\.rs:'`.
- [ ] **Étape 5 : ADR-0005.** « Les touches interdites » nomme Ctrl+F6 dans la forme de la règle (« Alt+F6 ou
  Ctrl+F6, qui passent d'une fenêtre à l'autre d'un même programme »), dit la limite de `Ctrl+Tab` (« permis pour
  les onglets d'un dialogue ; dans Excel, il change de classeur, et la garde du premier plan n'arrête que l'unité
  suivante ») et garde `Ctrl+Alt+<lettre>` ; sa dernière phrase (« une touche dont on apprend qu'elle change de
  fenêtre rejoint la liste, avec son test ») gagne « sauf `Ctrl+Tab`, permis pour les onglets d'un dialogue
  (Edouard, 28/11/2026), dont la limite est dite plus bas » ; la précision de la tranche 1 (« F6, Ctrl+F6 et Maj+F6
  restent permis ») devient « F6 et Maj+F6 restent permis ; Ctrl+F6 rejoint les interdites à la tranche 2 » ; la ligne
  « Amendé » et la sous-section de la tranche 2 reçoivent ce point ; « Confirmation » nomme `ctrl_f6_est_interdite`.
- [ ] **Étape 6 : commit** : `git add src/domaine/touches.rs src/domaine/gestes.rs src/domaine/faux.rs src/bureau/clavier.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md`
  (et `.cargo/mutants.toml` s'il change), puis
  `git commit -m "feat(domaine): Ctrl+F6 interdite, événements du pointeur et leurs relâchements"`.

### Tâche 3 : la sortie et le modèle d'un élément

**Fichiers :** créer `src/domaine/element.rs` ; modifier `src/domaine/sortie.rs`, `src/domaine/mod.rs`.

**Interfaces :**
- Consomme : `Ligne`, `formater`, `valeur`, `echapper`, `Arret`, `Issue`.
- Produit :
  ```rust
  // sortie.rs
  impl Issue { /// Fait, ces lignes, et un avis sur la sortie d'erreur.
      pub fn avis(sortie: Vec<String>, avis: String) -> Issue }
  // element.rs
  /// Un élément lu. Chaque `Option` d'un motif est `Some` si et seulement si l'élément offre ce motif : `coche`,
  /// Toggle ; `choisi`, SelectionItem ; `deplie`, ExpandCollapse ; `lecture_seule`, Value (la ligne ne porte
  /// `valeur` que pour lui) ; `defile`, Scroll ; et `invocable`, Invoke.
  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct ElementLu { pub parent: Option<usize>, pub genre: i32, pub nom: String, pub id: String, pub classe: String,
      pub raccourci: String, pub acces: String, pub annonce: i32, pub motdepasse: bool, pub actif: bool, pub pid: u32,
      pub coche: Option<i32>, pub deplie: Option<i32>, pub choisi: Option<bool>, pub lecture_seule: Option<bool>,
      pub defile: Option<Defile>, pub invocable: bool }
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub struct Defile { pub vertical: bool, pub horizontal: bool }
  #[derive(Clone, Debug, PartialEq)] pub struct ErreurUia { pub code: u32, pub texte: String }
  pub const PREMIER_TYPE: i32 = 50_000;
  pub const TYPES: [&str; 41] = ["Button", "Calendar", "CheckBox", "ComboBox", "Edit", "Hyperlink", "Image", "ListItem",
      "List", "Menu", "MenuBar", "MenuItem", "ProgressBar", "RadioButton", "ScrollBar", "Slider", "Spinner", "StatusBar",
      "Tab", "TabItem", "Text", "ToolBar", "ToolTip", "Tree", "TreeItem", "Custom", "Group", "Thumb", "DataGrid",
      "DataItem", "Document", "SplitButton", "Window", "Pane", "Header", "HeaderItem", "Table", "TitleBar", "Separator",
      "SemanticZoom", "AppBar"];
  pub const GROUPE: i32 = 50_026; pub const DOCUMENT: i32 = 50_030; pub const FENETRE: i32 = 50_032;
  pub const VOLET: i32 = 50_033; pub const BARRE_DE_TITRE: i32 = 50_037;
  pub const UIA_DISPARU: u32 = 0x8004_0201; pub const RPC_DECONNECTE: u32 = 0x8001_0108;
  pub const OBJET_DECONNECTE: u32 = 0x8004_01FD; pub const UIA_DELAI: u32 = 0x8013_1505;
  pub const UIA_INACTIF: u32 = 0x8004_0200; pub const UIA_NON_PRIS: u32 = 0x8004_0204;
  pub const UIA_OPERATION_INVALIDE: u32 = 0x8013_1509;
  pub fn nom_du_type(genre: i32) -> &'static str;          // « Inconnu » hors de la table
  pub fn type_de_nom(nom: &str) -> Option<i32>;            // sans égard à la casse ASCII
  pub fn ancetres(arbre: &[ElementLu], i: usize) -> Vec<usize>;   // du parent à la racine
  pub fn profondeur(arbre: &[ElementLu], i: usize) -> usize;
  pub fn etats(e: &ElementLu) -> Vec<&'static str>;
  pub fn dans_de(arbre: &[ElementLu], i: usize) -> Option<&str>;
  pub fn document_vide(arbre: &[ElementLu]) -> bool;
  pub fn ligne_d_element(arbre: &[ElementLu], i: usize, valeur: Option<&str>, dans: bool, rang: Option<usize>) -> Ligne;
  pub fn arret_uia(e: &ErreurUia) -> Arret;
  #[cfg(test)] pub(crate) mod essai { pub fn e(parent: Option<usize>, genre: i32, nom: &str) -> ElementLu; pub fn arbre_d_essai() -> Vec<ElementLu>; }
  ```
  `arbre_d_essai()`, que les tâches suivantes réemploient par ses indices (pid 100, `actif` vrai partout) :

  | i | parent | type | nom | id | le reste |
  |---|---|---|---|---|---|
  | 0 | — | Window | Appli d'essai | | |
  | 1 | 0 | Pane | | | |
  | 2 | 1 | Document | Appli d'essai | | |
  | 3 | 2 | Button | Départ | depart | `invocable` |
  | 4 | 2 | Edit | Champ | champ | `lecture_seule: Some(false)` |
  | 5 | 2 | Edit | Mot de passe | motdepasse | `motdepasse`, `lecture_seule: Some(false)` |
  | 6 | 2 | CheckBox | Case | case | `coche: Some(0)` |
  | 7 | 2 | Group | Couleur | | |
  | 8 | 7 | RadioButton | Toutes | couleur-toutes | `choisi: Some(true)` |
  | 9 | 2 | Group | Taille | | |
  | 10 | 9 | RadioButton | Toutes | taille-toutes | `choisi: Some(true)` |
  | 11 | 2 | Group | Forme | | |
  | 12 | 11 | RadioButton | Toutes | forme-toutes | `choisi: Some(true)` |
  | 13 | 2 | Group | Options | | |
  | 14 | 13 | CheckBox | Option externe | option-externe | `coche: Some(0)` |
  | 15 | 13 | Group | Options | | |
  | 16 | 15 | CheckBox | Option interne | option-interne | `coche: Some(0)` |
  | 17 | 2 | Group | Cadre des lignes | cadre | `defile: Some(Defile { vertical: true, horizontal: false })` |
  | 18 | 2 | Text | Lignes | titre-defilee | |

- [ ] **Étape 1 : les tests, rouges.**
  ```rust
  #[test] fn issue_avis_garde_sa_sortie() { assert_eq!(Issue::avis(vec!["a".into()], "b".into()),
      Issue { code: Code::Fait, sortie: vec!["a".into()], erreur: Some("b".into()) }); }
  #[test] fn chaque_type_a_son_nom() { assert_eq!(nom_du_type(50_000), "Button"); assert_eq!(nom_du_type(50_026), "Group");
      assert_eq!(nom_du_type(50_040), "AppBar"); assert_eq!(nom_du_type(49_999), "Inconnu"); assert_eq!(nom_du_type(50_041), "Inconnu");
      assert_eq!(type_de_nom("radiobutton"), Some(50_013)); assert_eq!(type_de_nom("APPBAR"), Some(50_040));
      assert_eq!(type_de_nom("Inconnu"), None); assert_eq!(type_de_nom("Bouton"), None); }
  #[test] fn ligne_d_element_dans_l_ordre_des_cles() { /* l'arbre [e(None, 50_032, "Fenêtre"), e(Some(0), 50_026,
      "Groupe"), la case e(Some(1), 50_002, "Case") avec id "case", classe "c", raccourci "Ctrl+K", acces "k", annonce 1,
      coche Some(1), actif faux] : formater(&ligne_d_element(&arbre, 2, None, true, Some(2))) == "[CheckBox] <Case>
      id=case classe=c raccourci=Ctrl+K acces=k annonce=polite etat=coche,inactif dans=Groupe rang=2" */ }
  #[test] fn valeur_suit_la_classe() { /* Edit « Champ », id champ, lecture_seule Some(true), valeur Some("deux mots") :
      "[Edit] <Champ> id=champ valeur=\"deux mots\" etat=lecture-seule" ; valeur Some("") : "… valeur=\"\" …" */ }
  #[test] fn motdepasse_remplace_la_valeur() { /* motdepasse vrai, valeur Some("x") : la ligne porte motdepasse=oui, ni
      « valeur= » ni « x » */ }
  #[test] fn nom_vide_sans_chevrons_et_annonce_assertive() { /* Pane sans nom : "[Pane]" ; annonce 2 : "annonce=assertive" ;
      annonce 0 ou 3 : pas de clé */ }
  #[test] fn etats_dans_l_ordre() { /* coche 0 → ["decoche"] ; 2 → ["indetermine"] ; choisi Some(true) et actif faux →
      ["choisi", "inactif"] ; deplie 1 ou 2 → ["deplie"], 0 → ["replie"], 3 → [] ; lecture_seule Some(true) →
      ["lecture-seule"] ; tout ensemble, dans l'ordre d'ADR-0006 */ }
  #[test] fn dans_nomme_le_groupe_le_plus_proche() { let a = arbre_d_essai();
      assert_eq!(dans_de(&a, 16), Some("Options")); assert_eq!(dans_de(&a, 8), Some("Couleur")); assert_eq!(dans_de(&a, 4), None);
      /* un élément sous une List nommée « Lignes », elle-même sous un Group sans nom : Some("Lignes") */ }
  #[test] fn document_vide_regle() { /* [Window, Pane, Document sans enfant] : vrai ; [Window, TitleBar, Button sous la
      TitleBar] : vrai ; [Window, Button] : faux ; arbre_d_essai() : faux ; [Window, Pane, Document, Custom] : faux ;
      [Window, Pane, Document, Button (parent 2), Document sans enfant (parent 2)] : faux, le Document vide n'étant pas
      celui de la page ; [Window, Pane, Document sans enfant, Document (parent 1), Button (parent 3)] : vrai, le premier
      Document le plus extérieur étant vide */ }
  #[test] fn arret_uia_classe_les_codes() { let e = |code| ErreurUia { code, texte: "Invoke : 0x80004005".into() };
      for c in [UIA_DISPARU, RPC_DECONNECTE, OBJET_DECONNECTE] { assert_eq!(arret_uia(&e(c)), Arret::Introuvable("l'élément a disparu".into())); }
      assert_eq!(arret_uia(&e(UIA_DELAI)), Arret::Echec("échec du système : délai de l'automatisation dépassé".into()));
      assert_eq!(arret_uia(&e(UIA_INACTIF)), Arret::Echec("échec du système : l'élément est inactif".into()));
      for c in [UIA_NON_PRIS, UIA_OPERATION_INVALIDE] { assert_eq!(arret_uia(&e(c)), Arret::Echec("échec du système : l'élément refuse le geste".into())); }
      assert_eq!(arret_uia(&e(0x8000_4005)), Arret::Echec("échec du système : Invoke : 0x80004005".into())); }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- domaine::element domaine::sortie`.
- [ ] **Étape 3 : le code**, `element.rs` et `Issue::avis` ; `mod.rs` déclare `element`. `document_vide` suit la
  décision 6 ; `dans_de` cherche le plus proche ancêtre nommé de type `Group`, sinon le plus proche ancêtre nommé,
  ni l'un ni l'autre n'étant la racine, ni de type `Window` ou `Document`.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/element\.rs:'` et
  `-F '^src/domaine/sortie\.rs:'`.
- [ ] **Étape 5 : commit** : `git add src/domaine/element.rs src/domaine/sortie.rs src/domaine/mod.rs`, puis
  `git commit -m "feat(domaine): le modèle d'un élément de l'automatisation et sa ligne"`.

### Tâche 4 : le sélecteur et ses options

**Fichiers :** créer `src/domaine/selecteur.rs` ; modifier `src/domaine/cli.rs`, `src/domaine/mod.rs`, `Cargo.toml`,
`Cargo.lock`, ADR-0006.

**Interfaces :**
- Consomme : `ElementLu`, `type_de_nom`, `ancetres`, `essai::arbre_d_essai` (Tâche 3) ; `Selecteur`, `Arret`.
- Produit :
  ```rust
  /// NFC, puis U+00A0, U+202F et U+2007 en U+0020.
  pub fn normaliser(texte: &str) -> String;
  /// 64 : « type inconnu : <type> » ; si `exige`, « sélecteur vide : … » sans --nom, --contient, --type, --id,
  /// --classe ni --dans.
  pub fn analyser_le_selecteur(sel: &Selecteur, exige: bool) -> Result<(), Arret>;
  /// Vrai si l'élément `i` répond à toutes les options de `sel`, `--rang` mis à part.
  pub fn repond(arbre: &[ElementLu], i: usize, sel: &Selecteur) -> bool;
  /// Les indices qui répondent, W exclu, en ordre de document.
  pub fn candidats(arbre: &[ElementLu], sel: &Selecteur) -> Vec<usize>;
  /// Un seul élément : aucun (2), plusieurs sans --rang (2), le rang au-delà (2).
  pub fn choisir_un(arbre: &[ElementLu], sel: &Selecteur) -> Result<usize, Arret>;
  /// Les racines des sous-arbres de `arbre --dans <nom>` : les plus extérieures, hors W, Window et Document.
  pub fn sous_arbres(arbre: &[ElementLu], nom: &str) -> Vec<usize>;
  ```
  `Cargo.toml` : `unicode-normalization = "=0.1.26"` dans `[dependencies]`.

- [ ] **Étape 1 : les tests, rouges.** `sel("--nom Toutes --dans Forme")` construit un `Selecteur` des mots donnés,
  par `analyser(&a("pilote trouver --fenetre x --nom Toutes --dans Forme"))` : `a` découpe sur les espaces, si bien
  que `sel` n'admet aucune valeur à espace, et un sélecteur qui en porte une se construit à la main
  (`Selecteur { dans: Some("Appli d'essai".into()), ..Default::default() }`). Les cas marqués « couverture » passent
  dès leur écriture (clap les tient depuis la tranche 1) : ils gardent le comportement que la tranche livre, et ne se
  voient pas rouges.
  ```rust
  #[test] fn nom_compte_les_insecables_pour_des_espaces() { let mut a = arbre_d_essai();
      for n in ["Nom\u{a0}complet", "Nom\u{202f}complet", "Nom\u{2007}complet"] { a[4].nom = n.into();
          assert_eq!(candidats(&a, &Selecteur { nom: Some("Nom complet".into()), ..Default::default() }), vec![4], "{n:?}"); }
      a[4].nom = "Cafe\u{301}".into(); assert_eq!(candidats(&a, &Selecteur { nom: Some("Café".into()), ..Default::default() }), vec![4]); }
  #[test] fn nom_est_sensible_a_la_casse_et_contient_une_sous_chaine() { let a = arbre_d_essai();
      assert!(candidats(&a, &sel("--nom départ")).is_empty()); assert_eq!(candidats(&a, &sel("--contient part")), vec![3]); }
  #[test] fn type_sans_egard_a_la_casse() { assert_eq!(candidats(&arbre_d_essai(), &sel("--type radiobutton")), vec![8, 10, 12]); }
  #[test] fn id_et_classe_exacts() { /* --id champ → [4] ; --id Champ → [] ; une classe « btn » → --classe btn trouve, --classe bt non */ }
  #[test] fn dans_garde_tout_ancetre_hors_fenetre_et_document() { let mut a = arbre_d_essai();
      assert_eq!(candidats(&a, &sel("--dans Options")), vec![14, 15, 16]);
      let appli = Selecteur { dans: Some("Appli d'essai".into()), ..Default::default() };
      assert!(candidats(&a, &appli).is_empty());   // W et le Document portent ce nom : aucun ne compte
      a[0].nom = "Essai".into(); a[2].nom = "Essai".into(); assert!(candidats(&a, &sel("--dans Essai")).is_empty());
      a[1].nom = "Essai".into(); assert_eq!(candidats(&a, &sel("--dans Essai")), (2..=18).collect::<Vec<_>>()); }  // le témoin : le Pane compte
  #[test] fn candidats_excluent_la_racine() { assert_eq!(candidats(&arbre_d_essai(), &sel("--contient Appli")), vec![2]); }
  #[test] fn choisir_un_refuse_l_ambiguite_et_le_rang_au_dela() { let a = arbre_d_essai();
      assert_eq!(choisir_un(&a, &sel("--nom Toutes")), Err(Arret::Introuvable("plusieurs éléments répondent au sélecteur : 3 ; --rang départage".into())));
      assert_eq!(choisir_un(&a, &sel("--nom Toutes --rang 3")), Ok(12));
      assert_eq!(choisir_un(&a, &sel("--nom Toutes --rang 4")), Err(Arret::Introuvable("aucun élément au rang 4 : 3 candidats".into())));
      assert_eq!(choisir_un(&a, &sel("--nom Rien")), Err(Arret::Introuvable("aucun élément ne répond au sélecteur".into()))); }
  #[test] fn sous_arbres_les_plus_exterieurs() { assert_eq!(sous_arbres(&arbre_d_essai(), "Options"), vec![13]); }
  #[test] fn selecteur_vide_et_type_inconnu() {
      assert_eq!(analyser_le_selecteur(&Selecteur::default(), true), Err(Arret::Usage("sélecteur vide : --nom, --contient, --type, --id, --classe ou --dans".into())));
      assert_eq!(analyser_le_selecteur(&Selecteur::default(), false), Ok(()));
      assert_eq!(analyser_le_selecteur(&Selecteur { rang: Some(2), ..Default::default() }, true).is_err(), true);
      assert_eq!(analyser_le_selecteur(&sel("--type Bouton"), false), Err(Arret::Usage("type inconnu : Bouton".into()))); }
  // cli.rs
  #[test] fn rang_zero_sort_en_64() { let e = analyser(&a("pilote trouver --fenetre x --nom n --rang 0")).unwrap_err();
      assert_eq!((e.code, e.texte.as_str()), (Code::Usage, "valeur refusée : --rang <RANG>")); }
  #[test] fn options_de_texte_admettent_un_tiret() { /* couverture : « trouver --fenetre x --nom -a --contient -b --id -c --classe -d
      --dans -e » rend les cinq valeurs ; « valeur --fenetre x --id i --egale -z » rend egale Some("-z") ; « arbre --fenetre x
      --dans -g » rend dans Some("-g") ; « focus --fenetre x --titre -t » rend titre Some("-t") */ }
  #[test] fn clic_accepte_un_point_negatif() { assert!(matches!(analyser(&a("pilote clic --fenetre x --x -5 --y -3")).unwrap(),
      Commande::Clic { x: -5, y: -3, .. })); /* de même molette --x -1 --y -2 --crans 1 */ }
  // bornes_des_options gagne, en couverture : ("arbre --fenetre x --max 1", true), ("defiler --fenetre x --nom n --a 0", true),
  // ("defiler --fenetre x --nom n --a 100", true), ("trouver --fenetre x --nom n --rang 1", true).
  #[test] fn aide_des_crans_dit_le_sens() { /* l'aide de `molette` (`--help`) porte « positifs vers le haut » */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- domaine::selecteur domaine::cli`.
- [ ] **Étape 3 : le code.** `selecteur.rs`, comme la décision 7 le dit ; `cli.rs` : `rang` par
  `value_parser!(u32).range(1..)` ; `--x` et `--y` de `clic` et `molette` en `allow_negative_numbers = true`, leur aide
  « L'abscisse, en pixels physiques depuis le coin haut gauche de la zone cliente. » (et l'ordonnée de même) ; l'aide
  de `--crans` : « Les crans, de -50 à 50, sauf 0 ; positifs vers le haut. » ; « (décision N) » remplacé dans ce
  fichier s'il en reste.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/selecteur\.rs:'` et
  `-F '^src/domaine/cli\.rs:'`.
- [ ] **Étape 5 : ADR-0006**, premier amendement : la ligne « Amendé » ; « Le sélecteur » dit la décision 7 au
  présent ; « Les bornes » admet le nombre négatif de `--x` et `--y`, dit leur repère (des pixels physiques depuis le
  coin haut gauche de la zone cliente) et le sens de `--crans` (positif vers le haut) ; « Conséquences » : « Un changement amende cet
  ADR en place (`docs/modele/documentation.md`), avec la compétence dans la même PR dès qu'elle existe » ; la section
  `## Confirmation` naît, avec `src/domaine/selecteur.rs` et `src/domaine/cli.rs` ; la sous-section de la tranche 2
  naît.
- [ ] **Étape 6 : commit** : `git add src/domaine/selecteur.rs src/domaine/cli.rs src/domaine/mod.rs Cargo.toml Cargo.lock docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`,
  puis `git commit -m "feat(domaine): le sélecteur, NFC et espaces insécables, et ses options"`.

### Tâche 5 : le trait `Bureau` étendu, le faux bureau et les aides des tests

**Fichiers :** modifier `src/domaine/bureau.rs`, `src/domaine/faux.rs`, `src/domaine/gestes.rs` (le module `essai`,
Verr. Maj illisible), `src/bureau/mod.rs` et `src/bureau/clavier.rs` (des réalisations provisoires), ADR-0005,
ADR-0006.

**Interfaces :**
- Consomme : `ElementLu`, `ErreurUia` (Tâche 3) ; `IdFenetre`, `PremierPlan`, `Duree`, `ErreurSysteme`.
- Produit :
  ```rust
  // bureau.rs
  pub use crate::domaine::element::{ElementLu, ErreurUia};
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Rect { pub x: i32, pub y: i32, pub largeur: u32, pub hauteur: u32 }
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Point { pub x: i32, pub y: i32 }
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct DelaisUia { pub connexion: Duree, pub transaction: Duree }
  /// Le focus du système : son processus, son marquage, et son indice dans la dernière lecture de l'arbre (`None`
  /// s'il n'en est pas un élément).
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct FocusLu { pub pid: u32, pub motdepasse: bool, pub indice: Option<usize> }
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub struct EtatLu { pub coche: Option<i32>, pub choisi: Option<bool> }
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum GesteUia { Focaliser, Invoquer, Basculer, Choisir, Defiler(u8) }
  /// Une capture : l'image de `fenetre` (le rectangle de la fenêtre à l'écran), en BGRA de haut en bas, le cadre
  /// de DWM et la zone cliente à l'écran, s'ils se lisent.
  #[derive(Clone, Debug, PartialEq)] pub struct Image { pub largeur: u32, pub hauteur: u32, pub bgra: Vec<u8>,
      pub fenetre: Rect, pub cadre: Option<Rect>, pub client: Option<Rect> }
  pub trait Bureau: Disposition { /* … les 20 méthodes de la tranche 1, `verr_maj` changée : */
      /// Verr. Maj allumée, ou `None` si son état ne se lit pas : le domaine refuse alors.
      fn verr_maj(&mut self) -> Option<bool>;
      /* puis quatorze méthodes nouvelles : */
      fn ouvrir_l_automatisation(&mut self, delais: DelaisUia) -> Result<(), ErreurUia>;
      fn lire_l_arbre(&mut self, f: IdFenetre) -> Result<Vec<ElementLu>, ErreurUia>;
      fn lire_le_focus(&mut self) -> Result<Option<FocusLu>, ErreurUia>;
      fn lire_la_valeur(&mut self, i: usize) -> Result<String, ErreurUia>;
      fn relire_l_etat(&mut self, i: usize) -> Result<EtatLu, ErreurUia>;
      fn agir(&mut self, i: usize, geste: GesteUia) -> Result<(), ErreurUia>;
      fn repond(&mut self, f: IdFenetre, delai: Duree) -> bool;
      fn capturer(&mut self, f: IdFenetre) -> Result<Image, ErreurSysteme>;
      fn ecrire_fichier(&mut self, chemin: &Path, octets: &[u8]) -> Result<(), ErreurSysteme>;
      fn zone_cliente(&mut self, f: IdFenetre) -> Option<Rect>;           // à l'écran
      fn fenetre_sous(&mut self, p: Point) -> Option<PremierPlan>;      // GA_ROOT, son PID, ses propriétaires
      fn bureau_virtuel(&mut self) -> Rect;
      fn boutons_inverses(&mut self) -> bool;
      /// Vrai si le fil de `pilote` est sensible au DPI de chaque écran (PMv2) : les coordonnées lues sont alors
      /// des pixels physiques.
      fn sensible_au_dpi(&mut self) -> bool;
  }
  // faux.rs : AppelFaux gagne
  OuvrirUia(DelaisUia), LireArbre(IdFenetre), LireFocus, LireValeur(usize), RelireEtat(usize), Agir(usize, GesteUia),
  Repond(IdFenetre, Duree), Capturer(IdFenetre), Ecrire(PathBuf), ZoneCliente(IdFenetre), FenetreSous(Point), BureauVirtuel,
  BoutonsInverses, SensibleAuDpi
  // gestes.rs
  #[cfg(test)] pub(crate) mod essai { pub(crate) fn x(b: &mut FauxBureau, s: &str) -> Issue;
      pub(crate) fn aucun_geste(b: &FauxBureau) -> bool; pub(crate) fn gestes(b: &FauxBureau) -> Vec<AppelFaux>;
      pub(crate) fn appele(b: &FauxBureau, quoi: fn(&AppelFaux) -> bool) -> bool;
      pub(crate) fn compte(b: &FauxBureau, appel: &AppelFaux) -> usize; pub(crate) fn ecoule(b: &FauxBureau) -> u64;
      /// Vrai si chaque `Envoi` et chaque `Agir` suit exactement `[PremierPlan, Processus, PremierPlan]`, et s'il y en
      /// a au moins un : pour un geste sans activation ni insertion partielle, le premier plan déjà accepté.
      pub(crate) fn premier_plan_juste_avant(b: &FauxBureau) -> bool; }
  ```
  `aucun_geste` compte aussi `Agir` parmi les gestes. Les tests de `gestes.rs` les importent de `essai`, et ne les
  redéfinissent plus ; `premier_plan_verifie_avant_chaque_unite` s'écrit par `premier_plan_juste_avant`.
  `FauxBureau` gagne, par champ public : `arbres: VecDeque<Vec<ElementLu>>`, `focus: VecDeque<Result<Option<FocusLu>,
  ErreurUia>>`, `valeurs: BTreeMap<usize, VecDeque<String>>`, `etats: VecDeque<EtatLu>` et `zones:
  VecDeque<Option<Rect>>` (chaque lecture prend la tête tant qu'il en reste plus d'une, puis rend la dernière, à
  chaque fois) ; `erreur_ouverture`, `erreur_arbre`, `erreur_valeur`, `erreur_etat`, `erreur_agir: Option<ErreurUia>` ;
  `repond: bool` ; `image: Result<Image, ErreurSysteme>` ; `ecrits: Vec<(PathBuf, Vec<u8>)>` et `ecriture_ratee: bool` ;
  `sous_le_point: Option<PremierPlan>` ; `virtuel: Rect` ; `inverses: bool` ; `dpi_par_ecran: bool` ; `niveaux:
  Vec<(u32, Option<u32>)>` (le niveau d'un PID, consulté avant `niveau_cible`) ; les vols, chacun le premier plan qui
  passe à cette fenêtre juste après le n-ième appel : `vol_apres_l_arbre`, `vol_apres_le_focus`, `vol_apres_agir:
  Option<(usize, PremierPlan)>` ; `uia_ouverte: bool`. `verr_maj` devient `Option<bool>` (`Some(false)` par défaut ;
  les tests de la tranche 1 qui posaient `true` posent `Some(true)`). `avec_appli()` : `arbres` [`arbre_d_essai()`],
  `focus` [`Ok(Some(FocusLu { pid: 100, motdepasse: false, indice: Some(4) }))`], `valeurs` {4 : ["bonjour"]},
  `repond` vrai, `image` `Ok(image_d_essai())`, `zones` [`Some(Rect { x: 100, y: 200, largeur: 800, hauteur: 600 })`],
  `sous_le_point` le premier plan de W, `virtuel` `Rect { x: 0, y: 0, largeur: 1920, hauteur: 1080 }`, `dpi_par_ecran`
  vrai, le reste vide, faux ou absent. `image_d_essai()` : 10 × 8, chaque pixel `[10, 20, 30, 0]` sauf (2, 3) `[0, 0,
  255, 0]`, `fenetre` `Rect { x: 100, y: 100, largeur: 10, hauteur: 8 }`, `cadre` `Some(Rect { x: 101, y: 100, largeur:
  8, hauteur: 7 })`, `client` `Some(Rect { x: 102, y: 102, largeur: 6, hauteur: 4 })`. Chaque lecture de
  l'automatisation (`lire_l_arbre`, `lire_le_focus`, `lire_la_valeur`, `relire_l_etat`, `agir`) panique si
  `ouvrir_l_automatisation` n'a pas réussi avant elle. `agir` enregistre son appel, puis rend `erreur_agir`.
  `BureauWindows` réalise chaque méthode nouvelle par un provisoire qui rend l'échec (`ErreurUia { code: 0x8000_4001,
  texte: "pas encore réalisé" }`, `ErreurSysteme`, `false`, `None`, un `Rect` nul), et `verr_maj` par `Some` de la
  lecture d'aujourd'hui : la Tâche 11 les remplace tous.
  Dans `gestes.rs`, la position 8 refuse un état illisible (1, « Verr. Maj illisible ») comme une Verr. Maj allumée.

- [ ] **Étape 1 : les tests, rouges.**
  ```rust
  // faux.rs
  #[test] fn lecture_avant_l_ouverture_panique() { let r = std::panic::catch_unwind(|| {
      let mut b = FauxBureau::avec_appli(); let _ = b.lire_l_arbre(IdFenetre(1)); }); assert!(r.is_err()); }
  #[test] fn arbres_successifs_puis_le_dernier() { /* arbres [a, b] : trois lectures rendent a, b, b ; trois LireArbre */ }
  #[test] fn etats_et_zones_successifs_puis_le_dernier() { /* etats [x, y] : relire rend x, y, y ; zones [Some(z1),
      Some(z2)] : zone_cliente rend z1, z2, z2 */ }
  #[test] fn ecrire_garde_le_fichier_ou_echoue() { /* ecrire c.png [1, 2] : ecrits == [("c.png", [1, 2])] ; ecriture_ratee :
      Err, rien d'écrit */ }
  #[test] fn agir_note_puis_echoue() { /* erreur_agir Some(e) : agir(3, Invoquer) rend Err(e), et Agir(3, Invoquer) est noté */ }
  #[test] fn vols_apres_l_arbre_le_focus_et_agir() { /* vol_apres_l_arbre (2, q) : le premier plan est W après la 1re
      lecture de l'arbre, q après la 2e ; de même vol_apres_le_focus (1, q) après la 1re lecture du focus, et
      vol_apres_agir (1, q) après le 1er agir, même en erreur */ }
  #[test] fn niveau_d_un_pid_avant_niveau_cible() { /* niveaux [(101, Some(0x3000))] : niveau_integrite(Some(101)) rend
      Some(0x3000), niveau_integrite(Some(100)) rend niveau_cible ; Niveau(Some(101)) noté */ }
  // gestes.rs
  #[test] fn verr_maj_illisible_refuse() { /* verr_maj None : « taper --fenetre appli-essai --texte a --comme-au-clavier »
      → Refus « Verr. Maj illisible », aucun geste ; sans --comme-au-clavier, Fait */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- domaine::faux domaine::gestes`.
- [ ] **Étape 3 : le code** : le trait, les types, le faux, le module `essai` de `gestes.rs` et Verr. Maj illisible,
  les provisoires de `BureauWindows` ; « (décision N) » remplacé dans `bureau.rs` s'il en reste. `cargo build -p
  pilote` passe sous Windows.
- [ ] **Étape 4 : voir vert** : `cargo test --workspace`, puis `cargo mutants -p pilote -F '^src/domaine/gestes\.rs:'`.
- [ ] **Étape 5 : les ADR.** ADR-0005, « Les gestes et l'ordre des gardes », position 8 : Verr. Maj allumée ou
  illisible (1). ADR-0006 : « Verr. Maj illisible » dans « Les messages », et au 1 de « Les codes de sortie ».
  « Confirmation » nomme `verr_maj_illisible_refuse` ; la ligne « Amendé » et les sous-sections reçoivent ce point.
- [ ] **Étape 6 : commit** : `git add src/domaine/bureau.rs src/domaine/faux.rs src/domaine/gestes.rs src/bureau/mod.rs src/bureau/clavier.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`
  (et `.cargo/mutants.toml` s'il change), puis
  `git commit -m "feat(domaine): le trait Bureau étendu à l'automatisation, la capture et le pointeur"`.

### Tâche 6 : les lectures — focus, arbre, trouver, valeur et defilables

**Fichiers :** créer `src/domaine/lectures.rs` ; modifier `src/domaine/gestes.rs` (`Travail`, `Suivi`, `sonder`,
`lire_fenetres_et_processus`, `echec`, `echec_systeme`, les deux essais de « pas encore livrée »),
`src/domaine/mod.rs`, ADR-0005, ADR-0006.

**Interfaces :**
- Consomme : Tâches 3 à 5, dont les aides de `gestes::essai` ; `retenir`, `premier_plan_accepte`, `sonder`, `Suivi`,
  `lire_fenetres_et_processus`, `echec`, `echec_systeme`, que cette tâche passe en `pub(crate)`.
- Produit :
  ```rust
  // lectures.rs
  pub const SONDE: Duree = Duree(100);
  pub const DOCUMENT_VIDE_MAX: Duree = Duree(2_000);
  pub const REPONSE_MAX: Duree = Duree(2_000);
  pub const DELAIS: DelaisUia = DelaisUia { connexion: Duree(2_000), transaction: Duree(5_000) };
  pub enum Lecture { Focus { cible: Cible, selecteur: Selecteur, attendre: Option<u32> },
      Arbre { cible: Cible, max: u32, dans: Option<String>, attendre: Option<u32> },
      Trouver { cible: Cible, selecteur: Selecteur, attendre: Option<u32> },
      Valeur { cible: Cible, selecteur: Selecteur, egale: Option<String>, attendre: Option<u32> }, Defilables(Cible) }
  pub(crate) fn lire(l: &Lecture, s: &mut Suivi, b: &mut dyn Bureau) -> Result<Issue, Arret>;
  /// La réponse de W (3), puis l'ouverture de l'automatisation avec `DELAIS`.
  pub(crate) fn ouvrir(w: IdFenetre, b: &mut dyn Bureau) -> Result<(), Arret>;
  /// L'arbre de `f`, relu toutes les `SONDE` tant qu'il est vide et que `fin` n'est pas passée, puis tel quel.
  pub(crate) fn lire_l_arbre(f: IdFenetre, fin: Moment, b: &mut dyn Bureau) -> Result<Vec<ElementLu>, Arret>;
  /// `essai` tout de suite, puis toutes les `SONDE` tant que `fin` n'est pas passée : `Err` arrête, `Ok(Err(a))` se
  /// rejoue, et sort avec `a` à `fin` ; `Ok(Ok(v))` rend `v`.
  pub(crate) fn attendre_que<T>(fin: Moment, b: &mut dyn Bureau,
      essai: impl FnMut(&mut dyn Bureau) -> Result<Result<T, Arret>, Arret>) -> Result<T, Arret>;
  /// La ligne de l'élément `i`, sa valeur lue s'il offre Value et n'est pas un mot de passe.
  pub(crate) fn ligne(arbre: &[ElementLu], i: usize, dans: bool, rang: Option<usize>, b: &mut dyn Bureau) -> Result<String, Arret>;
  // gestes.rs
  enum Travail { Lister, Geste { … }, Lecture(Lecture), PasEncoreLivree(&'static str) }
  pub(crate) struct Suivi { /* les champs de la tranche 1, pub(crate) */ }
  pub(crate) fn sonder<T>(…);  pub(crate) fn premier_plan_accepte(…);
  ```
  L'analyse (`analyser_le_travail`) juge, avant le journal : `focus --rang` (64) ; `analyser_le_selecteur(sel,
  false)` pour `focus` et `trouver`, `(sel, true)` pour `valeur`. Chaque lecture prend `debut = b.maintenant()` juste
  avant sa première lecture de l'arbre ; la sonde se borne à `debut + max(DOCUMENT_VIDE_MAX, --attendre)`, les
  reprises à `debut + --attendre` (aucune sans `--attendre`), et chaque reprise relit l'arbre par `lire_l_arbre`,
  bornée à la même fin que la sonde (décision 6).

- [ ] **Étape 1 : les tests, rouges.** `vide()` rend `[Window « Appli d'essai », Pane sans nom, Document sans nom ni
  enfant]`.
  ```rust
  #[test] fn arbre_rend_les_lignes_indentees() { let mut b = FauxBureau::avec_appli(); let i = x(&mut b, "arbre --fenetre appli-essai");
      assert_eq!(i.code, Code::Fait); assert_eq!(i.sortie.len(), 19); assert_eq!(i.sortie[0], "[Window] <Appli d'essai>");
      assert_eq!(i.sortie[1], "  [Pane]"); assert_eq!(i.sortie[3], "      [Button] <Départ> id=depart");
      assert_eq!(i.sortie[4], "      [Edit] <Champ> id=champ valeur=bonjour");
      assert_eq!(i.sortie[5], "      [Edit] <Mot de passe> id=motdepasse motdepasse=oui");
      assert_eq!(i.sortie[16], "          [CheckBox] <Option interne> id=option-interne etat=decoche"); assert_eq!(i.erreur, None); }
  #[test] fn arbre_ne_lit_jamais_la_valeur_d_un_mot_de_passe() { /* après arbre : LireValeur(4) noté, LireValeur(5) jamais */ }
  #[test] fn arbre_coupe_le_dit_sur_la_sortie_d_erreur() { /* « arbre --max 3 » : Fait, 3 lignes, erreur Some("arbre coupé : 3
      lignes sur 19") ; aucune LireValeur au-delà de la 3e ligne */ }
  #[test] fn arbre_dans_un_groupe() { /* « arbre --dans Options » : sortie == ["[Group] <Options>", "  [CheckBox] <Option
      externe> id=option-externe etat=decoche", "  [Group] <Options>", "    [CheckBox] <Option interne> id=option-interne
      etat=decoche"] ; « arbre --dans Rien » : Introuvable « aucun élément « Rien » pour --dans » */ }
  #[test] fn lecture_sonde_un_document_vide() { /* arbres [vide(), vide(), arbre_d_essai()] : arbre en 0, 19 lignes,
      LireArbre(IdFenetre(1)) trois fois, ecoule(&b) == 200 */ }
  #[test] fn document_vide_rend_ce_qu_il_a_apres_2_s() { /* arbres [vide()] : Fait, sortie ["[Window] <Appli d'essai>",
      "  [Pane]", "    [Document]"], 21 LireArbre, ecoule == 2000 ; avec --attendre 3000 : 31 LireArbre, ecoule == 3000 */ }
  #[test] fn trouver_sur_un_document_vide_tient_sa_borne() { /* arbres [vide()] : « trouver --nom Départ --attendre 3000 »
      → Introuvable « aucun élément ne répond au sélecteur », ecoule == 3000, 31 LireArbre ; « trouver --nom Départ »
      → le même arrêt, ecoule == 2000 ; « valeur --id champ --attendre 500 » → ecoule == 2000, la sonde décidant */ }
  #[test] fn hote_qui_ne_repond_pas_sort_en_3() { /* repond faux : arbre → Echec « échec du système : la fenêtre ne répond
      pas » ; Repond(IdFenetre(1), Duree(2000)) noté ; ni OuvrirUia ni LireArbre */ }
  #[test] fn ouverture_porte_les_delais() { /* après arbre : OuvrirUia(DelaisUia { connexion: Duree(2000), transaction:
      Duree(5000) }) noté une fois, avant LireArbre */ }
  #[test] fn delai_et_erreur_de_l_automatisation() { /* erreur_arbre code UIA_DELAI → Echec « échec du système : délai de
      l'automatisation dépassé » ; code 0x80004005, texte « ElementFromHandleBuildCache : E_FAIL » → Echec « échec du
      système : ElementFromHandleBuildCache : E_FAIL » ; erreur_ouverture de même */ }
  #[test] fn lectures_n_activent_rien() { for s in ["focus", "arbre", "trouver --nom Départ", "valeur --id champ", "defilables"] {
      let mut b = FauxBureau::avec_appli(); x(&mut b, &format!("{s} --fenetre appli-essai"));
      assert!(aucun_geste(&b), "{s}"); assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_) | AppelFaux::EtatSession
          | AppelFaux::Niveau(_) | AppelFaux::TouchesEnfoncees | AppelFaux::Agir(..))), "{s}"); } }
  #[test] fn focus_rend_l_element_focalise() { let mut b = FauxBureau::avec_appli();
      assert_eq!(x(&mut b, "focus --fenetre appli-essai"), Issue::fait(vec!["[Edit] <Champ> id=champ valeur=bonjour".into()])); }
  #[test] fn focus_hors_de_la_fenetre_visee_sort_en_1() { /* quatre cas, chacun Refus « focus hors de la fenêtre visée »,
      sortie vide : le premier plan à la fenêtre 9 d'un processus 300 (ni OuvrirUia, ni LireArbre, ni LireFocus) ; le
      focus d'un PID 999 hors de l'arbre retenu ; le focus d'indice None ; et focus Ok(None) */ }
  #[test] fn focus_lit_l_arbre_du_dialogue_accepte() { /* premier plan : fenêtre 2 de PID 100, propriétaires [1] :
      LireArbre(IdFenetre(2)) */ }
  #[test] fn focus_avec_selecteur_et_attente() { /* « focus --id depart » : Introuvable « l'élément focalisé ne répond pas
      au sélecteur » ; focus [indice 4, 4, 4, indice 3] et --attendre 1000 : Fait ["[Button] <Départ> id=depart"],
      ecoule == 300 ; focus toujours 4 et --attendre 300 : même Introuvable, ecoule == 300 */ }
  #[test] fn focus_refuse_le_rang() { /* « focus --rang 1 » : Usage « options incompatibles : --rang <RANG> », journal fermé compris */ }
  #[test] fn trouver_liste_les_candidats_avec_leur_rang() { let mut b = FauxBureau::avec_appli();
      assert_eq!(x(&mut b, "trouver --fenetre appli-essai --nom Toutes").sortie, vec![
          "[RadioButton] <Toutes> id=couleur-toutes etat=choisi dans=Couleur rang=1",
          "[RadioButton] <Toutes> id=taille-toutes etat=choisi dans=Taille rang=2",
          "[RadioButton] <Toutes> id=forme-toutes etat=choisi dans=Forme rang=3"]); }
  #[test] fn trouver_rang_rien_et_tout() { /* --rang 2 : la seule ligne de taille-toutes, rang=2 ; --rang 4 : Introuvable
      « aucun élément au rang 4 : 3 candidats » ; --nom Rien : « aucun élément ne répond au sélecteur » ; trouver sans
      option : 18 lignes */ }
  #[test] fn trouver_attend_un_element_qui_arrive() { /* arbres [a, a, a, a + Button « Arrivé »] et --attendre 1000 :
      Fait, une ligne "[Button] <Arrivé> rang=1", ecoule == 300 */ }
  #[test] fn ambiguite_ne_s_attend_pas() { /* « valeur --nom Toutes --attendre 1000 » : Introuvable « plusieurs éléments
      répondent au sélecteur : 3 ; --rang départage », un seul LireArbre, ecoule == 0 */ }
  #[test] fn valeur_rend_la_ligne_de_l_element() { /* « valeur --id champ » : ["[Edit] <Champ> id=champ valeur=bonjour"] */ }
  #[test] fn valeur_refuse_un_mot_de_passe_meme_avec_egale() { for s in ["valeur --id motdepasse", "valeur --id motdepasse
      --egale x"] { /* Refus « valeur non rendue : champ de mot de passe », jamais LireValeur(5) */ } }
  #[test] fn valeur_sans_motif_sort_en_3() { /* « valeur --id depart » : Echec « échec du système : l'élément n'offre pas le
      motif Value » */ }
  #[test] fn valeur_egale() { /* --egale bonjour : Fait, la ligne ; --egale autre : Introuvable « la valeur diffère de
      « autre » » ; valeurs {4 : ["", "", "bonjour"]} et « --egale bonjour --attendre 1000 » : Fait, ecoule == 200 ;
      valeurs {4 : ["e\u{301}"]} et --egale é : Fait */ }
  #[test] fn selecteur_vide_sort_en_64() { /* « valeur --fenetre appli-essai » : Usage « sélecteur vide : --nom,
      --contient, --type, --id, --classe ou --dans » ; « trouver --type Bouton » : Usage « type inconnu : Bouton » ; aucune
      lecture de fenêtre */ }
  #[test] fn defilables_liste_ce_qui_defile() { /* ["[Group] <Cadre des lignes> id=cadre"] ; un arbre sans `defile` : Fait,
      sortie vide */ }
  #[test] fn lecture_ecrit_sa_ligne_au_journal() { /* arbre : une ligne « arbre », processus appli-essai, code 0 */ }
  ```
  `sous_commande_pas_encore_livree_sort_en_64` et `pas_encore_livree_exige_le_journal` passent de `focus --fenetre x`
  à `fermer --fenetre x` (« pas encore livrée : fermer »).
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- domaine::lectures domaine::gestes`.
- [ ] **Étape 3 : le code.** `lectures.rs` suit les décisions 5, 6 et 8 ; `Travail::Lecture` porte les cinq
  sous-commandes ; `Suivi`, `sonder`, `premier_plan_accepte`, `lire_fenetres_et_processus`, `echec` et
  `echec_systeme` passent en `pub(crate)` ; « (décision N) » remplacé dans `gestes.rs` s'il en reste.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/lectures\.rs:'` et
  `-F '^src/domaine/gestes\.rs:'`.
- [ ] **Étape 5 : les ADR.** ADR-0005, « Ce qu'on lit reste dans la fenêtre retenue » : la décision 5, et la lecture
  ratée de `IsPassword` qui fait échouer la lecture (décision 4). ADR-0006 : « La sortie », la décision 8 (sauf la
  capture) ; « L'attente », la décision 6, l'échéance unique et le document de la page ; « L'analyse », « pas
  encore livrée » réservé à la tranche 3 ; « Les codes de sortie », ces lignes : au 2, la valeur qui diffère, avec ou
  sans `--attendre`, l'élément disparu, aucun élément pour `arbre --dans` ; au 3, la fenêtre qui ne répond pas, le
  délai de l'automatisation, l'élément inactif ou qui refuse le geste ; au 64, le type inconnu, le sélecteur vide,
  `focus --rang` ; « Les messages », les lignes de la table de ce plan pour les lectures, `valeur non rendue :
  champ de mot de passe` compris. Les deux « Confirmation » nomment `src/domaine/lectures.rs` ; la ligne « Amendé »
  et les sous-sections reçoivent ces points.
- [ ] **Étape 6 : commit** : `git add src/domaine/lectures.rs src/domaine/gestes.rs src/domaine/mod.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`
  (et `.cargo/mutants.toml` s'il change), puis `git commit -m "feat(domaine): focus, arbre, trouver, valeur et defilables"`.

### Tâche 7 : la capture

**Fichiers :** créer `src/domaine/capture.rs` ; modifier `src/domaine/gestes.rs` (`Travail::Capture`),
`src/domaine/mod.rs`, `Cargo.toml`, `Cargo.lock`, ADR-0005, ADR-0006.

**Interfaces :**
- Consomme : `Image`, `Rect`, `sensible_au_dpi` (Tâche 5) ; `lectures::REPONSE_MAX` (Tâche 6) ; `keepassxc_dans`,
  `retenir`, `Suivi`.
- Produit :
  ```rust
  /// L'image rognée, en RGB, et la zone cliente relative à elle.
  #[derive(Clone, Debug, PartialEq)] pub struct Rogne { pub largeur: u32, pub hauteur: u32, pub rgb: Vec<u8>, pub zone: Option<Rect> }
  pub fn rogner(image: &Image) -> Result<Rogne, Arret>;    // 3 « capture vide » ; le cadre ∩ la fenêtre, ou l'image entière
  pub fn toute_noire(r: &Rogne) -> bool;
  pub fn png(r: &Rogne) -> Result<Vec<u8>, Arret>;         // RGB, 8 bits, Compression::Fast
  pub(crate) fn capturer(cible: &Cible, sortie: &Path, s: &mut Suivi, b: &mut dyn Bureau) -> Result<Issue, Arret>;
  ```
  `Cargo.toml` : `png = "=0.18.1"` dans `[dependencies]`.

- [ ] **Étape 1 : les tests, rouges.**
  ```rust
  #[test] fn rognage_garde_le_cadre_dwm() { let r = rogner(&image_d_essai()).unwrap();
      assert_eq!((r.largeur, r.hauteur, r.rgb.len()), (8, 7, 8 * 7 * 3));
      assert_eq!(r.zone, Some(Rect { x: 1, y: 2, largeur: 6, hauteur: 4 }));
      assert_eq!(&r.rgb[(3 * 8 + 1) * 3..(3 * 8 + 1) * 3 + 3], &[255, 0, 0]); }   // le pixel (2, 3) de l'image, en RGB
  #[test] fn sans_cadre_l_image_entiere() { /* cadre None, client None : 10 × 8, zone None */ }
  #[test] fn capture_vide_sort_en_3() { /* largeur 0, ou un cadre hors de la fenêtre : Echec « échec du système : capture vide » */ }
  #[test] fn png_sans_alpha() { let p = png(&rogner(&image_d_essai()).unwrap()).unwrap();
      assert_eq!(&p[..8], b"\x89PNG\r\n\x1a\n"); assert_eq!(&p[16..24], &[0, 0, 0, 8, 0, 0, 0, 7]); assert_eq!((p[24], p[25]), (8, 2)); }
  #[test] fn capture_ecrit_le_png_et_dit_sa_ligne() { let mut b = FauxBureau::avec_appli();
      let i = x(&mut b, "capture --fenetre appli-essai --sortie c.png");
      assert_eq!(i, Issue::fait(vec!["[Capture] <c.png> largeur=8 hauteur=7 zone-cliente=1,2,6,4".into()]));
      assert_eq!(b.ecrits.len(), 1); assert_eq!(b.ecrits[0].0, PathBuf::from("c.png")); assert!(aucun_geste(&b));
      assert!(!appele(&b, |a| matches!(a, AppelFaux::PrendreVerrou(_) | AppelFaux::EtatSession | AppelFaux::Niveau(_)
          | AppelFaux::TouchesEnfoncees))); }   // une capture ne prend ni le verrou, ni le poste, ni le niveau
  #[test] fn capture_sans_dpi_par_ecran_sort_en_3() { /* dpi_par_ecran faux : Echec « échec du système : pilote n'est
      pas sensible au DPI », SensibleAuDpi noté après EstReduite, ni Repond, ni Capturer, ni Ecrire */ }
  #[test] fn capture_refuse_une_fenetre_reduite() { /* la fenêtre 1 réduite : Refus « capture refusée : fenêtre réduite »,
      ni Repond, ni Capturer, ni Ecrire */ }
  #[test] fn keepassxc_par_explorer_refuse_la_capture() { /* avec_keepassxc_par_explorer(), « capture --fenetre explorer
      --sortie c.png » : Refus « capture refusée : fenêtre de KeePassXC », ni Capturer ni Ecrire ; de même pour une fenêtre
      enfant de W de KeePassXC */ }
  #[test] fn capture_d_un_hote_qui_ne_repond_pas_sort_en_3() { /* repond faux : Echec « échec du système : la fenêtre ne
      répond pas », ni Capturer ni Ecrire */ }
  #[test] fn capture_noire_sort_en_3() { /* chaque pixel [0, 0, 0, 255] : Echec « échec du système : capture toute noire »,
      rien d'écrit */ }
  #[test] fn capture_dans_un_dossier_absent_sort_en_64() { /* ecriture_ratee : Usage « fichier non écrit : c.png » */ }
  #[test] fn capture_remplace_un_fichier_existant() { /* deux captures du même chemin : deux Ecrire, l'issue Fait à chaque fois */ }
  #[test] fn capture_echouee_dit_le_systeme() { /* image Err(ErreurSysteme("PrintWindow")) : Echec « échec du système :
      PrintWindow » */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib domaine::capture`.
- [ ] **Étape 3 : le code**, dans l'ordre de la décision 5 pour `capture`, puis la décision 10.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/capture\.rs:'`.
- [ ] **Étape 5 : les ADR.** ADR-0005, « Les captures » : la sensibilité au DPI exigée, le rognage, les jugements,
  l'écriture après eux, à la place d'un fichier existant, KeePassXC et la fenêtre réduite. ADR-0006 : la ligne
  `[Capture]` dans « La sortie », et `largeur`, `hauteur` et `zone-cliente` dans la liste des clés ; « Les codes de
  sortie », ces lignes : au 1, la capture d'une fenêtre de KeePassXC ; au 3, la capture vide et `pilote` qui n'est
  pas sensible au DPI ; au 64, le fichier non écrit ; « Les messages », ceux de la capture. « Confirmation » et les
  sous-sections reçoivent ces points.
- [ ] **Étape 6 : commit** : `git add src/domaine/capture.rs src/domaine/gestes.rs src/domaine/mod.rs Cargo.toml Cargo.lock docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`,
  puis `git commit -m "feat(domaine): capture de la fenêtre retenue en PNG"`.

### Tâche 8 : la garde d'une unité, la règle du champ de KeePassXC et le niveau des dialogues

**Fichiers :** modifier `src/domaine/gestes.rs`, ADR-0005, ADR-0006.

**Interfaces :**
- Consomme : `lire_le_focus`, `ouvrir_l_automatisation`, `niveaux`, `vol_apres_le_focus`, `gestes::essai` (Tâche 5) ;
  `lectures::{ouvrir, lire_l_arbre, DOCUMENT_VIDE_MAX, DELAIS}` (Tâche 6) ; `keepassxc_dans`, `premier_plan_accepte`,
  `juger_le_niveau`, `accepte`.
- Produit :
  ```rust
  pub const CADENCE_KEEPASSXC: Duree = Duree(100);
  /// Le niveau du processus `pid` comparé à celui de `pilote`, s'il n'est pas déjà dans `juges` (P y est d'emblée).
  pub(crate) fn juger_le_niveau_de(pid: u32, juges: &mut BTreeSet<u32>, b: &mut dyn Bureau) -> Result<(), Arret>;
  /// Le premier plan accepté pour `r`, avec l'instantané des processus qu'il a lu : `premier_plan_accepte` le
  /// rend sans l'instantané.
  pub(crate) fn premier_plan_et_instantane(r: &Retenue, b: &mut dyn Bureau) -> Result<Option<(PremierPlan, Vec<Processus>)>, Arret>;
  /// Ce que la garde d'une unité garde d'une unité à l'autre.
  pub(crate) struct Garde { pub r: Retenue, pub juges: BTreeSet<u32>, pub keepassxc: bool,
      pub arbre_de: Option<IdFenetre>, pub arbre: Vec<ElementLu> }
  /// L'issue de la garde : l'unité part (le focus de KeePassXC dans un champ de mot de passe, ou non) ; le premier
  /// plan n'est plus accepté, ou a changé entre ses deux lectures ; le focus de KeePassXC illisible.
  pub(crate) enum Avant { Part { motdepasse: bool }, Change, FocusIllisible }
  /// La garde d'une unité (décision 13) : (a) le premier plan ; (b) `entre` (les lectures propres au geste, qui
  /// rendent le PID d'une autre fenêtre à juger), le niveau des PID pas encore jugés, l'arbre et le focus de
  /// KeePassXC ; (c) le premier plan relu, égal, si (b) a lu quoi que ce soit. Rien ne se lit après (c).
  pub(crate) fn garder_l_unite(g: &mut Garde, b: &mut dyn Bureau,
      entre: impl FnMut(&PremierPlan, &[Processus], &mut dyn Bureau) -> Result<Option<u32>, Arret>) -> Result<Avant, Arret>;
  ```
  `refuser_keepassxc` et le message `KEEPASSXC` disparaissent ; à leur place, le texte refusé (positions 3 et 10),
  puis, si KeePassXC se lit à la retenue ou au rejugement, `lectures::ouvrir(w)` après la position 12, et la boucle
  des unités passe par `garder_l_unite` (sans `entre`, pour le clavier) : `Change` refuse « le premier plan a changé :
  <k> unités sur <n> sont parties » ; `FocusIllisible`, « KeePassXC : focus illisible ; … » ; `Part { motdepasse:
  true }`, le refus du champ pour `taper`, et pour `touche` hors des touches de déplacement nues. L'arbre de la
  garde se lit par `lectures::lire_l_arbre(pp.id, maintenant + DOCUMENT_VIDE_MAX, b)` quand `arbre_de` n'est pas
  `pp.id` ; un focus `Err`, `Ok(None)`, ou d'`indice` nul est illisible ; son `pid` ne se juge pas (décision 12).

- [ ] **Étape 1 : les tests, rouges.** `avec_keepassxc_par_explorer()` est celui de la tranche 1, le premier plan posé à
  sa fenêtre 7 (`pp(7, 60, Vec::new())`) pour que les unités partent sans activation ; `focus_mdp()` rend
  `Ok(Some(FocusLu { pid: 60, motdepasse: true, indice: Some(5) }))`, `focus_libre()` de même avec `motdepasse:
  false, indice: Some(4)`.
  ```rust
  #[test] fn keepassxc_taper_passe_hors_d_un_champ_de_mot_de_passe() { /* focus [focus_libre()] : « taper --fenetre
      explorer --texte ab » en 0, deux unités envoyées, OuvrirUia(DELAIS), un LireArbre(IdFenetre(7)), puis deux
      LireFocus */ }
  #[test] fn keepassxc_refuse_taper_dans_un_champ_de_mot_de_passe() { /* focus [focus_mdp()] : Refus « KeePassXC : focus
      dans un champ de mot de passe ; 0 unités sur 2 sont parties », aucune unité envoyée */ }
  #[test] fn keepassxc_relit_le_focus_avant_chaque_unite() { /* focus [libre, mdp] et « --texte abc » : Refus « … ; 1 unités
      sur 3 sont parties », une seule unité envoyée, deux LireFocus */ }
  #[test] fn keepassxc_sonde_l_arbre_avant_le_focus() { /* arbres [vide(), arbre_d_essai()] et focus [focus_libre()] :
      « touche --fenetre explorer Tab » en 0 ; deux LireArbre avant le premier LireFocus, ecoule ≥ 100 */ }
  #[test] fn keepassxc_premier_plan_change_pendant_la_lecture_du_focus() { /* W d'appli-essai, une fenêtre enfant de
      KeePassXC (la règle s'applique), focus [libre] ; vol_apres_le_focus (1, la fenêtre 2 de PID 100, que W possède,
      acceptée) : « taper --fenetre appli-essai --texte ab » → Refus « le premier plan a changé : 0 unités sur 2 sont
      parties », aucune unité envoyée : un premier plan relu, même accepté, doit égaler celui d'avant les lectures */ }
  #[test] fn keepassxc_refuse_un_texte_qui_porte_une_tabulation() { for t in ["a\tb", "a\nb", "a\r\nb"] { /* Refus « KeePassXC :
      un texte qui porte une tabulation ou un saut de ligne est refusé », sans PrendreVerrou */ } }
  #[test] fn keepassxc_touche_dans_un_champ_de_mot_de_passe() { for t in ["Tab", "Maj+Tab", "Haut", "Bas", "Gauche", "Droite",
      "Debut", "Fin", "PageHaut", "PageBas", "Echap"] { /* focus [mdp] : « touche --fenetre explorer <t> » en 0 */ }
      for t in ["Entree", "Espace", "Ctrl+V", "Ctrl+Tab", "Maj+Haut", "Alt+Bas", "F2"] { /* Refus « KeePassXC : seules les
      touches de déplacement passent dans un champ de mot de passe ; 0 unités sur 1 sont parties » */ } }
  #[test] fn keepassxc_focus_illisible_refuse() { /* focus [Ok(None)], puis [Err(ErreurUia { code: 0x80004005, .. })], puis
      [Ok(Some(FocusLu { pid: 60, motdepasse: false, indice: None }))] : chacun Refus « KeePassXC : focus illisible ; 0
      unités sur 1 sont parties » */ }
  #[test] fn keepassxc_attend_100_ms_entre_deux_unites() { /* « taper --texte ab » : Attendre(Duree(100)) après chaque
      unité, jamais Attendre(CADENCE) */ }
  #[test] fn keepassxc_enfant_de_w_applique_la_regle() { /* W d'appli-essai, une fenêtre enfant de KeePassXC : focus [mdp]
      → le même refus ; « activer » ne lit pas le focus */ }
  #[test] fn keepassxc_rejuge_apres_l_activation() { /* l'enfant KeePassXC ne paraît qu'à la relecture : « taper --texte
      a\tb » refuse au rejugement, après l'activation ; « taper --texte a » lit le focus avant son unité */ }
  #[test] fn hors_de_keepassxc_le_focus_ne_se_lit_pas() { /* appli-essai : « taper --texte ab » sans OuvrirUia, LireArbre
      ni LireFocus */ }
  #[test] fn niveau_juge_pour_un_dialogue_d_un_autre_processus() { /* processus 101, fils de 100 ; premier plan : fenêtre 2
      de PID 101, propriétaires [1] ; niveaux [(101, Some(0x3000))] : « touche Tab » → Refus « fenêtre plus élevée que
      pilote », aucune unité ; [(101, None)] → « niveau d'intégrité illisible » ; [(101, Some(0x2000))] et « --fois 3 » :
      Fait, Niveau(Some(101)) noté une fois */ }
  #[test] fn premier_plan_juste_avant_chaque_envoi_du_clavier() { /* premier_plan_juste_avant(&b) pour : « taper --fenetre
      explorer --texte ab » (KeePassXC, focus [libre]) ; « touche --fenetre explorer Tab » (KeePassXC, focus [mdp]) ;
      « touche --fenetre appli-essai Tab --fois 3 » sous le dialogue de PID 101 au niveau 0x2000 ; et, pour KeePassXC,
      un LireFocus entre les deux triplets qui précèdent chaque envoi */ }
  ```
  Les tests `keepassxc_par_explorer_refuse_taper`, `keepassxc_par_explorer_refuse_touche`,
  `keepassxc_enfant_de_w_refuse_taper` et le constructeur `refus_keepassxc` de la tranche 1 se retirent, remplacés
  par ceux-ci ; `premier_plan_verifie_avant_chaque_unite` reste vert tel quel.
- [ ] **Étape 2 : voir rouge** : `cargo test --lib domaine::gestes`.
- [ ] **Étape 3 : le code**, décisions 12 et 13.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/gestes\.rs:'`.
- [ ] **Étape 5 : les ADR.** ADR-0005, « Le premier plan accepté » : la garde d'une unité, les lectures d'entre-deux
  entre deux lectures égales du premier plan, l'envoi juste après la dernière (décision 13) ; « KeePassXC » : la puce
  « D'ici la tranche 2 … le refus est entier » cède à la règle en vigueur, décision 12, le focus jugé sur l'arbre lu
  et sa limite (le focus lu après une tabulation, 100 ms ; un élément né après la lecture refusé) comprises ; « Le
  niveau d'intégrité se juge par comparaison » : la décision 13 ; la conséquence « D'ici la tranche 2 du lot 1,
  KeePassXC ne reçoit ni `touche` ni `taper` » se retire. ADR-0006 : la ligne du message de KeePassXC se retire du
  tableau des messages, les quatre messages de la décision 12 y entrent, et le focus illisible de KeePassXC entre au
  1 de « Les codes de sortie ». « Confirmation » et les sous-sections reçoivent ces points.
- [ ] **Étape 6 : commit** : `git add src/domaine/gestes.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`
  (et `.cargo/mutants.toml` s'il change), puis
  `git commit -m "feat(domaine): la garde d'une unité, le champ de KeePassXC et le niveau des dialogues"`.

### Tâche 9 : les gestes de l'automatisation — focaliser, invoquer, choisir et defiler

**Fichiers :** créer `src/domaine/automatisation.rs` ; modifier `src/domaine/gestes.rs` (`Travail`,
`preparer_le_geste`, `Suivi::agir`), `src/domaine/mod.rs`, ADR-0005, ADR-0006.

**Interfaces :**
- Consomme : `lectures::{ouvrir, lire_l_arbre, SONDE, DOCUMENT_VIDE_MAX}` ; `choisir_un`, `analyser_le_selecteur` ;
  `arret_uia`, `UIA_DELAI` ; `garder_l_unite`, `Garde`, `Avant` (Tâche 8) ; `vol_apres_l_arbre`, `vol_apres_agir`,
  `gestes::essai` (Tâche 5) ; `sonder`.
- Produit :
  ```rust
  // automatisation.rs
  pub const ETAT_MAX: Duree = Duree(2_000);
  #[derive(Clone, Debug, PartialEq)] pub enum Demande { Focaliser, Invoquer, Choisir, Defiler(u8) }
  #[derive(Clone, Debug, PartialEq)] pub struct Automatisation { pub cible: Cible, pub selecteur: Selecteur, pub demande: Demande }
  pub(crate) fn automatiser(a: &Automatisation, s: &mut Suivi, b: &mut dyn Bureau) -> Result<Issue, Arret>;
  // gestes.rs
  /// Les positions 3 à 10 de l'ordre des gardes, communes à tout geste qui vise une fenêtre.
  pub(crate) struct Prepare { pub r: Retenue, pub w: FenetreLue, pub tous: Vec<Processus>, pub pp: PremierPlan }
  pub(crate) fn preparer_le_geste(cible: &Cible, frappe: Option<&Frappe>, s: &mut Suivi, b: &mut dyn Bureau) -> Result<Prepare, Arret>;
  impl Suivi { /// Compte le geste, puis agit.
      pub(crate) fn agir(&mut self, b: &mut dyn Bureau, i: usize, g: GesteUia) -> Result<(), ErreurUia> }
  ```
  `geste` (le clavier) passe par `preparer_le_geste` sans changer d'ordre ; les tests de la tranche 1 restent
  verts tels quels. Chaque appel qui agit passe par `garder_l_unite` (sans `entre`), puis `Suivi::agir` ; `Change`
  refuse « le premier plan a changé : <k> unités sur <n> sont parties » (décision 9) ; une erreur de `agir` au code
  `UIA_DELAI` sort en « échec du système : délai de l'automatisation dépassé pendant le geste ; il a pu agir : relire
  avant de rejouer », toute autre par `arret_uia`.

- [ ] **Étape 1 : les tests, rouges.** `etat(c)` rend `EtatLu { coche: Some(c), choisi: None }`.
  ```rust
  #[test] fn invoquer_agit_apres_le_premier_plan() { let mut b = FauxBureau::avec_appli();
      assert_eq!(x(&mut b, "invoquer --fenetre appli-essai --nom Départ"), Issue::fait(Vec::new()));
      let apres = b.appels.iter().skip_while(|a| !matches!(a, AppelFaux::LireArbre(_))).cloned().collect::<Vec<_>>();
      assert!(apres.contains(&AppelFaux::PremierPlan)); assert_eq!(compte(&b, &AppelFaux::Agir(3, GesteUia::Invoquer)), 1);
      /* PrendreVerrou, EtatSession, Niveau, TouchesEnfoncees viennent avant Repond, OuvrirUia et LireArbre */ }
  #[test] fn invoquer_un_element_sans_le_geste_sort_en_3() { /* --id titre-defilee : Echec « échec du système : l'élément
      n'offre pas le motif Invoke », aucun Agir */ }
  #[test] fn element_disparu_sort_en_2() { /* erreur_agir code UIA_DISPARU : Introuvable « l'élément a disparu » */ }
  #[test] fn premier_plan_perdu_avant_le_geste() { /* vol_apres_l_arbre (1, la fenêtre 9 d'un processus 300) :
      « invoquer --id depart » → Refus « le premier plan a changé : 0 unités sur 1 sont parties », aucun Agir */ }
  #[test] fn delai_pendant_le_geste_dit_qu_il_a_pu_agir() { /* erreur_agir au code UIA_DELAI : « invoquer --id depart » →
      Echec « échec du système : délai de l'automatisation dépassé pendant le geste ; il a pu agir : relire avant de
      rejouer », Agir(3, Invoquer) noté, la ligne du journal au code 3 ; erreur_arbre au code UIA_DELAI : Echec « échec
      du système : délai de l'automatisation dépassé », sans la suite */ }
  #[test] fn invoquer_un_element_inactif_sort_en_3() { /* erreur_agir au code UIA_INACTIF : Echec « échec du système :
      l'élément est inactif » ; au code UIA_NON_PRIS : « échec du système : l'élément refuse le geste » */ }
  #[test] fn geste_de_l_automatisation_prend_le_verrou() { /* verrou occupé : Refus « un autre geste est en cours depuis 5 s »,
      ni OuvrirUia ni LireArbre */ }
  #[test] fn geste_compte_avant_d_agir() { /* erreur_agir de code UIA_DISPARU et ajout_rate : Echec « échec du système : le
      journal n'a pas reçu sa ligne », le geste compté même quand l'appel échoue ; sans erreur_agir, ajout_rate : la même
      issue */ }
  #[test] fn choisir_attend_l_etat_avant_de_rebasculer() { /* etats [etat(0), etat(0), etat(0), etat(1)] : « choisir --id case »
      Fait, Agir(6, Basculer) une fois, ecoule == 300 */ }
  #[test] fn choisir_rebascule_un_etat_indetermine() { /* etats [etat(2), etat(1)] : deux Agir(6, Basculer), Fait */ }
  #[test] fn choisir_sort_en_3_si_l_etat_ne_change_pas() { /* etats [etat(0)] : Echec « échec du système : l'état n'a pas
      changé en 2 s », une bascule, ecoule == 2000 */ }
  #[test] fn choisir_sort_en_3_apres_deux_bascules() { /* etats [etat(2), etat(0)] : Echec « échec du système : l'élément n'est
      pas coché après deux bascules », deux bascules */ }
  #[test] fn choisir_ne_decoche_jamais() { /* case à coche Some(1) : Fait, aucun Agir, aucun RelireEtat */ }
  #[test] fn choisir_choisit_un_bouton_radio() { /* choisi Some(false) sur 8, sans Toggle ; etats [choisi Some(false),
      choisi Some(true)] : « choisir --id couleur-toutes » Fait, Agir(8, Choisir) une fois ; choisi Some(true) : aucun Agir */ }
  #[test] fn choisir_sans_motif_sort_en_3() { /* --id depart : Echec « échec du système : l'élément n'offre ni Toggle ni
      SelectionItem » */ }
  #[test] fn sondes_d_etat_sans_premier_plan() { /* entre Agir(6, Basculer) et le dernier RelireEtat, aucun PremierPlan */ }
  #[test] fn seconde_bascule_rejuge_le_premier_plan() { /* etats [etat(2), etat(1)] et vol_apres_agir (1, la fenêtre 9
      d'un processus 300) : Refus « le premier plan a changé : 1 unités sur 2 sont parties », une seule bascule */ }
  #[test] fn defiler() { /* « defiler --id cadre --a 40 » : Agir(17, Defiler(40)) ; --id titre-defilee : Echec « échec du
      système : l'élément n'offre pas le motif Scroll » ; defile horizontal seul : Echec « échec du système : l'élément ne
      défile pas verticalement » */ }
  #[test] fn focaliser() { /* « focaliser --id zone » sur un Edit d'id zone : Agir(i, Focaliser), Fait, sortie vide */ }
  #[test] fn ambiguite_avant_tout_geste() { /* « invoquer --nom Toutes » : Introuvable « plusieurs éléments … : 3 ; --rang
      départage », aucun Agir */ }
  #[test] fn niveau_d_un_dialogue_avant_l_agir() { /* premier plan : dialogue de PID 101 au niveau 0x3000 : Refus « fenêtre
      plus élevée que pilote », aucun Agir */ }
  #[test] fn premier_plan_juste_avant_chaque_agir() { /* premier_plan_juste_avant(&b) pour « focaliser --id zone »,
      « invoquer --id depart », « choisir --id case » (etats [etat(2), etat(1)], deux bascules), « choisir --id
      couleur-toutes » (Select), « defiler --id cadre --a 40 », et « invoquer --id depart » sous le dialogue de PID 101
      au niveau 0x2000 */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- domaine::automatisation domaine::gestes`.
- [ ] **Étape 3 : le code**, décision 9 ; l'analyse juge `analyser_le_selecteur(sel, true)` pour les quatre.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/automatisation\.rs:'` et
  `-F '^src/domaine/gestes\.rs:'`.
- [ ] **Étape 5 : les ADR.** ADR-0005, « Les gestes et l'ordre des gardes » : l'ordre de la décision 9, la garde
  d'une unité avant chaque appel qui agit, pas avant les sondes ; « Le journal », « Après un geste » : l'appel qui
  agit compte dès avant de partir. ADR-0006, « Les gestes de l'automatisation » : l'algorithme de `choisir`,
  `defiler` vertical et le délai dépassé pendant le geste ; « Les codes de sortie », ces lignes : au 1, le premier
  plan changé avant un appel de l'automatisation ; au 3, l'état inchangé de `choisir`, l'élément qui n'est pas
  coché après deux bascules, qui ne défile pas verticalement, le délai dépassé pendant le geste ; « Les messages » :
  ceux de l'automatisation. « Confirmation » et les sous-sections reçoivent ces points.
- [ ] **Étape 6 : commit** : `git add src/domaine/automatisation.rs src/domaine/gestes.rs src/domaine/mod.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`
  (et `.cargo/mutants.toml` s'il change), puis `git commit -m "feat(domaine): focaliser, invoquer, choisir et defiler"`.

### Tâche 10 : le pointeur — clic et molette

**Fichiers :** créer `src/domaine/pointeur.rs` ; modifier `src/domaine/gestes.rs` (`Travail`),
`src/domaine/mod.rs`, ADR-0005, ADR-0006.

**Interfaces :**
- Consomme : `Evenement::{Pointeur, Bouton, Molette}`, `CRAN` (Tâche 2) ; `zone_cliente`, `fenetre_sous`,
  `bureau_virtuel`, `boutons_inverses`, `sensible_au_dpi`, `zones`, `gestes::essai` (Tâche 5) ; `garder_l_unite`,
  `Garde`, `Avant` (Tâche 8) ; `preparer_le_geste` (Tâche 9) ; `insertion_complete`, `accepte`, `CADENCE`.
- Produit :
  ```rust
  #[derive(Clone, Debug, PartialEq)] pub enum Action { Clic { double: bool, droit: bool }, Molette(i32) }
  #[derive(Clone, Debug, PartialEq)] pub struct Pointage { pub cible: Cible, pub x: i32, pub y: i32, pub action: Action }
  /// Le point d'écran `p`, en coordonnées normalisées du bureau virtuel, au quart de pixel (décision 11).
  pub fn normaliser(p: Point, virtuel: Rect) -> (u16, u16);
  /// Le lot d'une unité, au point normalisé : un clic, un double clic ou un cran ; `inverses` échange les boutons.
  pub fn lot(action: &Action, x: u16, y: u16, inverses: bool) -> Vec<Evenement>;
  pub(crate) fn pointer(p: &Pointage, s: &mut Suivi, b: &mut dyn Bureau) -> Result<Issue, Arret>;
  ```
  `pointer` : `preparer_le_geste`, les touches enfoncées relues, la sensibilité au DPI ; puis, pour chaque unité (une
  pour un clic, `|crans|` pour la molette), `garder_l_unite` avec un `entre` qui relit la zone cliente et le bureau
  virtuel, calcule le point d'écran, juge la zone, l'écran et la fenêtre sous le point (par `accepte`, sur
  l'instantané reçu), garde ce point, et rend le PID de la fenêtre sous le point ; puis `normaliser` ce même point,
  `lot`, l'envoi, `insertion_complete`, la cadence. Aucun lot ne se calcule d'avance : chaque unité vise le point
  qu'elle vient de juger.

- [ ] **Étape 1 : les tests, rouges.** `v(x, y, l, h)` rend un `Rect`.
  ```rust
  #[test] fn normalisation_vise_le_quart_du_pixel() { let e = v(0, 0, 1024, 768);
      assert_eq!(normaliser(Point { x: 0, y: 0 }, e), (16, 21)); assert_eq!(normaliser(Point { x: 512, y: 384 }, e), (32784, 32789));
      assert_eq!(normaliser(Point { x: 1023, y: 767 }, e), (65488, 65472));
      let d = v(-1920, 0, 3840, 1080); assert_eq!(normaliser(Point { x: -1920, y: 0 }, d).0, 4); assert_eq!(normaliser(Point { x: 0, y: 0 }, d).0, 32772); }
  #[test] fn clic_envoie_un_seul_lot() { let mut b = FauxBureau::avec_appli();
      assert_eq!(x(&mut b, "clic --fenetre appli-essai --x 10 --y 20"), Issue::fait(Vec::new()));
      assert_eq!(b.envois(), vec![vec![Evenement::Pointeur { x: 3763, y: 13365 },
          Evenement::Bouton { droit: false, relache: false }, Evenement::Bouton { droit: false, relache: true }]]); }
  #[test] fn double_clic_un_seul_lot_de_cinq() { /* --double : un envoi [Pointeur, enfoncé, relâché, enfoncé, relâché] */ }
  #[test] fn clic_droit_porte_le_bouton_droit() { /* --droit : Bouton { droit: true, … } ; --double --droit : cinq, droits */ }
  #[test] fn boutons_inverses_echangent_les_boutons() { /* inverses : un clic simple porte droit: true, --droit porte droit: false */ }
  #[test] fn molette_un_lot_par_cran() { /* --crans -3 : trois envois [Pointeur, Molette { delta: -CRAN }], chacun suivi
      d'Attendre(CADENCE) ; --crans 2 : deux, delta CRAN */ }
  #[test] fn point_hors_de_la_zone_cliente_sort_en_1() { for (px, py) in [(800, 10), (-1, 10), (10, 600), (10, -1)] {
      /* Refus « point hors de la zone cliente : <px>,<py> pour 800×600 », aucun Pointeur envoyé */ }
      /* (799, 599) passe */ }
  #[test] fn clic_refuse_un_point_hors_de_l_ecran() { /* zones [Some(Rect { x: 1900, y: 200, largeur: 800, hauteur: 600 })] :
      « --x 50 --y 10 » → Refus « point hors de l'écran : 50,10 » */ }
  #[test] fn clic_refuse_un_point_recouvert_dans_le_domaine() { /* sous_le_point : fenêtre 9 de PID 300 : Refus « point recouvert
      par une autre fenêtre », aucun envoi ; sous_le_point None : de même */ }
  #[test] fn clic_sur_un_dialogue_de_w_passe() { /* sous_le_point : fenêtre 2 de PID 100, propriétaires [1] : Fait */ }
  #[test] fn zone_cliente_illisible_sort_en_3() { /* zones [None] : Echec « échec du système : zone cliente illisible » */ }
  #[test] fn pointeur_sans_dpi_par_ecran_sort_en_3() { /* dpi_par_ecran faux : clic, puis molette : Echec « échec du
      système : pilote n'est pas sensible au DPI », ni ZoneCliente ni envoi */ }
  #[test] fn molette_s_arrete_quand_le_premier_plan_change() { /* --crans 3, le premier plan volé après le 1er envoi :
      Refus « le premier plan a changé : 1 unités sur 3 sont parties » */ }
  #[test] fn chaque_unite_relit_le_point() { /* --crans 2 : ZoneCliente et FenetreSous notés avant chaque envoi ; zones
      [Some(v(100, 200, 800, 600)), Some(v(300, 200, 800, 600))] et « molette --x 10 --y 20 --crans 2 » : le premier
      Pointeur vise l'écran (110, 220), le second (310, 220), chacun normalisé */ }
  #[test] fn niveau_d_un_dialogue_sous_le_point() { /* processus 101, fils de 100 ; sous_le_point : fenêtre 2 de PID 101,
      propriétaires [1] ; niveaux [(101, Some(0x3000))] : « clic --x 10 --y 20 » → Refus « fenêtre plus élevée que
      pilote », aucun envoi ; niveaux [(101, Some(0x2000))] et « molette --x 10 --y 20 --crans 3 » : Fait,
      Niveau(Some(101)) noté une fois */ }
  #[test] fn premier_plan_juste_avant_chaque_envoi_du_pointeur() { /* premier_plan_juste_avant(&b) pour « clic --x 10 --y
      20 », « clic --x 10 --y 20 --double » et « molette --x 10 --y 20 --crans 3 » : chaque envoi suit le triplet, et
      ZoneCliente et FenetreSous se lisent entre les deux triplets qui le précèdent */ }
  #[test] fn clic_prend_les_gardes() { /* une touche de modification enfoncée : Refus « touche de modification enfoncée »,
      aucun Pointeur ; la fenêtre plus élevée : Refus, aucun envoi */ }
  #[test] fn clic_insere_en_partie_relache_le_bouton() { /* insertions [2, 1] : Echec « échec du système : SendInput a inséré 2
      événements sur 3 ; relâchement : 1 sur 1 » */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib domaine::pointeur`.
- [ ] **Étape 3 : le code**, décisions 11 et 13.
- [ ] **Étape 4 : voir vert**, puis `cargo mutants -p pilote -F '^src/domaine/pointeur\.rs:'`.
- [ ] **Étape 5 : les ADR.** ADR-0005, « Les gestes et l'ordre des gardes » : la phrase « Aux tranches suivantes :
  pour `clic` et `molette` … » devient la règle au présent, avec la décision 11 : la sensibilité au DPI, le point
  relu et jugé à chaque unité, la normalisation, son échelle et l'essai qui la départage, les boutons inversés ; le
  niveau de la fenêtre sous le point (décision 13). ADR-0006 : « Les bornes », `--x` et `--y` ; « Les codes de
  sortie », ces lignes : au 1, les trois refus du point à la place de « point de clic ou de molette hors de la
  fenêtre retenue » ; au 3, la zone cliente illisible et `pilote` qui n'est pas sensible au DPI ; « Les messages »,
  ceux du point. « Confirmation » et les sous-sections reçoivent ces points.
- [ ] **Étape 6 : commit** : `git add src/domaine/pointeur.rs src/domaine/gestes.rs src/domaine/mod.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`
  (et `.cargo/mutants.toml` s'il change), puis `git commit -m "feat(domaine): clic et molette au point de la zone cliente"`.

### Tâche 11 : la couche Windows — l'automatisation, la capture, le pointeur et Verr. Maj

**Fichiers :** créer `src/bureau/uia.rs`, `src/bureau/capture.rs` ; modifier `src/bureau/mod.rs`,
`src/bureau/fenetres.rs`, `src/bureau/clavier.rs`, `Cargo.toml`, `Cargo.lock`.

**Interfaces :**
- Consomme : le trait de la Tâche 5.
- Produit : `BureauWindows` réalise les quatorze méthodes nouvelles et `verr_maj` en `Option` ; plus aucun
  provisoire. Les appels, sans condition qui décide :
  - `ouvrir_l_automatisation` : `CoCreateInstance::<_, IUIAutomation>(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)` ;
    `cast::<IUIAutomation2>()` puis `SetConnectionTimeout`, `SetTransactionTimeout` (les millisecondes reçues),
    `SetAutoSetFocus(false)` ; `cast::<IUIAutomation6>()`, s'il réussit, puis
    `SetConnectionRecoveryBehavior(ConnectionRecoveryBehaviorOptions_Disabled)` ; une requête de cache
    (`CreateCacheRequest`, `TreeScope_Subtree`, le filtre par défaut) qui ajoute les propriétés
    `UIA_ControlTypePropertyId`, `UIA_NamePropertyId`, `UIA_AutomationIdPropertyId`, `UIA_ClassNamePropertyId`,
    `UIA_AcceleratorKeyPropertyId`, `UIA_AccessKeyPropertyId`, `UIA_LiveSettingPropertyId`,
    `UIA_IsPasswordPropertyId`, `UIA_IsEnabledPropertyId`, `UIA_ProcessIdPropertyId`,
    `UIA_ToggleToggleStatePropertyId`, `UIA_ExpandCollapseExpandCollapseStatePropertyId`,
    `UIA_SelectionItemIsSelectedPropertyId`, `UIA_ValueIsReadOnlyPropertyId`,
    `UIA_ScrollVerticallyScrollablePropertyId`, `UIA_ScrollHorizontallyScrollablePropertyId`, et les motifs
    `UIA_InvokePatternId`, `UIA_TogglePatternId`, `UIA_SelectionItemPatternId`, `UIA_ValuePatternId`,
    `UIA_ScrollPatternId`, `UIA_ExpandCollapsePatternId`. Le client et la requête vivent dans `BureauWindows`.
  - `lire_l_arbre` : `ElementFromHandleBuildCache(W, requête)`, puis `GetCachedChildren` en profondeur d'abord, une
    erreur de code `HRESULT(0)` lue comme « aucun enfant » ; chaque motif par `GetCachedPatternAs`, une erreur de code
    `HRESULT(0)` lue comme `None`, puis l'état par le `Cached…` du motif (`CachedToggleState`, `CachedIsSelected`,
    `CachedExpandCollapseState`, `CachedIsReadOnly`, `CachedVerticallyScrollable` et `CachedHorizontallyScrollable`),
    jamais comme une propriété de l'élément : une `Option` d'`ElementLu` n'est `Some` que si le motif est offert
    (Tâche 3) ; les éléments gardés, dans l'ordre de la liste rendue. `CachedIsPassword` qui échoue fait échouer la
    lecture entière, en `ErreurUia` : jamais `false` par défaut.
  - `lire_le_focus` : `GetFocusedElement` (`HRESULT(0)` : `Ok(None)`), `CurrentProcessId`, `CurrentIsPassword`,
    l'indice par `CompareElements` contre les éléments gardés ; une erreur de `CurrentProcessId` ou de
    `CurrentIsPassword` rend `Err`, que le domaine dit « focus illisible » pour KeePassXC. Le commentaire de ces deux
    méthodes dit cette règle ; la revue de la tâche vérifie qu'aucun `unwrap_or_default`, `unwrap_or(false)`,
    `unwrap_or_else` ni `.ok()` ne touche `IsPassword` ni `ProcessId`.
  - `lire_la_valeur` : le motif Value en cache de l'élément gardé, puis `CurrentValue`.
  - `relire_l_etat` : `CurrentToggleState` et `CurrentIsSelected` des motifs en cache.
  - `agir` : `SetFocus`, `Invoke`, `Toggle`, `Select`, ou `SetScrollPercent(UIA_ScrollPatternNoScroll, f64::from(a))`.
  - `ErreurUia` : `code` est `e.code().0 as u32` ; `texte`, « <appel> : 0x<code> », en huit chiffres hexadécimaux
    majuscules, suivi de « : <message> » si `e.message()` n'est pas vide, ou « <appel> : rien rendu » pour
    `HRESULT(0)`. Un indice hors de la dernière lecture rend `E_INVALIDARG` (0x80070057), « <appel> : indice hors de
    l'arbre ».
  - `repond` : `SendMessageTimeoutW(W, WM_NULL, 0, 0, SMTO_ABORTIFHUNG, délai, None)` non nul.
  - `capturer` (`capture.rs`) : `GetWindowRect` ; `CreateCompatibleDC(None)` ; `CreateDIBSection` 32 bits,
    `biHeight` négatif, `BI_RGB` ; `SelectObject`, qui rend l'objet d'avant ; `PrintWindow(W, hdc,
    PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT))` ; `GdiFlush()`, avant toute lecture des bits (`CreateDIBSection`
    l'exige) ; la copie des octets ; `SelectObject(hdc, ancien)`, puis `DeleteObject(bitmap)` (une bitmap encore
    choisie dans un DC ne se supprime pas), puis `DeleteDC` ; `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)` ;
    la zone cliente. Une garde à `Drop`, qui ne panique pas, tient le DC, la bitmap et l'objet d'avant, pour que
    chaque sortie anticipée les rende (`HDC` n'a pas de `Free` dans `windows` 0.62.2). Un échec de `PrintWindow`
    rend `ErreurSysteme("PrintWindow")`.
  - `ecrire_fichier` : `std::fs::write`, l'erreur dite.
  - `zone_cliente` : `GetClientRect` puis `ClientToScreen`.
  - `fenetre_sous` : `WindowFromPoint`, `GetAncestor(GA_ROOT)`, puis le PID et la chaîne des propriétaires comme
    `premier_plan` les lit.
  - `bureau_virtuel` : `GetSystemMetrics` de `SM_XVIRTUALSCREEN`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`,
    `SM_CYVIRTUALSCREEN` ; `boutons_inverses` : `GetSystemMetrics(SM_SWAPBUTTON) != 0`.
  - `sensible_au_dpi` : `AreDpiAwarenessContextsEqual(GetThreadDpiAwarenessContext(),
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`.
  - `verr_maj` : `GetKeyState(VK_CAPITAL) & 1` sur un fil neuf (`thread::Builder::spawn`, puis `join`), `Some` de
    la lecture ; un fil qui ne naît pas ou qui panique rend `None`, que le domaine refuse (Tâche 5).

- [ ] **Étape 1 : les features et les modules.** `Cargo.toml` : `Win32_Graphics_Gdi`, `Win32_Storage_Xps`,
  `Win32_UI_Accessibility`, à leur place dans la liste triée. Aucun test : la couche n'en a pas.
- [ ] **Étape 2 : le code**, méthode par méthode, chaque bloc `unsafe` avec son `// SAFETY:` ; les objets de
  l'automatisation naissent et meurent sur le fil de `lancer` ; « (décision N) » remplacé dans ces fichiers s'il en
  reste.
- [ ] **Étape 3 : vérifier** : `cargo build -p pilote`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` ; `git grep -n "pas encore réalisé" -- src` ne rend rien.
- [ ] **Étape 4 : commit** : `git add src/bureau/uia.rs src/bureau/capture.rs src/bureau/mod.rs src/bureau/fenetres.rs src/bureau/clavier.rs Cargo.toml Cargo.lock`,
  puis `git commit -m "feat(bureau): automatisation, capture, pointeur et Verr. Maj sur un fil neuf"`.

### Tâche 12 : l'application d'essai

**Fichiers :** modifier `essai/appli/src/main.rs`, `essai/appli/src/page.html`.

**Interfaces :**
- Produit : quatre options, chacune une fois au plus : `--couleur <#rrggbb>` (la page ne montre qu'un bloc fixe de
  cette couleur sur toute la fenêtre), `--au-dessus` (la fenêtre toujours au-dessus), `--position <x>,<y>` (sa
  position extérieure, en pixels physiques, négatifs admis), `--taille <l>,<h>` (sa zone cliente, en pixels
  physiques, 1 au moins) ; chacune refusée sinon par `refusee(o, valeur)`. L'hôte écrit `{"genre":"gel"}` avant de
  dormir ; la page écrit `{"genre":"fige"}` avant sa boucle de 8 s, lancée par `setTimeout(…, 0)`. La page dit
  `clic`, `double` et `droit` (`click`, `dblclick`, `contextmenu`, leur `id`), `change` (son `id`, `etat` à
  `checked`), `defile` (son `id`, `valeur` au pourcentage entier de défilement vertical) ; `#cadre` porte
  `role="region"`, `aria-label="Cadre des lignes"` et `tabindex="-1"`, pour qu'il puisse prendre le focus hors de
  l'ordre de tabulation, et que `defilables_et_defiler` ait son témoin ; à la fin du document, hors de l'ordre de
  tabulation (`tabindex="-1"`) : un bouton `#cible` « Cible », fixe en (300, 200) CSS, 100 × 50 ; un cadre
  `#roulette`, fixe en (300, 300), 200 × 100, de 30 lignes ; un bloc vide `#inerte` (`role="note"`,
  `aria-label="Inerte"`), qui n'offre ni Invoke ni Scroll ; une case `#case-lente` (`role="checkbox"`,
  `aria-checked="false"`, « Case lente ») : un élément ARIA n'émet aucun `change` natif, si bien que, 300 ms après
  un clic, la page passe `aria-checked` à `true` et écrit elle-même `{"genre":"change","id":"case-lente","etat":true}` ;
  une case `#case-fugace` (`role="checkbox"`, `aria-checked="false"`, « Case fugace »), que son clic retire de la
  page, pour que la sonde d'état relise un élément disparu ; un bouton `#retarder` « Retarder », qui ajoute un bouton
  « Arrivé » une seconde après son clic. L'ordre de tabulation du début reste celui de la tranche 1.

- [ ] **Étape 1 : les tests de `regles`, écrits** (ils portent l'attribut de l'application, `#[ignore]` : ils ne se
  voient rouges puis verts qu'en CI, lancés par la sentinelle, à la Tâche 13) :
  ```rust
  #[test] fn options_de_fenetre_se_lisent() { /* « --couleur #3366cc --au-dessus --position -10,20 --taille 800,600 » :
      couleur Some("#3366cc"), au_dessus vrai, position Some((-10, 20)), taille Some((800, 600)) */ }
  #[test] fn options_de_fenetre_refusees() { /* --couleur 3366cc, --couleur #33gg00, --position 10, --taille 0,600,
      --taille 800 : chacune refusée par son message ; chacune répétée : refusée */ }
  #[test] fn gel_ecrit_sa_ligne() { /* la ligne que l'hôte écrit avant de dormir : ligne(TITRE, "{\"genre\":\"gel\"}") */ }
  ```
- [ ] **Étape 2 : voir compiler** : `cargo build -p appli-essai` ; `cargo test --workspace` (ces tests restent
  ignorés sur le poste, la sentinelle de la CI les lance).
- [ ] **Étape 3 : le code** ; `Window::set_focus()` jamais appelé.
- [ ] **Étape 4 : commit** : `git add essai/appli/src/main.rs essai/appli/src/page.html`, puis
  `git commit -m "feat(essai): options de fenêtre, cibles du pointeur et signaux de gel"`.

### Tâche 13 : les aides des essais, l'artefact, les délais provisoires et les essais de lecture

Les Tâches 13 à 16 portent les essais de bout en bout, que seule la CI lance : chacune s'envoie, se lit en CI, puis
se relit, pour qu'un rouge ne mêle jamais les défauts de plusieurs tâches. Chaque essai porte
`#[ignore = "il pilote le bureau : la CI le lance"]` et suit les contraintes générales : une lecture qui suit un
geste attend d'abord l'événement qui le prouve, puis `CALME`, ou une lecture qui attend ; une preuve négative a son
témoin.

**Fichiers :** modifier `tests/commun/mod.rs`, `.github/workflows/ci.yml`, ADR-0005, ADR-0006 ; créer
`tests/essais_lecture.rs`.

**Interfaces :**
- Consomme : tout ce qui précède ; `lancer_appli`, `Profil`, `pilote`, `Sortie`, `sonder`, `exiger_la_ci`, `CALME`.
- Produit (`tests/commun/mod.rs`, chaque fonction publique commençant par `exiger_la_ci`) :
  ```rust
  pub fn image_png(chemin: &Path) -> (u32, u32, Vec<u8>);        // RGB 8 bits, par png::Decoder ; panique sinon
  pub fn zone_cliente(appli: &Appli) -> (i32, i32, u32, u32);   // à l'écran
  pub fn cadre_dwm(appli: &Appli) -> (i32, i32, u32, u32);
  pub fn position_du_pointeur() -> (i32, i32);
  pub fn dpi(appli: &Appli) -> u32;
  /// Le PID de la fenêtre de premier niveau sous le point d'écran (`WindowFromPoint`, puis `GA_ROOT`).
  pub fn fenetre_au_point(x: i32, y: i32) -> Option<u32>;
  pub fn bureau_virtuel() -> (i32, i32, u32, u32);
  ```

- [ ] **Étape 1 : les aides**, dans `tests/commun/mod.rs`.
- [ ] **Étape 2 : `tests/essais_lecture.rs`** :
  - `lecture_sonde_un_document_vide` : sitôt `pret`, `arbre` en 0 ; la première ligne commence par
    `[Window] <Appli d'essai>` ; une ligne, blancs ôtés, commence par `[Button] <Départ> id=depart` ; erreur vide.
  - `lecture_rend_raccourci_annonce_et_etat` : `trouver --id recherche` porte `raccourci=/` ; `trouver --id case`
    porte `etat=decoche` ; `invoquer --id annoncer` en 0, puis `trouver --id annonce --attendre 3000` porte
    `annonce=polite`. Chacun de ces champs passe, dans la couche, d'une propriété en cache à une clé de la ligne.
  - `focus_rend_l_element_focalise` : `--focus champ`, `activer` en 0, puis `focus --id champ --attendre 3000` en 0 :
    une ligne, qui commence par `[Edit] <Champ> id=champ` et porte ` valeur=""`.
  - `focus_hors_de_la_fenetre_visee_sort_en_1` : l'application et une copie `autre`, `activer --fenetre autre`, puis
    `focus --fenetre appli-essai` : (1, « focus hors de la fenêtre visée »), sortie vide.
  - `trois_toutes_departages_par_dans` (chaque sélecteur porte `--type RadioButton`, pour qu'un texte « Toutes » de
    Chromium ne compte pas) : `trouver --type RadioButton --nom Toutes` : trois lignes, chacune commençant par
    `[RadioButton] <Toutes>`, portant `dans=Couleur rang=1`, `dans=Taille rang=2`, `dans=Forme rang=3` ;
    `… --dans Taille` : une ligne, `id=taille-toutes` ; `focaliser --type RadioButton --nom Toutes` : (2, « plusieurs
    éléments répondent au sélecteur : 3 ; --rang départage »).
  - `groupes_emboites_de_meme_nom` : `trouver --type CheckBox --dans Options` : deux lignes, `id=option-externe` puis
    `id=option-interne`, chacune `dans=Options` ; `arbre --dans Options` : la première ligne commence par
    `[Group] <Options>`, une ligne par `  [Group] <Options>`, et la ligne qui, blancs ôtés, commence par
    `[CheckBox] <Option interne>` est indentée de plus de deux espaces.
  - `mot_de_passe_ne_sort_jamais` : `--focus motdepasse`, `taper --texte abc` en 0 ; la troisième `saisie` d'id
    `motdepasse`, puis `CALME` : le champ porte bien trois caractères quand on lit ; `arbre` : la ligne
    `id=motdepasse` porte `motdepasse=oui`, aucune ligne ne porte `abc`, ni `valeur=` sur cette ligne ;
    `valeur --id motdepasse` et `valeur --id motdepasse --egale abc` : (1, « valeur non rendue : champ de mot de
    passe ») ; `trouver --id motdepasse` : `motdepasse=oui`, sans `abc`.
  - `valeur_lit_le_champ` : `--focus champ`, `taper --texte bonjour` ; l'événement de la valeur complète
    (`"valeur":"bonjour"`), puis `CALME` ; `valeur --id champ --egale bonjour --attendre 3000` en 0 ; puis `valeur
    --id champ` porte `valeur=bonjour` ; `--egale autre` : (2, « la valeur diffère de « autre » »).
  - `trouver_attend_un_element_qui_arrive` : `trouver --type Button --nom Arrivé` : (2, « aucun élément ne répond au
    sélecteur ») ; `invoquer --type Button --nom Retarder` en 0 ; `trouver --type Button --nom Arrivé --attendre 5000`
    en 0, une ligne qui commence par `[Button] <Arrivé>`.
  - `arbre_et_capture_d_arriere_plan_laissent_le_premier_plan` : l'application puis `autre`, `activer --fenetre
    autre` ; `arbre --fenetre appli-essai` et `capture --fenetre appli-essai --sortie <profil>/arriere.png` en 0 ;
    `fenetres` donne `premier-plan=oui` à `autre`.
  - `capture_relue` : l'application `--couleur #3366cc --position 100,100 --taille 800,600`, puis `autre`
    `--couleur #cc6633 --au-dessus --position 100,100 --taille 800,600`, par-dessus elle ; le témoin d'abord :
    `fenetre_au_point` au centre de la zone cliente de l'application rend le PID d'`autre`, sinon panique
    (« autre ne couvre pas l'application : l'essai ne prouverait rien ») ; `capture --fenetre appli-essai` en 0 ;
    l'image relue par `image_png` a la largeur et la hauteur de la ligne ; son pixel au centre de la zone cliente
    vaut (0x33, 0x66, 0xcc) à 8 près sur chaque composante, ni noir ni (0xcc, 0x66, 0x33).
  - `capture_rogne_l_ombre` : la largeur et la hauteur de l'image égalent celles de `cadre_dwm` ; `zone-cliente=` égale
    `zone_cliente` moins l'origine de `cadre_dwm`.
  - `capture_refuse_une_fenetre_reduite` : `--reduite` ; (1, « capture refusée : fenêtre réduite »), le fichier absent.
  - `arbre_d_un_hote_fige_sort_en_3` : `invoquer --type Button --nom "Geler l'hôte"` en 0 ; la ligne `"genre":"gel"`
    en 5 s au plus ; `arbre` : (3, « échec du système : la fenêtre ne répond pas ») en moins de 6 s.
  - `capture_d_un_hote_fige_sort_en_3` : de même pour `capture`, le fichier absent.
  - `arbre_d_une_page_figee_rend_l_arbre_d_avant` : `arbre` en 0 ; `invoquer --type Button --nom "Figer la page"` en
    0 ; la ligne `"genre":"fige"` ; `arbre` en 0 en moins de 6 s, une ligne commençant (blancs ôtés) par
    `[Button] <Départ>`.
- [ ] **Étape 3 : la CI.** L'étape de dépôt ajoute `${{ runner.temp }}/pilote-essais/*/*.png`, avant les deux
  exclusions, qui restent dernières, et son commentaire le nomme. `windows` passe à `timeout-minutes: 20 #
  provisoire : remesuré à la Tâche 17 de la tranche 2`, `mutants` à `40`, même commentaire.
- [ ] **Étape 4 : les ADR.** Les « Confirmation » d'ADR-0005 et d'ADR-0006 nomment ces essais, chacun sous la règle
  qu'il éprouve (`tests/essais_lecture.rs`).
- [ ] **Étape 5 : vérifier sur le poste** : `cargo build -p pilote -p appli-essai`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace` (le contrôle statique voit chaque essai ignoré et aucun
  lancement hors du module commun) ; jamais les essais eux-mêmes.
- [ ] **Étape 6 : commit et envoi** : `git add tests/commun/mod.rs tests/essais_lecture.rs .github/workflows/ci.yml docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`,
  puis `git commit -m "test(essais): la lecture et la capture de bout en bout, délais provisoires"`, puis
  `git push origin feat/tranche2`. La PR s'ouvre en brouillon si elle ne l'est pas.
- [ ] **Étape 7 : lire la CI.** `gh pr checks` ; un essai rouge se lit dans son journal et l'artefact ; un essai
  instable se dit à Edouard avant d'en changer le statut. Chaque correction se commite, s'envoie et se relit ; la
  tâche se relit sur la tête verte.

### Tâche 14 : les essais des gestes

**Fichiers :** modifier `tests/commun/mod.rs`, ADR-0005, ADR-0006 ; créer `tests/essais_gestes.rs`.

**Interfaces :**
- Consomme : les aides de la Tâche 13.
- Produit (`tests/commun/mod.rs`) :
  ```rust
  /// Les boutons de la souris échangés (`SwapMouseButton(TRUE)`) le temps de la garde ; son `Drop` les remet, sans
  /// paniquer, essai en échec compris.
  pub struct BoutonsInverses;  impl BoutonsInverses { pub fn poser() -> BoutonsInverses }
  ```

- [ ] **Étape 1 : `tests/essais_gestes.rs`** :
  - `invoquer_un_bouton` : `invoquer --type Button --nom Départ` en 0, puis un événement `clic` d'id `depart`.
  - `invoquer_un_element_sans_le_geste_sort_en_3` : `invoquer --id inerte` : (3, « échec du système : l'élément
    n'offre pas le motif Invoke »).
  - `element_disparu_sort_en_2` : `choisir --id case-fugace` : (2, « l'élément a disparu ») : sa bascule retire la
    case de la page, et la sonde d'état relit un élément qui n'est plus. Un 3 dirait que la couche ne rend pas le
    code de l'élément disparu.
  - `choisir_coche_une_case` : `choisir --id case` en 0 ; un `change` d'id `case`, `etat` `true` ; `trouver --id case`
    porte `etat=coche` ; un second `choisir --id case` en 0, puis `CALME`, sans nouveau `change`.
  - `choisir_attend_une_case_lente` : `choisir --id case-lente` en 0 ; après `CALME`, un seul `change` d'id
    `case-lente` ; `trouver --id case-lente` porte `etat=coche`.
  - `choisir_un_bouton_radio_par_dans` : `choisir --type RadioButton --nom Ronde --dans Forme` en 0 ; le `change` d'id
    `forme-ronde` ; `trouver --id forme-ronde` porte `choisi` dans son `etat`.
  - `focaliser_porte_le_focus` : `focaliser --id zone` en 0 ; un `focus` d'id `zone` ; `focus --id zone --attendre
    3000` en 0 ; puis `focus` commence par `[Edit] <Zone> id=zone`.
  - `defilables_et_defiler` : `defilables` : une ligne porte `<Cadre des lignes>` ; `defiler --id cadre --a 100` en 0,
    puis un `defile` d'id `cadre`, valeur 99 au moins, et, après `CALME`, aucun `focus` d'id `cadre` ; le témoin :
    `focaliser --id cadre` en 0 donne un `focus` d'id `cadre`, si bien que le cadre pouvait le prendre ; `defiler --id
    inerte --a 50` : (3, « échec du système : l'élément n'offre pas le motif Scroll »).
  - `clic_clique_au_point` : `--position 100,100 --taille 800,600` ; `dpi` vaut 96, sinon panique « le runner n'est
    pas à 96 DPI : l'essai ne prouverait rien » ; `clic --x 350 --y 225` en 0 ; un `clic` d'id `cible` ;
    `position_du_pointeur()` égale l'origine de `zone_cliente` plus (350, 225).
  - `clic_au_bord_du_bureau_virtuel` : `dpi` vaut 96 ; l'application placée par `--position` pour que la dernière
    colonne du bureau virtuel (`x + l − 1` de `bureau_virtuel`) tombe dans sa zone cliente, à la hauteur des neuf
    dixièmes du bureau virtuel ; `fenetre_au_point` y rend l'application, sinon panique (« la barre des tâches ou une
    autre fenêtre couvre le point : l'essai ne prouverait rien ») ; `clic` au point de la zone cliente qui y tombe, en
    0 ; `position_du_pointeur()` égale le point visé, sur chaque axe. Il départage l'échelle de la normalisation
    (décision 11) : un pixel avant dit l'échelle de 65 535, et la formule change avec ses tests du domaine.
  - `boutons_inverses_cliquent_a_gauche` : `BoutonsInverses::poser()`, puis `GetSystemMetrics(SM_SWAPBUTTON)` non
    nul, sinon panique ; `clic --x 350 --y 225` en 0 ; un `clic` d'id `cible`, et, après `CALME`, aucun `droit` :
    Windows lit les boutons de `SendInput` comme les boutons physiques, et l'échange du domaine les remet.
  - `clic_droit_et_double` : `clic --x 350 --y 225 --droit` : un `droit` d'id `cible` ; `--double` : un `double` d'id
    `cible`.
  - `clic_refuse_un_point_recouvert` : `autre --au-dessus --position 700,500 --taille 200,150` ; un point de la zone
    cliente de l'application sous `autre` : (1, « point recouvert par une autre fenêtre »), et, après `CALME`, aucun
    `clic` dans l'une ni l'autre ; le témoin, `clic --x 350 --y 225`, en 0 avec son `clic`.
  - `clic_hors_de_la_zone_cliente_sort_en_1` : `clic --x -5 --y 10` : (1, « point hors de la zone cliente : -5,10 pour
    <l>×<h> »), `l` et `h` lus par `zone_cliente`.
  - `molette_defile_la_roulette` : `molette --x 400 --y 350 --crans -3` en 0, puis un `defile` d'id `roulette`, valeur
    au-dessus de 0 ; `--crans 3` : un `defile` d'id `roulette`, valeur 0.
- [ ] **Étape 2 : les ADR.** Les « Confirmation » nomment ces essais ; si `clic_au_bord_du_bureau_virtuel` a départagé
  l'échelle, ADR-0005 dit celle qui est mesurée.
- [ ] **Étape 3 : vérifier sur le poste**, comme à la Tâche 13.
- [ ] **Étape 4 : commit et envoi** : `git add tests/commun/mod.rs tests/essais_gestes.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md`,
  puis `git commit -m "test(essais): les gestes de l'automatisation et du pointeur de bout en bout"`, puis
  `git push origin feat/tranche2`.
- [ ] **Étape 5 : lire la CI**, comme à la Tâche 13. Si `clic_au_bord_du_bureau_virtuel` voit le pointeur un pixel
  avant le point visé, la formule de la décision 11 passe à l'échelle de 65 535 dans `src/domaine/pointeur.rs`, avec
  les valeurs attendues de `normalisation_vise_le_quart_du_pixel` et de `clic_envoie_un_seul_lot`, la mutation du
  module relancée, et le correctif se relit.

### Tâche 15 : KeePassXC, SystemSettings, les Paramètres et ArrowDown

**Fichiers :** modifier `tests/commun/mod.rs` (`Parametres::regle`), `tests/essais_gardes.rs`,
`tests/essais_clavier.rs`, ADR-0005, `docs/registre.md`.

**Interfaces :**
- Produit (`tests/commun/mod.rs`) :
  ```rust
  impl Parametres { /// « enfant SystemSettings », « AUMID des Paramètres », « ni enfant ni AUMID lisibles », ou
      /// « AUMID d'une autre application », lus maintenant, l'AUMID en STA comme `pilote` le lit (décision 2).
      pub fn regle(&self) -> &'static str }
  ```

- [ ] **Étape 1 : `tests/essais_gardes.rs`.** `copie_keepassxc_refuse_taper_et_touche` et la constante `KEEPASSXC`
  cèdent à `copie_keepassxc_garde_son_champ` : la copie `KeePassXC --focus motdepasse` ; `capture --fenetre
  KeePassXC --sortie <profil>/k.png` : (1, « capture refusée : fenêtre de KeePassXC »), le fichier absent ; `taper
  --fenetre KeePassXC --texte x` : (1, « KeePassXC : focus dans un champ de mot de passe ; 0 unités sur 1 sont
  parties ») ; `touche Ctrl+A` : (1, « KeePassXC : seules les touches de déplacement passent … ; 0 unités sur 1 sont
  parties ») ; `touche Gauche` en 0, avec son `touche` d'id `motdepasse` ; après `CALME`, aucune `saisie` d'id
  `motdepasse`. Puis `--focus zone` : `taper --texte "a\tsecret"` : (1, « KeePassXC : un texte qui porte une
  tabulation ou un saut de ligne est refusé »), aucune frappe ; `touche Tab` en 0 ; un `focus` d'id `motdepasse` ;
  `taper --texte s` : le refus du champ. Puis `--focus champ` : `taper --texte ab` en 0, `ab` saisi dans `champ`.
  `copie_systemsettings_refusee` gagne `arbre --fenetre SystemSettings` et `capture --fenetre SystemSettings
  --sortie …` : (1, « fenêtre refusée : processus SystemSettings »), le fichier absent.
  `parametres_reduits_restent_refuses` écrit `Parametres::regle` sur la sortie d'erreur, lue juste avant `activer`,
  et exige : code 1 et le message de SystemSettings si la règle est l'une des trois qui refusent ; code 2 sans les
  chevrons du titre sinon, la règle dite.
- [ ] **Étape 2 : `tests/essais_clavier.rs`** : `touche_bas_donne_arrowdown` : `--focus zone`, `touche Bas` en 0, puis
  un `touche` dont `key` et `code` valent `ArrowDown`.
- [ ] **Étape 3 : ADR-0005.** « Confirmation » nomme `copie_keepassxc_garde_son_champ` sous la règle du champ de
  KeePassXC et sous la capture refusée de ses fenêtres, `copie_systemsettings_refusee` et
  `parametres_reduits_restent_refuses` sous la liste de refus.
- [ ] **Étape 4 : vérifier sur le poste**, comme à la Tâche 13.
- [ ] **Étape 5 : commit et envoi** : `git add tests/commun/mod.rs tests/essais_gardes.rs tests/essais_clavier.rs docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md`,
  puis `git commit -m "test(essais): KeePassXC, SystemSettings, les Paramètres et ArrowDown de bout en bout"`, puis
  `git push origin feat/tranche2`.
- [ ] **Étape 6 : lire la CI**, comme à la Tâche 13.

### Tâche 16 : le jeton restreint

**Fichiers :** modifier `tests/commun/mod.rs`, `tests/essais_gardes.rs`, `.github/workflows/ci.yml`, ADR-0005.

**Interfaces :**
- Produit (`tests/commun/mod.rs`, chaque fonction publique commençant par `exiger_la_ci`) :
  ```rust
  /// La valeur de `SE_GROUP_INTEGRITY` (`windows`, `Win32_System_SystemServices`, une feature que le dépôt ne prend
  /// pas) : l'attribut du `TOKEN_MANDATORY_LABEL` que reçoit `SetTokenInformation(TokenIntegrityLevel)`.
  const SE_GROUP_INTEGRITY: u32 = 0x20;
  pub enum Jeton { Restreint, Abaisse }
  /// `pilote` lancé par CreateProcessAsUserW sous un jeton tiré de celui des essais (décision 16) ; le profil
  /// marqué au niveau moyen ; ses sorties dans `restreint-sortie.txt` et `restreint-erreur.txt` du profil ; 30 s au plus.
  pub fn pilote_restreint(profil: &Profil, args: &[&str], jeton: Jeton) -> Sortie;
  pub fn niveau_du_processus(pid: u32) -> u32;   pub fn niveau_des_essais() -> u32;   // paniquent s'ils ne se lisent pas
  ```
  Avant de lancer, `pilote_restreint` vérifie : `niveau_des_essais() == 0x3000` (sinon panique « le runner n'est pas
  élevé : l'essai ne prouverait rien ») ; le jeton fait au niveau 0x2000 ; pour `Restreint`, le groupe Administrateurs
  inactif (`CheckTokenMembership` sur un jeton d'emprunt tiré du jeton fait).

- [ ] **Étape 1 : `tests/essais_gardes.rs`**, dans cet ordre :
  - `pilote_restreint_lit_les_fenetres` : l'application ; `pilote_restreint(…, ["fenetres"], Jeton::Restreint)` en 0,
    une ligne `processus=appli-essai` ; il prouve le lancement, le bureau, le journal et l'énumération, sans verrou ni
    niveau.
  - `refuse_une_fenetre_plus_elevee` : l'application (niveau des essais, 0x3000), `pilote_restreint(…, ["touche",
    "--fenetre", "appli-essai", "Tab"], Jeton::Restreint)` : (1, « fenêtre plus élevée que pilote ») ; après
    `CALME`, aucune frappe ; le journal du profil porte la ligne `touche` au code 1.
  - `refuse_une_fenetre_plus_elevee_au_jeton_abaisse` : de même avec `Jeton::Abaisse`.
  Chaque essai écrit sur sa sortie d'erreur le code, le message et le journal du profil restreint avant de juger.
- [ ] **Étape 2 : la CI.** L'étape de dépôt ajoute `…/*/restreint-sortie.txt` et `…/*/restreint-erreur.txt`, avant
  les deux exclusions, et son commentaire les nomme.
- [ ] **Étape 3 : ADR-0005.** « Confirmation » nomme les trois essais sous « Le niveau d'intégrité se juge par
  comparaison », et dit ce que prouve chaque variante (décision 16).
- [ ] **Étape 4 : vérifier sur le poste**, comme à la Tâche 13.
- [ ] **Étape 5 : commit et envoi** : `git add tests/commun/mod.rs tests/essais_gardes.rs .github/workflows/ci.yml docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md`,
  puis `git commit -m "test(essais): le refus d'une fenêtre plus élevée sous un jeton restreint"`, puis
  `git push origin feat/tranche2`.
- [ ] **Étape 6 : lire la CI.** `gh pr checks`. Tout autre résultat que (0) pour `fenetres` et (1, « fenêtre plus
  élevée que pilote ») pour les deux refus — « niveau d'intégrité illisible », « journal inaccessible », « poste
  verrouillé », le verrou, un autre code, un lancement refusé — arrête l'exécutant : il le dit à Edouard, avec la
  sortie d'erreur de l'essai, et ne change aucune garde pour que l'essai passe (décision 16). Un essai instable : le
  dire à Edouard avant d'en changer le statut.

### Tâche 17 : les délais, les ADR relus, le registre, `CLAUDE.md` et la revue de la branche

**Fichiers :** modifier `.github/workflows/ci.yml`, `docs/decisions/0005-…`, `docs/decisions/0006-…`,
`docs/registre.md`, `CLAUDE.md`.

- [ ] **Étape 1 : les délais.** Sur la tête, CI verte, lire les passages à froid de `windows` et de `mutants`
  (`gh run view <id> --json jobs`) ; chaque délai devient deux fois le plus long, en minutes entières par excès, avec
  son commentaire (« deux fois N min M s, le passage à froid du <date> ») ; celui de `mutants` n'est jamais sous
  23 min (deux fois 11 min 12 s, le passage du 22/11/2026). `mutants-complet.yml` n'est pas touché (décision 17).
- [ ] **Étape 2 : les ADR relus.** Chaque ligne de la section « Ce que ce plan précise ou change du plan maître et des
  ADR » se retrouve, au présent, dans le corps d'ADR-0005 ou d'ADR-0006, et dans leur sous-section de la tranche 2 ;
  chaque test de « Confirmation » existe à son chemin ; aucun texte hors des plans ne dit encore « d'ici la tranche
  2 », « aux tranches suivantes » pour ce que la tranche livre, « pas encore livrée : focus », « valeur refusée :
  champ de mot de passe », ni le message de KeePassXC de la tranche 1 : `git grep -n -i '<mot>' -- . ':!.claude/plans'`
  pour chacun, les plans citant ces textes jusqu'à la clôture du lot, qui les supprime.
- [ ] **Étape 3 : le registre.** LE-1 : état « clos », `touche_bas_donne_arrowdown` vert dans `windows` ; une leçon de la
  tranche qui dépasse son code, par `/consigner`.
- [ ] **Étape 4 : `CLAUDE.md`.** L'état : « Tranche 2 du lot 1 faite : la lecture, les gestes et la capture. Prochaine :
  la tranche 3 du lot 1. »
- [ ] **Étape 5 : commit et envoi** : `git add .github/workflows/ci.yml docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md docs/registre.md CLAUDE.md`,
  puis `git commit -m "docs: ADR-0005 et ADR-0006 relus, CLAUDE.md, registre et délais de la CI"`, puis
  `git push origin feat/tranche2`.
- [ ] **Étape 6 : la revue de la branche entière**, par une session neuve, sur la tête commitée : chaque décision de ce
  plan retrouvée dans le code et dans les ADR, chaque test nommé présent, chaque critère de sortie de la tranche
  rempli (les essais verts dans `windows`, `mutants` sans survivant, `PrintWindow` prouvé sur WebView2 par
  `capture_relue`, le refus d'une fenêtre plus élevée prouvé ou son repli accepté et déclaré, les délais remesurés),
  chaque mot que la tranche périme cherché dans tout le dépôt. Ses constats confirmés se corrigent, se commitent et se
  relisent sur la nouvelle tête.
- [ ] **Étape 7 : la PR quitte le brouillon** (`gh pr ready`), la revue faite, ses corrections relues, `gh pr checks`
  tout vert.

---

## La PR de la tranche

Elle dit « Partie de #2 », nomme les dépendances nouvelles (`png` 0.18.1, `unicode-normalization` 0.1.26, et les
crates qu'elles tirent) et les trois features de `windows`, et porte « ## Écarts à la spec », chaque écart sous la
forme de la méthode :

- les précisions de la tranche 1 dont la tranche 2 livre le code (décision 20 du plan maître) : ce qu'on lit borné à
  W, la règle de KeePassXC étendue à `touche`, la sortie (le vocabulaire et les échappements d'une ligne d'élément,
  `motdepasse=oui`, la liste d'`etat`), les codes (au 1, `valeur` sur un mot de passe, le point de clic ou de molette,
  les refus de KeePassXC ; au 3, le délai de l'automatisation), les bornes (`--max`, `--a`, `--x` et `--y`), le
  sélecteur, l'attente et les options de `focus` (promesse : ADR-0002, « Ce qu'on lit reste dans la fenêtre visée »
  et « KeePassXC » ; ADR-0004, « La sortie », « Les codes de sortie », « Le sélecteur » et « L'attente ») — accord :
  21/11/2026 ;
- chaque ligne de la section « Ce que ce plan précise ou change du plan maître et des ADR » (promesse : la décision
  du plan maître ou la section d'ADR qu'elle cite) — accord : la date à laquelle Edouard accepte ce plan ;
- le repli de la décision 14, seulement s'il sert (promesse : issue #2, critères d'acceptation) — accord : la date
  qu'Edouard donne.

Son « Reste ouvert » : la disposition illisible rendue en `Option` (plan de la tranche 3) ; `--hauteur` qui exige
`--largeur` (tranche 3) ; toute leçon de la revue de la branche qui n'aurait pas sa tâche. Après la fusion :
`gh workflow run mutants-complet.yml`, sa durée notée ; son délai (25 min, Edouard, 28/11/2026) relevé par une PR si
deux fois cette durée le dépasse. S'il est annulé à son délai, sans mesure, la session le dit à Edouard, avec la
durée du job `mutants` de la PR, qui a muté le diff de la tranche, pour qu'il décide du délai ; aucune session ne le
change sans lui.
