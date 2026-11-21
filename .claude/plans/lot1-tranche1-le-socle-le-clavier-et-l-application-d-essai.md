# Tranche 1 · le socle, le clavier et l'application d'essai — plan d'implémentation

> **Pour les agents :** compétence requise : `superpowers:subagent-driven-development` (recommandée) ou
> `superpowers:executing-plans`, tâche par tâche. Les étapes se cochent (`- [ ]`). Chaque tâche suit le test
> d'abord : le test écrit, lancé et vu rouge, puis le code, puis le test vert, puis le commit. Sur le poste, ne
> lancer ni `pilote`, ni `appli-essai`, ni `cargo test` avec `--ignored` ou `--include-ignored` : `cargo build`,
> `cargo clippy`, `cargo fmt` et `cargo test` sans ces options sont permis.

Plan maître : `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md`

**But :** un `pilote` qui tape, sûr dès le premier jour : la fenêtre retenue, la liste de refus, le premier plan
vérifié avant chaque unité, le verrou des gestes, le journal ; l'application d'essai ; et la CI qui le prouve sur
WebView2.

**Architecture :** l'espace de travail du plan maître (décisions 3 et 4). Le domaine, `src/domaine/`, pur, décide
de tout contre le trait `Bureau`, et se teste sous Linux sur le faux bureau. La couche Windows, `src/bureau/`, lit
et agit sans décider. `src/systeme.rs` ouvre le journal, lit l'horloge, la date et l'environnement. L'application
d'essai, `essai/appli/`, est un membre à part.

**Pile :** celle du plan maître. Pour `pilote` : `clap` 4.6.7 (`derive`), et `windows` 0.62.2 sous Windows avec les
features `Win32_Foundation`, `Win32_Graphics_Dwm`, `Win32_Security`, `Win32_Storage_EnhancedStorage`,
`Win32_System_Com`, `Win32_System_Com_StructuredStorage`, `Win32_System_Diagnostics_ToolHelp`,
`Win32_System_RemoteDesktop`, `Win32_System_StationsAndDesktops`, `Win32_System_SystemInformation`,
`Win32_System_Threading`, `Win32_System_Variant`, `Win32_UI_HiDpi`, `Win32_UI_Input_KeyboardAndMouse`,
`Win32_UI_Shell`, `Win32_UI_Shell_PropertiesSystem`, `Win32_UI_WindowsAndMessaging`. Pour `appli-essai`, sous
Windows : `wry` 0.57.1, `tao` 0.37.2, et `windows` 0.62.2 (`Win32_Foundation`, `Win32_Security`,
`Win32_System_Threading`, `Win32_UI_WindowsAndMessaging`), que wry et tao tirent déjà à cette version : une
dépendance nommée dans la PR. `png` et `unicode-normalization` attendent la tranche 2.

**Spec :** le plan maître et ses décisions 1 à 20 ; [le cadrage](../../docs/cadrage.md), questions 13, 14 et 16 ;
les ADR 0001 à 0004 ; le design du 2026-11-21, hors du dépôt, que le brief reprend.

---

## Les décisions de ce plan

Le plan maître décide ; ce plan précise ce qu'il laissait à la tranche 1. **Les décisions 1, 2, 3, 11 et 14
changent le plan maître**, d'après les sondes de la préparation et la relecture de ce plan : la PR de ce plan les
déclare, et Edouard les accepte avant l'exécution. La décision 3 est aussi un écart à la spec.

1. **Le niveau d'intégrité se juge par comparaison.** Sur le poste, le jeton filtré de la session ouvre le jeton
   d'une cible élevée du même compte (sonde en lecture seule du 2026-11-21) : « illisible » n'arrive que pour un
   autre compte. La garde refuse si le niveau de la cible dépasse celui de `pilote` (« fenêtre plus élevée que
   pilote »), ou si l'un des deux ne se lit pas (« niveau d'intégrité illisible »). L'essai de la décision 14 du
   plan maître, à la tranche 2, attendra donc le premier message.
2. **La disposition du clavier est celle du fil focalisé**, lue par `GetGUIThreadInfo(0)` (le fil de `hwndFocus`,
   à défaut de `hwndActive`, puis de la fenêtre au premier plan), puis `GetKeyboardLayout` de ce fil. Dans une
   WebView2, ce fil est celui de `msedgewebview2`. Ce HKL va à `VkKeyScanExW`, `ToUnicodeEx` et `MapVirtualKeyExW`.
3. **Le refus de KeePassXC, d'ici la tranche 2, est entier :** `touche` et `taper` refusent tout geste quand P, ou
   le processus d'une fenêtre enfant de W, est KeePassXC (« KeePassXC : touche et taper sont refusés dans cette
   version »), tant que la garde de son champ de mot de passe, qui demande l'automatisation, n'existe pas. Il se
   juge avec la liste de refus, juste après la fenêtre retenue et avant le verrou, puis de nouveau après
   l'activation. C'est plus strict que la décision 6 du plan maître et qu'ADR-0002 (« KeePassXC »), qui
   permettent ces gestes hors du champ de mot de passe : un écart à la spec, déclaré dans la PR de la tranche avec
   pour accord la date à laquelle Edouard accepte ce plan. La tranche 2 rétablit la règle du champ. Le test
   `keepassxc_par_explorer_refuse_taper` tient ici la place de `keepassxc_par_explorer_refuse_la_capture`, qui
   arrive avec la capture.
4. **Toutes les sous-commandes s'analysent**, bornes comprises (décision 11 du plan maître). Celles que la tranche
   ne livre pas sortent en 64, « pas encore livrée : <sous-commande> », avec leur ligne au journal.
5. **L'analyse et le journal.** L'analyse comprend clap, puis, pour `touche`, les touches interdites (1) et
   inconnues (64), puis, pour `taper`, le texte lu, décodé et borné quelle que soit sa source (64) : rien de cela
   ne lit une fenêtre. Une commande valide exige ensuite le journal (1 sinon), avant toute lecture de fenêtre et
   tout geste. L'aide, la version et un refus de l'analyse écrivent leur ligne si le journal s'ouvre, sans que son
   échec change le code. La liste fermée des sous-commandes porte les 21 du lot, plus `aide` et `version` ; tout
   autre premier mot donne `inconnue`. Une ligne qui ne parvient pas au journal après un geste fait sortir en 3,
   « échec du système : le journal n'a pas reçu sa ligne ». Une panique se rattrape dans `executer` : sa ligne va
   au journal au code 3, et l'outil écrit la seule ligne « échec du système : panique ».
6. **Les messages** que les tests vérifient, mot pour mot (ADR-0006 les reprend) :

   | Code | Message |
   |---|---|
   | 1 | `fenêtre refusée : processus <nom>` |
   | 2 | `aucune fenêtre de « <nom> »` |
   | 2 | `plusieurs fenêtres de « <nom> » : <titre> <titre> ; --titre ou --pid départage` |
   | 1 | `un autre geste est en cours depuis 5 s` |
   | 1 | `poste verrouillé` |
   | 1 | `fenêtre plus élevée que pilote` |
   | 1 | `niveau d'intégrité illisible` |
   | 1 | `touche de modification enfoncée` |
   | 1 | `Windows a refusé le premier plan` |
   | 1 | `le premier plan a changé : <k> unités sur <n> sont parties` |
   | 1 | `touche interdite : <touche>` |
   | 64 | `touche inconnue : <touche>` |
   | 1 | `caractère absent du clavier : « <c> »` |
   | 1 | `touche morte : « <c> »` |
   | 1 | `Verr. Maj est allumée` |
   | 1 | `KeePassXC : touche et taper sont refusés dans cette version` |
   | 1 | `journal inaccessible : ouvert dans une autre application ?` |
   | 64 | `pas encore livrée : <sous-commande>` |
   | 64 | `texte illisible : ni UTF-8 ni UTF-16 à BOM` |
   | 64 | `texte trop long : <n> caractères, 10000 au plus` |
   | 64 | `fichier illisible : <chemin>` |
   | 64 | `entrée standard illisible` |
   | 64 | `option inconnue : <arg>` |
   | 64 | `option requise : <arg>` |
   | 64 | `valeur manquante : <arg>` |
   | 64 | `valeur refusée : <arg>` |
   | 64 | `options incompatibles : <arg>` |
   | 64 | `trop de valeurs : <arg>` |
   | 64 | `sous-commande inconnue : <arg>` |
   | 64 | `sous-commande manquante` |
   | 64 | `usage : <première ligne de clap>` |
   | 3 | `échec du système : <ce qui a échoué>` |

   Chaque message part sur la sortie d'erreur, en une ligne, ses noms, ses titres, ses chemins et ses caractères
   échappés (décision 7 du plan maître) ; `<c>` est le caractère échappé.
7. **Les sorties de la tranche :** `fenetres` écrit une ligne par fenêtre, `[Fenetre] <titre> processus=<nom>
   pid=<pid>`, plus `premier-plan=oui` pour celle qui l'a, triées par nom de processus, puis PID, puis titre ;
   `activer` écrit la ligne de la fenêtre retenue ; `touche` et `taper` n'écrivent rien sur la sortie standard.
8. **L'activation** (question 13) : restaurer une fenêtre réduite, puis sonder qu'elle ne l'est plus (100 ms,
   2 s au plus ; sinon 1, « Windows a refusé le premier plan ») ; rien si le premier plan est accepté ; sinon un
   mouvement de souris nul, la demande du premier plan, une sonde (100 ms, 1 s au plus) ; sinon, en un seul envoi,
   Alt gauche enfoncée, la touche virtuelle non attribuée 0xE8 enfoncée puis relâchée, Alt relâchée ; une sonde
   jusqu'à ce qu'aucune touche de modification ne soit enfoncée (10 ms, 500 ms au plus ; sinon 1, « touche de
   modification enfoncée », sans demande) ; la demande ; une sonde (100 ms, 1 s au plus) ; sinon 1, « Windows a
   refusé le premier plan ».
9. **Un caractère tapé par la disposition** (`--comme-au-clavier`, `barre`, `interrogation`) : `\n` et `\t`
   donnent Entrée et Tab, sans modificateur ; sinon `VkKeyScanExW`. Il refuse (« caractère absent du clavier ») :
   le résultat -1, un caractère hors du plan de base, un état qui porte Ctrl sans Alt (2, 3) ou Alt sans Ctrl
   (4, 5), ou l'un des bits 8, 16 ou 32 ; puis, par `ToUnicodeEx` (drapeau 4, l'état du clavier intact), un
   produit autre que le seul caractère demandé ; une touche morte (un résultat négatif) refuse en « touche
   morte ». Maj est Maj gauche ; l'état 6 (AltGr) envoie Ctrl gauche puis Alt droite, l'état 7 Maj gauche, Ctrl
   gauche puis Alt droite ; puis la touche, puis les relâchements dans l'ordre inverse. Pour `taper
   --comme-au-clavier` seulement (décision 8 du plan maître), Verr. Maj se lit après les touches enfoncées et
   avant l'activation (1, « Verr. Maj est allumée ») ; `touche barre` et `touche interrogation` n'y sont pas
   soumis. Tous les lots d'un `taper --comme-au-clavier` se calculent après l'activation et avant le premier
   envoi : un caractère refusé ne laisse partir aucun caractère.

   Chaque événement de touche, d'un caractère comme d'une combinaison, porte la touche virtuelle et son code de
   balayage (`MapVirtualKeyExW`, `MAPVK_VK_TO_VSC_EX`) ; il est étendu quand l'octet haut vaut 0xE0, ou quand la
   touche virtuelle est l'une de 0x21 à 0x28, 0x2D ou 0x2E (PageHaut, PageBas, Fin, Debut, Gauche, Haut, Droite,
   Bas, Inser, Suppr), dont `MapVirtualKeyExW` rend le code du pavé numérique, sans 0xE0 (relevé du poste en
   lecture seule : VK_UP donne 0x0048 ; seuls Ctrl droite et Alt droite portent 0xE0).
10. **Le poste verrouillé** se juge sur trois faits lus : le nom du bureau d'entrée (`OpenInputDesktop`), puis les
    drapeaux et l'état de la session (`WTSQuerySessionInformationW`, `WTSSessionInfoEx`). Le poste est verrouillé
    si le bureau d'entrée n'est pas `Default` ou ne se lit pas ; si les drapeaux lus disent « verrouillé » ; si
    l'état lu n'est pas « actif ». Des drapeaux inconnus (-1) ou illisibles, et un état illisible, ne
    verrouillent pas à eux seuls : les autres faits lus décident.
11. **Le premier plan accepté** resserre la décision 5 du plan maître, qui acceptait toute fenêtre de P : la
    fenêtre au premier plan est W ; ou la chaîne de ses propriétaires mène à W, et elle appartient à un processus
    de l'arbre de P que ni la liste de refus ni la règle de KeePassXC ne refusent. Une autre fenêtre de P, non
    possédée par W, n'est pas acceptée : la frappe ne part jamais dans un autre dialogue ou une autre fenêtre du
    même processus. Une fenêtre au premier plan nulle n'est jamais acceptée. Pour une application Tauri, le
    premier plan est la fenêtre de premier niveau, W, et rien ne change.
12. **L'application d'essai** reçoit `<dossier WebView2> <fichier d'événements>`, puis, au choix :
    - `--dialogue` : une seconde fenêtre titrée « Dialogue d'essai », sans propriétaire, dès le départ ;
    - `--reduite` : la fenêtre principale réduite au départ ;
    - `--focus <id>` : l'élément qui a le focus au chargement (`depart` sinon) ;
    - `--permettre-premier-plan` : `AllowSetForegroundWindow(ASFW_ANY)` au départ et à chaque `keydown` reçu, le
      droit tombant à chaque entrée ; le résultat de chaque appel s'écrit dans le fichier d'événements ;
    - `--voler-premier-plan <fichier d'événements de la cible> <n>` : la fenêtre, cachée au départ, sonde ce
      fichier toutes les 20 ms ; dès sa n-ième ligne `saisie`, elle se montre et demande le premier plan toutes
      les 20 ms jusqu'à l'avoir (5 s au plus), puis écrit la ligne `vole` ;
    - `appli-essai --tenir-le-verrou <fichier>`, un mode à part, reconnu en premier argument avant toute lecture
      de `<dossier> <fichier>` : sans fenêtre, prend `Local\pilote-gestes`, écrit `verrou pris` dans le fichier
      dès que l'attente rend `WAIT_OBJECT_0` ou `WAIT_ABANDONED`, et le garde jusqu'à sa mort.

    La fenêtre principale s'intitule « Appli d'essai ».
13. **La cadence de la frappe :** après chaque unité, `pilote` attend 5 ms (`CADENCE`) avant de vérifier le premier
    plan pour la suivante. Sans elle, des milliers d'événements partiraient en une fraction de seconde, resteraient
    en file, et iraient à une fenêtre qui aurait volé le premier plan après le dernier contrôle. 10 000 caractères
    prennent ainsi 50 s au moins.
14. **Des essais changent de forme ou de nom.**
    - Les appels de `pilote` depuis PowerShell 5.1 et depuis Git Bash, que la décision 16 du plan maître rangeait
      en étapes du job `windows`, deviennent des essais de bout en bout qui lancent ces shells : chaque appel y
      reçoit son profil jetable, et le résultat se juge sur le fichier d'événements et sur ce que le script
      recueille.
    - La sortie en UTF-8 sans BOM, que le plan maître éprouvait depuis les deux shells (Effets cherchés ; Outils,
      PowerShell 5.1 et la page OEM), se juge sur les octets reçus sans shell
      (`sortie_et_erreurs_en_utf8_sans_bom`), sur ceux que Git Bash écrit dans un fichier, et dans PowerShell 5.1
      réglé en UTF-8 par `[Console]::OutputEncoding`. Ce que PowerShell 5.1 décode par défaut, dans la page OEM,
      passe à la compétence de la tranche 4, qui dira comment régler la console (le cadrage le relève).
    - L'essai du verrou abandonné, qui devait tuer un `pilote` tenant le verrou (Outils du plan maître), tue
      `appli-essai --tenir-le-verrou`, qui dit quand il le tient.
    - Le vol du premier plan, que le plan maître (Points de vigilance) faisait partir sur une minuterie, part de la
      n-ième saisie reçue par la cible, qui renouvelle `AllowSetForegroundWindow(ASFW_ANY)` à chaque `keydown` :
      une minuterie tombait avant la frappe, et le droit tombe à la première touche.
    - Deux tests du plan maître changent de nom : `statique_cherche_des_noms_entiers` devient
      `cherche_des_noms_bornes` (le contrôle cherche aussi des préfixes), `poste_verrouille_refuse_en_1` devient
      `poste_verrouille_par_cas` (un tableau de cas, dont des cas non verrouillés).
    - `systeme::dossier_temporaire`, que le plan maître rangeait à la tranche 1, attend la tranche 3, qui seule
      s'en sert.

## Contraintes générales

Celles du plan maître valent toutes. En plus :

- Le domaine n'emploie ni `Instant`, ni `SystemTime`, ni `sleep`, ni `std::fs`, ni `std::env`, ni
  `std::process` : le temps est `Moment` et `Duree` (des millisecondes), et toute lecture passe par `Bureau`.
- Chaque lot d'événements relâche tout ce qu'il enfonce, dans l'ordre inverse ; `pilote` ne frappe jamais Alt
  seule, ni aucune touche hors d'un lot qu'il a formé.
- L'application d'essai n'appelle jamais `Window::set_focus()` de tao, qui injecte Alt par `SendInput`.
- Chaque refus d'avant l'activation se vérifie sans `Restaurer`, `Demande` ni `Envoi` au faux bureau.
- À la fin des Tâches 2 à 8, si `cargo mutants --version` répond 27.1.0 sur le poste,
  `cargo mutants -p pilote -f src/domaine/<module>.rs` ne laisse aucun survivant : un survivant se tue par un
  test, un mutant équivalent ou qui boucle s'exclut par son nom dans `.cargo/mutants.toml`, avec sa raison, et le
  commit de la tâche emporte ce fichier. Sinon, le job `mutants` de la PR en tient lieu (Tâche 11, étape 7).
- À la fin de chaque Tâche 2 à 11, avant son commit : `cargo fmt --all`, puis `cargo fmt --all --check` et
  `cargo clippy --workspace --all-targets -- -D warnings` sans faute. Les tests du plan sont écrits en forme
  compacte, que rustfmt réécrit.

## Points de vigilance

- Une fenêtre qui vole le premier plan au milieu d'une frappe : la frappe s'arrête, le code est 1, et le message
  compte les unités parties — `taper_s_arrete_quand_le_premier_plan_change` (Tâche 8) et
  `taper_refuse_quand_une_autre_fenetre_vole_le_premier_plan` (Tâche 11).
- Une touche de modification que l'utilisateur tient au départ, ou qu'un lot laisserait enfoncée : refus, et
  chaque lot relâche ce qu'il enfonce — `refuse_si_un_modificateur_est_enfonce` (Tâche 8) et
  `chaque_lot_relache_ce_qu_il_enfonce` (Tâche 4).
- Un dialogue titré au premier plan, puis une liste ouverte : sans `--titre`, le code 2 nomme les deux titres ;
  avec `--titre`, la frappe va à la fenêtre nommée, jamais au dialogue ; la liste ouverte ne compte pas —
  `deux_fenetres_titrees_sans_option_titre_sortent_en_2` et `premier_plan_refuse_une_autre_fenetre_de_p`
  (Tâche 7), leurs essais et `touche_avec_une_liste_ouverte_vise_la_fenetre_principale` (Tâche 11).
- Un texte qui porte un CRLF, une tabulation, un caractère hors du plan de base ou une lettre décomposée —
  `taper_crlf_donne_un_seul_entree` et `taper_hors_du_plan_de_base_envoie_la_paire` (Tâches 5 et 11),
  `lettre_decomposee_part_telle_quelle` (Tâche 5).
- Le journal ouvert dans une autre application : refus avant tout geste —
  `journal_inaccessible_refuse_avant_tout_geste` (Tâche 8) et `journal_verrouille_refuse_avant_tout_geste`
  (Tâche 11).

## Mécanismes annoncés

- `.github/workflows/ci.yml:18` — `socle:` — le job du socle garde sa clé ; la Tâche 1 ajoute `verifier` et `mutants`, la Tâche 11 `windows`, à côté de lui
- `.github/workflows/ci.yml:22` — `verifier` — le nom que le modèle réserve au job des vérifications du projet
- `.gitignore:5` — `.superpowers/` — ni `target/` ni `mutants.out*/` n'y sont : la Tâche 1 les ajoute
- `CLAUDE.md:15` — `Aucune avant la première PR d'implémentation` — la Tâche 12 y écrit les commandes
- `docs/decisions/README.md:5` — `Un ADR ne se réécrit pas` — ADR-0005 et ADR-0006 remplacent ADR-0002 et ADR-0004, qui ne changent que de statut
- `docs/decisions/0002-garde-fous-dans-le-code.md:32` — `La fenêtre visée` — la règle que la décision 5 du plan maître et la décision 11 de ce plan resserrent, reprise dans ADR-0005
- `docs/decisions/0002-garde-fous-dans-le-code.md:47` — `KeePassXC` — la règle que la décision 3 de ce plan suspend jusqu'à la tranche 2
- `docs/decisions/0004-interface-sortie-codes-et-selecteur.md:33` — `Les codes de sortie` — le tableau que la décision 8 du plan maître étend, repris dans ADR-0006
- `docs/cadrage.md:136` — `ne frappe jamais Alt seule` — l'ordre de l'activation, que la décision 8 de ce plan détaille
- `docs/cadrage.md:148` — `Un seul geste à la fois` — le verrou des gestes, 5 s d'attente puis le refus
- `.github/workflows/modele-socle.yml:142` — `--neuf` — ce plan, ajouté, se vérifie en entier, ses lieux ouverts
- `pupitre:.github/workflows/ci.yml:226-245` — `La sentinelle d'I30` — la sentinelle que la Tâche 11 reprend, `PILOTE_SENTINELLE` et `cargo test --workspace -- --include-ignored --test-threads=1` changés
- `pupitre:.github/workflows/ci.yml:298` — `--in-diff` — le job `mutants` ne mute que les lignes de la PR
- `pupitre:.github/workflows/mutants-complet.yml:7` — `cron` — le passage complet du lundi
- `pupitre:app/src-tauri/tests/statique.rs:93` — `fn controler` — la forme du contrôle statique, que la Tâche 1 reprend par mots bornés
- `pupitre:app/src-tauri/tests/statique.rs:66` — `HORLOGE` — les mots de l'horloge, refusés de même
- `pupitre:app/src-tauri/src/systeme.rs:91-102` — `starts_with` — la forme du garde-fou positif, que la Tâche 9 reprend pour le journal
- `pupitre:crates/stockage/tests/commun/mod.rs:54` — `fn dossier_jetable` — la seule fonction de dossier jetable des essais
- `pupitre:app/ui/src/recherche.rs:164` — `touche == "/"` — le prédicat que `touche barre` doit satisfaire, et que la page d'essai reprend : la touche `/`, sans Ctrl, Alt ni Méta, hors d'un champ

## Outils et cas limites

- `clap`, par `try_parse_from` — `--help` et `--version` arrivent en erreur, et une erreur d'usage sortirait en 2 — `aide_et_version_sortent_en_0` et `usage_inconnu_sort_en_64`, Tâche 3
- `clap`, dérivé — une sous-commande requise ajoute `arg_required_else_help` : `pilote` seul rend `DisplayHelpOnMissingArgumentOrSubcommand`, jamais `MissingSubcommand` — `sans_sous_commande_sort_en_64`, Tâche 3
- `clap` — une option sans sa valeur arrive en `InvalidValue` à valeur vide — `valeur_manquante_se_dit`, Tâche 3
- `clap` — une valeur qui commence par un tiret passe pour une option, sauf `allow_negative_numbers` ou `allow_hyphen_values` — `molette_accepte_des_crans_negatifs` et `taper_accepte_un_texte_qui_commence_par_un_tiret`, Tâche 3
- `cargo`, dans un espace de travail à paquet racine — sans `--workspace`, seul `pilote` se construit, se lint et se teste — chaque job passe `--workspace`, et `lancer_appli` panique si `appli-essai.exe` manque, Tâches 1 et 11
- `cargo test` — un seul filtre avant `--`, plusieurs après — les commandes « voir rouge » des Tâches 2 à 8
- `cargo mutants --in-diff` — un diff sans `.rs` du domaine sort en 0, un mutant qui boucle en 3 — le job `mutants` de la PR, lu à la Tâche 11
- `git`, lu par `build.rs` — un commit change la référence de la branche, pas `HEAD`, une référence compactée n'a pas de fichier, et cargo-mutants construit dans une copie sans `.git` — `version_porte_le_commit_ou_inconnu`, Tâche 3 ; `version_porte_le_commit` et l'étape « La version suit le commit » du job `windows`, Tâche 11
- le contrôle statique — un mot entier ne voit pas ses variantes (`GetTempPathW`), une sous-chaîne voit `Instant` dans `Instantane` — `cherche_des_noms_bornes` et `chaque_mot_refuse_a_sa_plantation`, Tâche 1
- `Get-ChildItem -Recurse -Force` — un fichier caché ne se voit qu'avec `-Force` — l'essai positif de la sentinelle, Tâche 11
- `actions/upload-artifact` — un chemin vide n'échoue pas avec `if-no-files-found: warn` — l'étape de dépôt, sous `%RUNNER_TEMP%\pilote-essais`, Tâche 11
- `SHGetKnownFolderPath` — il développe `USERPROFILE`, et échoue si `AppData\Local` manque — `journal_tombe_dans_la_sentinelle`, Tâche 9, et `journal_tombe_dans_le_profil_redirige`, Tâche 11
- `std::fs::OpenOptions`, en ajout — un fichier ouvert sans partage par une autre application refuse l'ouverture — `journal_verrouille_refuse_avant_tout_geste`, Tâche 11
- `std::panic::catch_unwind` — le crochet par défaut écrit d'abord plusieurs lignes sur la sortie d'erreur — `panique_sort_en_3_avec_sa_ligne`, Tâche 8, et le crochet muet de `lancer`, Tâche 9
- `SetForegroundWindow` — Windows refuse sans le dire, et Alt seule ouvre le mode menu d'une fenêtre Win32 — `activer_essaie_le_mouvement_nul_puis_alt_entourant` et `activer_ne_frappe_jamais_alt_seule`, Tâche 8 ; `activer_puis_taper_ne_perd_aucun_caractere`, Tâche 11
- `AllowSetForegroundWindow(ASFW_ANY)` — le droit tombe à chaque entrée du clavier — `taper_refuse_quand_une_autre_fenetre_vole_le_premier_plan`, qui le renouvelle à chaque `keydown`, Tâche 11
- `ShowWindow(SW_RESTORE)` — son retour dit si la fenêtre était visible, pas si elle est restaurée — `activer_restaure_une_fenetre_reduite` et `activer_refuse_une_fenetre_qui_reste_reduite`, Tâche 8, et l'essai, Tâche 11
- `SendInput` — rend le nombre d'événements insérés, UIPI le bloque sans erreur, et des événements en file partent après le contrôle — `envoi_incomplet_sort_en_3`, `une_combinaison_part_en_un_seul_envoi` et `la_frappe_attend_5_ms_entre_deux_unites`, Tâches 4 et 8
- `VkKeyScanExW` — -1 pour un caractère absent, l'état 6 pour AltGr, Ctrl+Entrée pour `\n`, Ctrl+G pour U+0007 — `comme_au_clavier_refuse_un_caractere_absent`, `taper_arobase_par_altgr`, `entree_et_tab_sans_modificateur` et `comme_au_clavier_refuse_ctrl_sans_alt`, Tâche 5
- `MapVirtualKeyExW`, `MAPVK_VK_TO_VSC_EX` — il rend sans 0xE0 le code des flèches, d'Inser, de Suppr, de Debut, de Fin, de PageHaut et de PageBas (relevé du poste : VK_UP donne 0x0048) — `fleches_et_inser_sont_etendues` et `evenement_coupe_le_balayage`, Tâche 4
- `ToUnicodeEx` — un résultat négatif est une touche morte, et l'état de clavier se lit sur VK_SHIFT, VK_CONTROL et VK_MENU, pas sur les touches latérales — `comme_au_clavier_refuse_une_touche_morte`, Tâche 5, et `taper_arobase_par_altgr`, Tâche 11
- `GetGUIThreadInfo` — ses fenêtres peuvent être nulles pendant une perte d'activation, et la fenêtre focalisée d'une WebView2 est à `msedgewebview2` — `fil_focalise_se_replie_sur_l_active_puis_le_premier_plan`, Tâche 7 ; `exiger_la_disposition_francaise`, qui active d'abord et exige la fenêtre focalisée à `msedgewebview2`, Tâche 11
- `GetAsyncKeyState` — rend « relâchée » quand UIPI isole le premier plan — `modificateurs_revus_apres_l_activation`, Tâche 8
- `GetKeyState(VK_CAPITAL)` — le bit bas dit Verr. Maj — `comme_au_clavier_refuse_quand_verr_maj_est_allumee`, Tâches 8 et 11
- `WaitForSingleObject`, sur `Local\pilote-gestes` — `WAIT_ABANDONED` donne la propriété, et un mutex existant ignore la propriété initiale — `verrou_abandonne_compte_comme_pris`, Tâche 8, et `un_verrou_abandonne_est_repris`, Tâche 11
- `OpenProcessToken` — lisible sur une cible élevée du même compte, refusé pour un autre compte — `niveau_plus_haut_refuse_en_1` et `niveau_illisible_refuse_en_1`, Tâche 8
- `WTSQuerySessionInformationW` et `OpenInputDesktop` — des drapeaux inconnus, un bureau autre que `Default`, un accès refusé — `poste_verrouille_par_cas`, Tâche 8
- `CreateToolhelp32Snapshot` — un PID se réutilise après la mort de son processus — `un_pid_repris_n_est_pas_un_descendant`, Tâche 7
- `GetWindow(GW_OWNER)` — l'absence de propriétaire arrive en erreur, souvent de code `HRESULT(0)` — `deux_fenetres_titrees_sans_option_titre_sortent_en_2`, Tâche 11, qui lit deux fenêtres sans propriétaire
- `EnumChildWindows` — `None` pour parent énumère tout le bureau, et un cadre porte ses propres filles — `cadre_avec_ses_seules_filles_sans_aumid_refuse`, Tâche 7 ; `copie_systemsettings_refusee`, où une `appli-essai` permise tourne à côté de la copie, et `parametres_reduits_restent_refuses`, qui lit les enfants d'un vrai cadre, Tâche 11
- `SHGetPropertyStoreForWindow` et `PKEY_AppUserModel_ID` — sans identifiant, la valeur est vide — `cadre_sans_enfant_ni_aumid_lisible_refuse`, Tâche 7
- `EnumWindows` — tao crée une fenêtre cachée et sans titre, « Tao Thread Event Target » — `fenetre_candidate_exclut_masquee_outil_sans_titre_invisible`, Tâche 7
- `Command::new`, sous Windows — `System32` passe avant le `PATH` du parent, et y trouve le `bash.exe` de WSL — `shell`, qui lance Git Bash et PowerShell 5.1 par leur chemin absolu et vérifie lequel répond, Tâche 11
- `wry`, `with_html` — une origine nulle, où l'IPC peut se perdre sans bruit — le protocole propre `essai://`, et la ligne `pret` attendue par `lancer_appli`, Tâches 10 et 11
- PowerShell 5.1 — son entrée standard part en ASCII, `Out-File` écrit en UTF-16 à BOM, et il décode la sortie d'un exécutable dans la page OEM, sauf `[Console]::OutputEncoding` réglé en UTF-8 — `taper_depuis_powershell_5_1_un_fichier_utf16`, Tâche 11 ; le décodage par défaut passe à la tranche 4 (décision 14)
- Git Bash — il réécrit un argument qui commence par `/` — `taper_depuis_git_bash_l_entree_standard`, qui passe `touche barre`, Tâche 11
- `Set-WinUserLanguageList` — aucune doc ne dit quand la disposition vaut pour les processus lancés ensuite — `exiger_la_disposition_francaise`, que les essais du clavier appellent avant de juger, Tâche 11

## Effets cherchés

- `verifier`, `windows` et `mutants` deviennent des contrôles requis — la règle de protection de `pilote` sur GitHub — Edouard lance par `!` le PUT d'un fichier tiré de la règle en service, après la fusion ; la session lit ensuite `gh api repos/EdouardZemb/pilote/rules/branches/main` et y trouve les six contextes
- ADR-0005 et ADR-0006 remplacent ADR-0002 et ADR-0004 — `docs/decisions/` et `CLAUDE.md` — les anciens ont le statut « remplacé par ADR-000N », les invariants renvoient aux nouveaux, et la revue de la branche compare chaque règle des anciens à sa reprise
- la disposition AZERTY du poste — le runner Windows, en disposition américaine — le job `windows` la pose, et `exiger_la_disposition_francaise` la prouve avant chaque essai du clavier
- chaque processus qu'un essai lance a son profil jetable — le runner Windows — la sentinelle échoue si un fichier paraît dans le profil redirigé, et `journal_tombe_dans_le_profil_redirige` trouve le journal à sa place
- aucun essai de bout en bout sur le bureau d'Edouard — le poste — chaque fonction publique de `tests/commun/mod.rs` commence par `exiger_la_ci`, `tests/statique.rs` refuse un lancement hors de ce module et un test sans `#[ignore]` hors du domaine, et `CLAUDE.md` écrit la règle
- `mutants-complet.yml` ne tourne que depuis la branche principale — GitHub, après la fusion — la session le lance par `gh workflow run mutants-complet.yml`, mesure son passage et note le délai dans la PR de la tranche 2
- un texte par `--fichier` ou par l'entrée standard — PowerShell 5.1 et Git Bash, lancés par leur chemin — `taper_depuis_powershell_5_1_un_fichier_utf16` et `taper_depuis_git_bash_l_entree_standard`
- la sortie et les refus en UTF-8 sans BOM — les octets lus sans shell, Git Bash, et PowerShell 5.1 réglé en UTF-8, au lieu des deux shells du plan maître (décision 14) — `sortie_et_erreurs_en_utf8_sans_bom` compare les octets reçus, `taper_depuis_git_bash_l_entree_standard` ceux du fichier qu'écrit Git Bash, `taper_depuis_powershell_5_1_un_fichier_utf16` le texte que lit PowerShell 5.1
- `pilote --version` dit le commit, ou `inconnu` — une copie sans `.git` (cargo-mutants), un worktree, une branche dont la référence change à chaque commit, et le binaire installé — `version_porte_le_commit_ou_inconnu`, `version_porte_le_commit` (les 12 premiers chiffres de `GITHUB_SHA`), l'étape « La version suit le commit » du job `windows` (deux commits, deux constructions), et le job `mutants` qui construit ; le binaire installé, à l'essai sur le poste (tranche 5)

## Revues

- la revue de chaque tâche, sur la tête qui part, par une session neuve : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12
- la revue de la branche entière, par une session neuve, sur la tête commitée, avant que la PR quitte le brouillon : Tâche 12

---

## Les fichiers

```
Cargo.toml                    l'espace de travail ; le paquet pilote (bibliothèque sans doctest, exécutable)
build.rs                      PILOTE_COMMIT : 12 chiffres hexadécimaux, ou « inconnu »
rust-toolchain.toml           1.99.1, rustfmt, clippy
.cargo/mutants.toml           le domaine seul, `--lib`
src/lib.rs                    pub mod domaine ; sous Windows : mod bureau, mod systeme, pub fn lancer()
src/main.rs                   lancer() sous Windows ; ailleurs, 64
src/domaine/mod.rs            les modules ; réexporte gestes::executer
src/domaine/sortie.rs         Code, Ligne, echapper, valeur, formater, Arret, Issue
src/domaine/cli.rs            Commande, Cible, Selecteur, Source, Forme, Aide, analyser, VERSION
src/domaine/touches.rs        Evenement, NomTouche, Combinaison, analyser_touche, vk, lot_combinaison, evenement
src/domaine/texte.rs          decoder, borner, Unite, unites, lot_unicode, Produit, Disposition, lot_clavier
src/domaine/journal.rs        DateLocale, Appel, ligne, sous_commande, SOUS_COMMANDES
src/domaine/processus.rs      Processus, meme_nom, arbre
src/domaine/refus.rs          REFUSES, AUMID_PARAMETRES, CLASSE_CADRE, processus_refuse, est_keepassxc, fenetre_refusee
src/domaine/fenetre.rs        IdFenetre, FenetreLue, PremierPlan, Retenue, candidate, retenir, accepte, lister
src/domaine/bureau.rs         le trait Bureau, Moment, Duree, EtatVerrou, EtatSession, ErreurSysteme ; réexporte Evenement, Produit
src/domaine/gestes.rs         executer, l'ordre des gardes, l'activation, l'envoi par unités, CADENCE
src/domaine/faux.rs           FauxBureau, AppelFaux (#[cfg(test)])
src/systeme.rs                le journal, l'horloge, la date, la session, la sensibilité au DPI
src/bureau/mod.rs             BureauWindows : impl Bureau
src/bureau/fenetres.rs        EnumWindows, le titre, la classe, les propriétaires, les enfants, DWM, l'AUMID, le premier plan
src/bureau/processus.rs       l'instantané, l'heure de création, le niveau d'intégrité
src/bureau/clavier.rs         SendInput, la disposition du fil focalisé, les touches enfoncées
src/bureau/gardes.rs          le verrou des gestes, le poste verrouillé
tests/statique.rs             les règles des sources
tests/commun/mod.rs           exiger_la_ci, dossier_jetable, Profil, Appli, pilote, shell, …
tests/essais_clavier.rs       les essais du clavier et des shells
tests/essais_gardes.rs        les essais des gardes, du verrou, du journal et de la sortie
essai/appli/Cargo.toml        appli-essai
essai/appli/src/main.rs       la boucle tao, les fenêtres, l'IPC, les options
essai/appli/src/page.html     la page
.github/workflows/ci.yml      verifier, windows, mutants
.github/workflows/mutants-complet.yml
docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md
docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md
```

---

### Tâche 1 : l'espace de travail, la CI de Linux et le contrôle statique

**Fichiers :**
- Créer : `Cargo.toml`, `build.rs`, `rust-toolchain.toml`, `.cargo/mutants.toml`, `src/lib.rs`, `src/main.rs`,
  `src/domaine/mod.rs`, `src/systeme.rs` et `src/bureau/mod.rs` (leur seule documentation de module, `//!`),
  `tests/statique.rs`, `.github/workflows/mutants-complet.yml`, `essai/appli/Cargo.toml`, `essai/appli/src/main.rs`
- Modifier : `.gitignore`, `.github/workflows/ci.yml`

**Interfaces :**
- Produit : la bibliothèque `pilote` (`pub mod domaine`), l'exécutable `pilote`, la variable de compilation
  `PILOTE_COMMIT`, les jobs `verifier` et `mutants`, le contrôle statique et ses aides `refuse` et `admet`.

- [ ] **Étape 1 : l'espace de travail.** `Cargo.toml` :
  ```toml
  [workspace]
  members = [".", "essai/appli"]
  resolver = "3"

  [package]
  name = "pilote"
  version = "0.1.0"
  edition = "2024"
  rust-version = "1.99"
  publish = false
  build = "build.rs"

  [lib]
  doctest = false

  [dependencies]
  clap = { version = "=4.6.7", features = ["derive"] }

  [target.'cfg(windows)'.dependencies]
  windows = { version = "=0.62.2", features = [ "Win32_Foundation", "…" ] }   # les features de la Pile
  ```
  `essai/appli/Cargo.toml` : `name = "appli-essai"`, `version = "0.1.0"`, `edition = "2024"`, `publish = false`,
  sans dépendance pour l'instant ; son `main` écrit « appli-essai ne tourne que sous Windows » et sort en 64,
  partout, jusqu'à la Tâche 10. `rust-toolchain.toml` : `[toolchain]`, `channel = "1.99.1"`,
  `components = ["rustfmt", "clippy"]`. `.gitignore` : ajouter `target/` et `mutants.out*/`.
- [ ] **Étape 2 : `build.rs`**, sans dépendance :
  - `git rev-parse --short=12 HEAD` ; en cas d'échec ou de sortie qui n'est pas faite de 12 chiffres
    hexadécimaux, `inconnu` ; puis `cargo:rustc-env=PILOTE_COMMIT=<valeur>` ;
  - `cargo:rerun-if-changed=build.rs`, puis `cargo:rerun-if-changed=<chemin>` pour chaque chemin qui existe
    parmi : `git rev-parse --git-path HEAD` ; si `git symbolic-ref -q HEAD` rend une référence,
    `git rev-parse --git-path <référence>` ; `git rev-parse --git-path packed-refs` ; `git rev-parse --git-path
    logs/HEAD`, que chaque commit, `pull`, `checkout` ou `reset` allonge, même quand la référence est compactée
    (après `git gc`, son fichier n'existe pas, et le commit suivant le crée sans toucher `HEAD` ni
    `packed-refs`). Sans git, rien de plus.
- [ ] **Étape 3 : `src/main.rs` et `src/lib.rs`.** `main` : sous Windows, `std::process::ExitCode::from(
  pilote::lancer())` ; ailleurs, `eprintln!("pilote ne tourne que sous Windows")` et `ExitCode::from(64)`.
  `lib.rs` : `pub mod domaine;` ; `#[cfg(windows)] mod bureau; #[cfg(windows)] mod systeme;` ;
  `#[cfg(windows)] pub fn lancer() -> u8`, qui rend 64 jusqu'à la Tâche 9. `src/domaine/mod.rs`, `src/systeme.rs`
  et `src/bureau/mod.rs` ne portent que leur documentation de module (le domaine : la règle d'ADR-0003).
- [ ] **Étape 4 : le test du contrôle statique, rouge.** `tests/statique.rs` reprend la forme de pupitre
  (`fn controler(sources: &[(String, String)]) -> Vec<String>`, des listes en `const &[&str]`, le parcours
  récursif depuis `CARGO_MANIFEST_DIR` de `src/`, `tests/` et `essai/appli/src/`, les chemins en `/`, les lignes
  de commentaire entières sautées, la faute « <chemin>:<ligne> contient « <mot> » »). Deux sortes de mots :
  - les mots entiers, qui ne comptent que sans lettre, chiffre ni `_` juste avant ni juste après : `Instant`,
    `SystemTime`, `UNIX_EPOCH`, `GetLocalTime`, `GetTickCount64`, `QueryPerformanceCounter`, `sleep`,
    `temp_dir`, `home_dir`, `LOCALAPPDATA`, `APPDATA`, `USERPROFILE`, `AppData`, `rand`, `SetValue`, `"TEMP"`,
    `"TMP"` ;
  - les préfixes, qui ne demandent la borne qu'avant eux : `std::time`, `env::var`, `GetEnvironmentVariable`,
    `GetEnvironmentStrings`, `ExpandEnvironmentStrings`, `GetTempPath`, `SHGetKnownFolder`, `SHGetFolderPath`,
    `CSIDL_`, `FOLDERID_`, `dirs::`.

  Les règles :
  - dans `src/`, hors de `src/systeme.rs` : l'horloge, l'environnement, les chemins devinés et le hasard, soit
    tous les mots ci-dessus sauf `SetValue` ;
  - dans `src/domaine/` : de plus `std::fs`, `std::env`, `std::process`, `windows::` (préfixes, la `const
    MOTS_DU_DOMAINE`) ;
  - partout sauf `tests/statique.rs` : `SetValue`.

  Les tests :
  ```rust
  #[test] fn sources_respectent_les_regles() { assert_eq!(controler(&lire_les_sources()), Vec::<String>::new()); }
  #[test] fn racines_lues() { let s = lire_les_sources(); for r in ["src/", "tests/", "essai/appli/src/"] {
      assert!(s.iter().any(|(c, _)| c.starts_with(r)), "{r}"); } }
  #[test] fn refuse_l_horloge_hors_de_systeme() {
      refuse("src/domaine/gestes.rs", "let t = Instant::now();", "src/domaine/gestes.rs:1 contient « Instant »"); }
  #[test] fn admet_l_horloge_dans_systeme() { admet("src/systeme.rs", "let t = std::time::Instant::now();"); }
  #[test] fn refuse_une_variante_par_son_prefixe() {
      refuse("src/bureau/mod.rs", "GetTempPathW(&mut b)", "src/bureau/mod.rs:1 contient « GetTempPath »");
      refuse("src/bureau/mod.rs", "std::env::vars_os()", "src/bureau/mod.rs:1 contient « env::var »"); }
  #[test] fn refuse_le_systeme_de_fichiers_dans_le_domaine() {
      refuse("src/domaine/texte.rs", "use std::fs;", "src/domaine/texte.rs:1 contient « std::fs »"); }
  #[test] fn refuse_set_value_partout() {
      refuse("tests/essais_clavier.rs", "x.SetValue(v)", "tests/essais_clavier.rs:1 contient « SetValue »"); }
  #[test] fn cherche_des_noms_bornes() { admet("src/domaine/bureau.rs", "struct Instantane;");
      admet("src/domaine/bureau.rs", "let grand = 1;"); }
  #[test] fn ignore_une_ligne_de_commentaire() { admet("src/domaine/bureau.rs", "// Instant::now() est refusé ici"); }
  #[test] fn chaque_mot_refuse_a_sa_plantation() { for mot in MOTS_ENTIERS.iter().chain(PREFIXES) {
      if *mot == "SetValue" { continue; }
      refuse("src/bureau/mod.rs", &format!("let x = {mot};"), &format!("src/bureau/mod.rs:1 contient « {mot} »")); } }
  #[test] fn chaque_mot_du_domaine_a_sa_plantation() { for mot in MOTS_DU_DOMAINE {
      refuse("src/domaine/gestes.rs", &format!("let x = {mot};"), &format!("src/domaine/gestes.rs:1 contient « {mot} »")); } }
  ```
  `refuse(chemin, texte, faute)` appelle `controler` sur la seule source plantée et attend exactement
  `vec![faute]` ; `admet(chemin, texte)` attend `vec![]`.
- [ ] **Étape 5 : voir rouge** : `cargo test --test statique` échoue (les fonctions ne sont pas écrites).
- [ ] **Étape 6 : écrire le contrôle**, puis `cargo test --workspace` vert, `cargo fmt --all --check` et
  `cargo clippy --workspace --all-targets -- -D warnings` sans faute, sur le poste.
- [ ] **Étape 7 : la CI de Linux.** Dans `.github/workflows/ci.yml`, après `socle:` et à la place du commentaire de
  la ligne 22 :
  ```yaml
    verifier:
      runs-on: ubuntu-latest
      timeout-minutes: 15 # provisoire : mesuré à la Tâche 12
      permissions:
        contents: read
      steps:
        - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        - name: Installer la chaîne de rust-toolchain.toml
          run: rustup toolchain install
        - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
        - name: Cargo.lock à jour
          run: |
            cargo update --workspace
            git diff --exit-code Cargo.lock
        - name: Déposer le Cargo.lock résolu
          if: failure()
          uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
          with:
            name: cargo-lock
            path: Cargo.lock
        - name: Format
          run: cargo fmt --all --check
        - name: Lint
          run: cargo clippy --workspace --all-targets -- -D warnings
        - name: Tests
          run: cargo test --workspace

    mutants:
      if: github.event_name == 'pull_request'
      runs-on: ubuntu-latest
      timeout-minutes: 60 # provisoire : mesuré à la Tâche 12
      permissions:
        contents: read
      steps:
        - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
          with:
            fetch-depth: 0
        - name: Installer la chaîne de rust-toolchain.toml
          run: rustup toolchain install
        - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
        - name: Installer cargo-mutants
          run: cargo install --locked cargo-mutants --version 27.1.0
        - name: Diff de la PR
          env:
            BASE_REF: ${{ github.base_ref }}
          run: git diff "origin/$BASE_REF...HEAD" > pr.diff
        - name: Mutation du domaine
          run: cargo mutants -p pilote --in-diff pr.diff --jobs "$(nproc)"
  ```
  La ligne 23, « Ce fichier est au projet », reste. `.github/workflows/mutants-complet.yml` :
  ```yaml
  # Le passage complet du test de mutation du domaine : chaque lundi, et à la demande.
  # Le job `mutants` de ci.yml, lui, n'examine que les lignes d'une PR.
  name: mutants-complet
  on:
    schedule:
      - cron: '0 5 * * 1'
    workflow_dispatch:
  permissions:
    contents: read
  jobs:
    mutants:
      runs-on: ubuntu-latest
      timeout-minutes: 60 # provisoire : celui du job mutants, puis mesuré après la fusion
      permissions:
        contents: read
      steps:
        - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        - name: Installer la chaîne de rust-toolchain.toml
          run: rustup toolchain install
        - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
        - name: Installer cargo-mutants
          run: cargo install --locked cargo-mutants --version 27.1.0
        - name: Passage complet
          run: cargo mutants -p pilote --jobs "$(nproc)"
  ```
  `.cargo/mutants.toml` : `examine_globs = ["src/domaine/**/*.rs"]`, `additional_cargo_test_args = ["--lib"]`,
  `exclude_re = []`, avec en commentaire : « Un mutant équivalent ou qui boucle s'exclut ici, nommément, avec sa
  raison. »
- [ ] **Étape 8 : commit** : `git add Cargo.toml Cargo.lock build.rs rust-toolchain.toml .cargo/mutants.toml src/
  tests/statique.rs essai/appli/Cargo.toml essai/appli/src/main.rs .gitignore .github/workflows/ci.yml
  .github/workflows/mutants-complet.yml`, puis
  `git commit -m "build: l'espace de travail, la CI de Linux et le contrôle statique"`.

### Tâche 2 : la sortie, les codes et les arrêts

**Fichiers :** créer `src/domaine/sortie.rs` ; modifier `src/domaine/mod.rs`.

**Interfaces :**
- Produit :
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum Code { Fait, Refus, Introuvable, Echec, Usage }      // 0, 1, 2, 3, 64
  impl Code { pub fn valeur(self) -> u8 }
  #[derive(Clone, Debug, PartialEq)]
  pub struct Ligne { pub genre: &'static str, pub nom: Option<String>, pub champs: Vec<(&'static str, String)> }
  pub fn echapper(texte: &str) -> String
  pub fn valeur(texte: &str) -> String
  pub fn formater(ligne: &Ligne) -> String
  #[derive(Clone, Debug, PartialEq)]
  pub enum Arret { Usage(String), Refus(String), Introuvable(String), Echec(String) }
  impl Arret { pub fn code(&self) -> Code; pub fn message(&self) -> &str }
  #[derive(Clone, Debug, PartialEq)]
  pub struct Issue { pub code: Code, pub sortie: Vec<String>, pub erreur: Option<String> }
  impl Issue { pub fn fait(sortie: Vec<String>) -> Issue; pub fn arret(a: Arret) -> Issue }
  ```

- [ ] **Étape 1 : les tests, rouges.**
  ```rust
  #[test] fn codes_valent_0_1_2_3_64() { assert_eq!([Code::Fait, Code::Refus, Code::Introuvable, Code::Echec,
      Code::Usage].map(Code::valeur), [0, 1, 2, 3, 64]); }
  #[test] fn arrets_portent_leur_code() { assert_eq!([Arret::Usage("u".into()).code(), Arret::Refus("r".into()).code(),
      Arret::Introuvable("i".into()).code(), Arret::Echec("e".into()).code()],
      [Code::Usage, Code::Refus, Code::Introuvable, Code::Echec]); }
  #[test] fn echappe_barre_retour_tab_chevron() { assert_eq!(echapper("a\\b\nc\rd\te>f"), r"a\\b\nc\rd\te\>f"); }
  #[test] fn sortie_echappe_les_controles() { assert_eq!(echapper("a\u{7}b\u{2028}c\u{2029}d\u{85}"),
      r"a\u{0007}b\u{2028}c\u{2029}d\u{0085}"); }
  #[test] fn valeur_entre_guillemets_si_espace_ou_guillemet() {
      assert_eq!(valeur("a b"), r#""a b""#); assert_eq!(valeur(r#"a"b"#), r#""a\"b""#); assert_eq!(valeur("ab"), "ab"); }
  #[test] fn valeur_vide_s_ecrit_guillemets() { assert_eq!(valeur(""), r#""""#); }
  #[test] fn ligne_fenetre_forme_exacte() {
      let l = Ligne { genre: "Fenetre", nom: Some("Appli d'essai".into()),
          champs: vec![("processus", "appli-essai".into()), ("pid", "100".into()), ("premier-plan", "oui".into())] };
      assert_eq!(formater(&l), "[Fenetre] <Appli d'essai> processus=appli-essai pid=100 premier-plan=oui"); }
  #[test] fn ligne_echappe_son_nom_et_ses_valeurs() {
      let l = Ligne { genre: "Fenetre", nom: Some("a>b".into()), champs: vec![("processus", "mon appli".into())] };
      assert_eq!(formater(&l), r#"[Fenetre] <a\>b> processus="mon appli""#); }
  #[test] fn ligne_sans_nom() { assert_eq!(formater(&Ligne { genre: "Ecoute", nom: None, champs: vec![] }), "[Ecoute]"); }
  #[test] fn refus_en_2_garde_une_ligne() {
      let i = Issue::arret(Arret::Introuvable(format!("plusieurs fenêtres : <{}>", echapper("a\nb"))));
      assert_eq!(i.code, Code::Introuvable); assert!(!i.erreur.unwrap().contains('\n')); }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib sortie`.
- [ ] **Étape 3 : écrire `sortie.rs`.** `echapper` : `\` → `\\`, `\n`, `\r`, `\t`, `>` → `\>`, tout autre
  caractère de la catégorie Cc et U+2028, U+2029 → `\u{XXXX}` (quatre chiffres hexadécimaux en majuscules au
  moins). `valeur` : `echapper`, puis `"` → `\"`, entre guillemets si le texte contient une espace ou un
  guillemet, `""` s'il est vide. `formater` : `[genre]`, puis ` <` + `echapper(nom)` + `>` si le nom existe, puis
  ` clé=` + `valeur(v)` pour chaque champ, dans l'ordre. `Issue::fait` : code `Fait`, sans erreur ;
  `Issue::arret` : le code de l'arrêt, sans sortie, l'erreur = le message.
- [ ] **Étape 4 : voir vert**, la mutation du module (Contraintes générales), puis commit :
  `git add src/domaine/ .cargo/mutants.toml` et `git commit -m "feat(domaine): la sortie, ses échappements et les codes"`.

### Tâche 3 : l'analyse des arguments

**Fichiers :** créer `src/domaine/cli.rs` ; modifier `src/domaine/mod.rs`.

**Interfaces :**
- Consomme : `Code` (Tâche 2).
- Produit :
  ```rust
  pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("PILOTE_COMMIT"), ")");
  #[derive(Clone, Debug, PartialEq)]
  pub struct Cible { pub fenetre: String, pub titre: Option<String>, pub pid: Option<u32> }
  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct Selecteur { pub nom: Option<String>, pub contient: Option<String>, pub genre: Option<String>,
      pub id: Option<String>, pub classe: Option<String>, pub dans: Option<String>, pub rang: Option<u32> }
  #[derive(Clone, Debug, PartialEq)]
  pub enum Source { Texte(String), Fichier(std::path::PathBuf), Entree }
  #[derive(Clone, Debug, PartialEq)]
  pub enum Forme { Dimensions { largeur: u32, hauteur: u32 }, Agrandir, Restaurer, Equivalent(u16) }
  #[derive(Clone, Debug, PartialEq)]
  pub enum Commande {
      Fenetres, Activer(Cible), Touche { cible: Cible, touches: Vec<String>, fois: u32 },
      Taper { cible: Cible, source: Source, comme_au_clavier: bool },
      Focus { cible: Cible, selecteur: Selecteur, attendre: Option<u32> },
      Arbre { cible: Cible, max: u32, dans: Option<String>, attendre: Option<u32> },
      Trouver { cible: Cible, selecteur: Selecteur, attendre: Option<u32> },
      Focaliser { cible: Cible, selecteur: Selecteur }, Invoquer { cible: Cible, selecteur: Selecteur },
      Choisir { cible: Cible, selecteur: Selecteur },
      Valeur { cible: Cible, selecteur: Selecteur, egale: Option<String>, attendre: Option<u32> },
      Defiler { cible: Cible, selecteur: Selecteur, a: u8 }, Defilables(Cible),
      Capture { cible: Cible, sortie: std::path::PathBuf },
      Molette { cible: Cible, x: i32, y: i32, crans: i32 }, Clic { cible: Cible, x: i32, y: i32, double: bool, droit: bool },
      Taille { cible: Cible, forme: Forme }, Lancer { exe: std::path::PathBuf, donnees_neuves: bool },
      Fermer(Cible), Nettoyer, Annonces { cible: Cible, pendant: u32 },
  }
  impl Commande { pub fn nom(&self) -> &'static str; pub fn cible(&self) -> Option<&Cible> }
  #[derive(Clone, Debug, PartialEq)]
  pub struct Aide { pub code: Code, pub texte: String }   // Fait : sortie standard ; Usage : sortie d'erreur
  pub fn analyser(args: &[std::ffi::OsString]) -> Result<Commande, Aide>
  ```
  `Selecteur::genre` porte l'option `--type` ; `Defiler::a` l'option `--a` (0 à 100) ; `Commande::nom` rend le
  nom de la sous-commande tel que l'écrit la ligne de commande. Pour les tests, un `#[cfg(test)] pub(crate) mod
  essai` de `cli.rs` produit `pub fn a(s: &str) -> Vec<OsString>` et `pub fn cible(f: &str) -> Cible`, que
  `journal.rs` et `gestes.rs` réemploient.

- [ ] **Étape 1 : les tests, rouges.** `a(s)` découpe sur les espaces et met `pilote` en tête, sauf s'il y est
  déjà ; `cible(f)` rend une cible sans titre ni PID.
  ```rust
  #[test] fn usage_inconnu_sort_en_64() { let e = analyser(&a("pilote focus --fenetre x --bidule")).unwrap_err();
      assert_eq!(e.code, Code::Usage); assert!(e.texte.starts_with("option inconnue : --bidule"), "{}", e.texte); }
  #[test] fn aide_et_version_sortent_en_0() {
      assert_eq!(analyser(&a("pilote --help")).unwrap_err().code, Code::Fait);
      let v = analyser(&a("pilote --version")).unwrap_err(); assert_eq!(v.code, Code::Fait);
      assert_eq!(v.texte.trim_end(), format!("pilote {VERSION}")); }
  #[test] fn version_porte_le_commit_ou_inconnu() { let (num, commit) = VERSION.split_once(" (").unwrap();
      assert_eq!(num, env!("CARGO_PKG_VERSION")); let c = commit.strip_suffix(')').unwrap();
      assert!(c == "inconnu" || (c.len() == 12 && c.chars().all(|x| x.is_ascii_hexdigit())), "{c}"); }
  #[test] fn sans_sous_commande_sort_en_64() { let e = analyser(&a("pilote")).unwrap_err();
      assert_eq!(e.code, Code::Usage); assert_eq!(e.texte, "sous-commande manquante"); }
  #[test] fn sous_commande_inconnue() { assert!(analyser(&a("pilote help")).unwrap_err().texte
      .starts_with("sous-commande inconnue : help")); }
  #[test] fn toutes_les_sous_commandes_s_analysent() { for s in [ "fenetres", "activer --fenetre x",
      "touche --fenetre x Tab", "taper --fenetre x --texte t", "taper --fenetre x -", "taper --fenetre x --fichier f",
      "focus --fenetre x", "arbre --fenetre x", "trouver --fenetre x --nom n", "focaliser --fenetre x --nom n",
      "invoquer --fenetre x --nom n", "choisir --fenetre x --nom n", "valeur --fenetre x --nom n",
      "defiler --fenetre x --nom n --a 50", "defilables --fenetre x", "capture --fenetre x --sortie c.png",
      "molette --fenetre x --x 1 --y 2 --crans 3", "clic --fenetre x --x 1 --y 2", "taille --fenetre x --agrandir",
      "lancer e.exe", "fermer --fenetre x", "nettoyer", "annonces --fenetre x --pendant 5" ] {
      let c = analyser(&a(&format!("pilote {s}"))).unwrap_or_else(|e| panic!("{s} : {}", e.texte));
      assert_eq!(c.nom(), s.split(' ').next().unwrap()); } }
  #[test] fn bornes_des_options() { for (s, ok) in [("touche --fenetre x Tab --fois 100", true),
      ("touche --fenetre x Tab --fois 101", false), ("touche --fenetre x Tab --fois 0", false),
      ("focus --fenetre x --attendre 30000", true), ("focus --fenetre x --attendre 30001", false),
      ("arbre --fenetre x --max 10000", true), ("arbre --fenetre x --max 10001", false), ("arbre --fenetre x --max 0", false),
      ("molette --fenetre x --x 0 --y 0 --crans 50", true), ("molette --fenetre x --x 0 --y 0 --crans 51", false),
      ("molette --fenetre x --x 0 --y 0 --crans -50", true), ("molette --fenetre x --x 0 --y 0 --crans -51", false),
      ("molette --fenetre x --x 0 --y 0 --crans 0", false), ("annonces --fenetre x --pendant 1", true),
      ("annonces --fenetre x --pendant 0", false), ("annonces --fenetre x --pendant 300", true),
      ("annonces --fenetre x --pendant 301", false), ("taille --fenetre x --equivalent 100", true),
      ("taille --fenetre x --equivalent 99", false), ("taille --fenetre x --equivalent 400", true),
      ("taille --fenetre x --equivalent 401", false), ("defiler --fenetre x --nom n --a 101", false)] {
      assert_eq!(analyser(&a(&format!("pilote {s}"))).is_ok(), ok, "{s}"); } }
  #[test] fn max_vaut_400_par_defaut() { assert!(matches!(analyser(&a("pilote arbre --fenetre x")).unwrap(),
      Commande::Arbre { max: 400, .. })); }
  #[test] fn fois_vaut_1_par_defaut() { assert!(matches!(analyser(&a("pilote touche --fenetre x Tab")).unwrap(),
      Commande::Touche { fois: 1, .. })); }
  #[test] fn molette_accepte_des_crans_negatifs() { assert!(matches!(
      analyser(&a("pilote molette --fenetre x --x 1 --y 2 --crans -3")).unwrap(), Commande::Molette { crans: -3, .. })); }
  #[test] fn taper_accepte_un_texte_qui_commence_par_un_tiret() {
      assert_eq!(analyser(&a("pilote taper --fenetre x --texte -abc")).unwrap(), Commande::Taper { cible: cible("x"),
          source: Source::Texte("-abc".into()), comme_au_clavier: false }); }
  #[test] fn taper_tiret_lit_l_entree() { assert!(matches!(analyser(&a("pilote taper --fenetre x -")).unwrap(),
      Commande::Taper { source: Source::Entree, .. })); }
  #[test] fn une_seule_source_de_texte() { assert!(analyser(&a("pilote taper --fenetre x --texte t -")).is_err());
      assert!(analyser(&a("pilote taper --fenetre x")).is_err()); }
  #[test] fn taille_exige_une_seule_forme() { assert!(analyser(&a("pilote taille --fenetre x --agrandir --restaurer")).is_err());
      assert!(analyser(&a("pilote taille --fenetre x")).is_err()); }
  #[test] fn fenetre_refuse_un_controle_et_plus_de_260_caracteres() {
      let args: Vec<OsString> = ["pilote", "activer", "--fenetre", "a\tb"].map(Into::into).into();
      assert_eq!(analyser(&args).unwrap_err().code, Code::Usage);
      assert!(analyser(&a(&format!("pilote activer --fenetre {}", "x".repeat(260)))).is_ok());
      assert!(analyser(&a(&format!("pilote activer --fenetre {}", "x".repeat(261)))).is_err()); }
  #[test] fn erreurs_de_clap_en_francais() { assert!(analyser(&a("pilote activer")).unwrap_err().texte
      .starts_with("option requise : --fenetre")); }
  #[test] fn valeur_manquante_se_dit() { assert!(analyser(&a("pilote activer --fenetre")).unwrap_err().texte
      .starts_with("valeur manquante : --fenetre")); }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib cli`.
- [ ] **Étape 3 : écrire `cli.rs`.** Un `#[derive(clap::Parser)] struct Brut` interne, `#[command(name = "pilote",
  version = VERSION, disable_help_subcommand = true)]`, que `analyser` convertit en `Commande` par
  `try_parse_from`. `--crans` : `allow_negative_numbers`, et un analyseur qui refuse 0 ; `--texte`, `--nom`,
  `--contient`, `--titre`, `--id`, `--classe`, `--dans`, `--egale` : `allow_hyphen_values`. `taper` : un groupe
  requis et exclusif de `--texte`, `--fichier` et d'un positionnel qui ne vaut que `-`. `taille` : un groupe
  requis et exclusif de `--largeur` (qui exige `--hauteur`), `--agrandir`, `--restaurer` et `--equivalent`.
  `--fenetre` : un analyseur qui refuse un caractère de contrôle et plus de 260 caractères. Les bornes de la
  décision 11 du plan maître par `value_parser` et ses plages. `DisplayHelp` et `DisplayVersion` donnent
  `Aide { code: Fait }` avec le texte de clap ; toute autre erreur `Aide { code: Usage }`, traduite :
  `UnknownArgument` → `option inconnue : <arg>` ; `MissingRequiredArgument` → `option requise : <arg>` ;
  `InvalidValue` à valeur vide → `valeur manquante : <arg>`, sinon `valeur refusée : <arg>` ; `ValueValidation`
  → `valeur refusée : <arg>` ; `ArgumentConflict` → `options incompatibles : <arg>` ; `InvalidSubcommand` →
  `sous-commande inconnue : <arg>` ; `MissingSubcommand` et `DisplayHelpOnMissingArgumentOrSubcommand` →
  `sous-commande manquante` ; `TooManyValues` et `WrongNumberOfValues` → `trop de valeurs : <arg>` ; les autres,
  `usage : ` puis la première ligne de clap. `<arg>` vient du contexte `InvalidArg` de l'erreur, sauf pour
  `InvalidSubcommand`, où il vient du contexte `InvalidSubcommand` (clap_builder 4.6.7, `error/mod.rs:511` et
  `540`) ; pour `MissingRequiredArgument`, ce contexte est une liste (`ContextValue::Strings`), dont les éléments
  se joignent par une espace.
- [ ] **Étape 4 : voir vert**, la mutation du module, puis commit : `git add src/domaine/ .cargo/mutants.toml` et
  `git commit -m "feat(domaine): l'analyse des arguments et leurs bornes"`.

### Tâche 4 : les touches

**Fichiers :** créer `src/domaine/touches.rs` ; modifier `src/domaine/mod.rs`.

**Interfaces :**
- Consomme : `Arret` (Tâche 2).
- Produit :
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum Evenement { Touche { vk: u8, scan: u8, etendue: bool, relachee: bool },
      Unicode { unite: u16, relachee: bool }, SourisNulle }
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum NomTouche { Tab, Echap, Entree, Espace, Retour, Suppr, Inser, Haut, Bas, Gauche, Droite, Debut, Fin,
      PageHaut, PageBas, F(u8), Lettre(char), Barre, Interrogation }
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub struct Combinaison { pub ctrl: bool, pub maj: bool, pub alt: bool, pub touche: NomTouche }
  pub fn analyser_touche(texte: &str) -> Result<Combinaison, Arret>      // Refus : interdite ; Usage : inconnue
  pub fn vk(touche: NomTouche) -> Option<u8>                              // None : Barre, Interrogation
  pub fn caractere(touche: NomTouche) -> Option<char>                     // Barre : '/', Interrogation : '?'
  pub fn evenement(vk: u8, balayage_ex: u16, relachee: bool) -> Evenement // scan = octet bas ; étendue : décision 9
  pub fn lot_combinaison(c: &Combinaison, vk: u8, balayage: &mut dyn FnMut(u8) -> u16) -> Vec<Evenement>
  pub mod vk {
      pub const RETOUR_CHARIOT: u8 = 0x0D; pub const TAB: u8 = 0x09; pub const ECHAP: u8 = 0x1B;
      pub const ESPACE: u8 = 0x20; pub const RETOUR: u8 = 0x08; pub const SUPPR: u8 = 0x2E; pub const INSER: u8 = 0x2D;
      pub const DEBUT: u8 = 0x24; pub const FIN: u8 = 0x23; pub const PAGE_HAUT: u8 = 0x21; pub const PAGE_BAS: u8 = 0x22;
      pub const GAUCHE: u8 = 0x25; pub const HAUT: u8 = 0x26; pub const DROITE: u8 = 0x27; pub const BAS: u8 = 0x28;
      pub const F1: u8 = 0x70; pub const A: u8 = 0x41;                     // F1 à F12 : 0x70 à 0x7B ; A à Z : 0x41 à 0x5A
      pub const MAJ: u8 = 0x10; pub const CTRL: u8 = 0x11; pub const ALT: u8 = 0x12;
      pub const MAJ_G: u8 = 0xA0; pub const MAJ_D: u8 = 0xA1; pub const CTRL_G: u8 = 0xA2; pub const CTRL_D: u8 = 0xA3;
      pub const ALT_G: u8 = 0xA4; pub const ALT_D: u8 = 0xA5; pub const WIN_G: u8 = 0x5B; pub const WIN_D: u8 = 0x5C;
      pub const VERR_MAJ: u8 = 0x14; pub const SANS_EFFET: u8 = 0xE8;
      pub const MODIFICATEURS: [u8; 8] = [MAJ_G, MAJ_D, CTRL_G, CTRL_D, ALT_G, ALT_D, WIN_G, WIN_D];
  }
  ```

- [ ] **Étape 1 : les tests, rouges.** `balayage_essai` et `touches` vivent dans un `#[cfg(test)] pub(crate) mod
  essai` de `touches.rs`, que les Tâches 5 et 8 réemploient. `balayage_essai` rend ce que `MapVirtualKeyExW` rend
  sur le poste (KBDFR, relevé en lecture seule) : Haut 0x48, Bas 0x50, Gauche 0x4B, Droite 0x4D, Inser 0x52,
  Suppr 0x53, Debut 0x47, Fin 0x4F, PageHaut 0x49, PageBas 0x51, tous sans 0xE0 ; Ctrl droite 0xE01D, Alt
  droite 0xE038 ; Tab 0x0F, Entrée 0x1C, Maj gauche 0x2A, Ctrl gauche 0x1D, Alt gauche 0x38 ; 0x30 → 0x0B,
  0xBF → 0x34, 0x41 → 0x10, 0x42 → 0x30 ; 0 pour toute autre touche. `touches(&lot)` rend les couples
  `(vk, relachee)` des événements de touche.
  ```rust
  #[test] fn noms_du_design() { for (t, v) in [("Tab", 0x09), ("Maj+Tab", 0x09), ("Echap", 0x1B), ("Entree", 0x0D),
      ("Espace", 0x20), ("Retour", 0x08), ("Suppr", 0x2E), ("Haut", 0x26), ("Bas", 0x28), ("Gauche", 0x25),
      ("Droite", 0x27), ("Debut", 0x24), ("Fin", 0x23), ("PageHaut", 0x21), ("PageBas", 0x22), ("F1", 0x70),
      ("F12", 0x7B), ("Ctrl+A", 0x41), ("Ctrl+Z", 0x5A), ("Alt+F4", 0x73), ("Inser", 0x2D)] {
      assert_eq!(vk(analyser_touche(t).unwrap().touche), Some(v), "{t}"); } }
  #[test] fn f13_est_inconnue() { assert_eq!(analyser_touche("F13"), Err(Arret::Usage("touche inconnue : F13".into()))); }
  #[test] fn noms_sans_egard_a_la_casse() { assert_eq!(analyser_touche("maj+tab"), analyser_touche("Maj+Tab")); }
  #[test] fn modificateurs_dans_tout_ordre() { assert_eq!(analyser_touche("Maj+Ctrl+Tab"), analyser_touche("Ctrl+Maj+Tab")); }
  #[test] fn lettre_seule_est_un_usage() { assert!(matches!(analyser_touche("a"), Err(Arret::Usage(_)))); }
  #[test] fn touches_interdites_sortent_en_1() { for t in ["Win", "Win+R", "Windows", "Alt+Tab", "alt+maj+tab",
      "Ctrl+Alt+Tab", "Ctrl+Alt+Suppr", "Alt+Echap", "Ctrl+Echap", "Ctrl+Maj+Echap"] {
      assert_eq!(analyser_touche(t), Err(Arret::Refus(format!("touche interdite : {t}"))), "{t}"); } }
  #[test] fn alt_f4_est_permis() { assert!(analyser_touche("Alt+F4").is_ok()); }
  #[test] fn touche_inconnue_sort_en_64() { assert_eq!(analyser_touche("Bidule"),
      Err(Arret::Usage("touche inconnue : Bidule".into()))); }
  #[test] fn caracteres_nommes_passent_par_la_disposition() {
      let b = analyser_touche("barre").unwrap(); assert_eq!((vk(b.touche), caractere(b.touche)), (None, Some('/')));
      let i = analyser_touche("interrogation").unwrap(); assert_eq!((vk(i.touche), caractere(i.touche)), (None, Some('?'))); }
  #[test] fn une_combinaison_part_en_un_seul_envoi() { let c = analyser_touche("Ctrl+Maj+Tab").unwrap();
      assert_eq!(touches(&lot_combinaison(&c, 0x09, &mut balayage_essai)),
          vec![(0xA2, false), (0xA0, false), (0x09, false), (0x09, true), (0xA0, true), (0xA2, true)]); }
  #[test] fn fleches_et_inser_sont_etendues() { for t in ["Haut", "Bas", "Inser", "Suppr", "PageHaut"] {
      let c = analyser_touche(t).unwrap(); let lot = lot_combinaison(&c, vk(c.touche).unwrap(), &mut balayage_essai);
      assert!(matches!(lot[0], Evenement::Touche { etendue: true, .. }), "{t}"); }
      let c = analyser_touche("Tab").unwrap();
      assert!(matches!(lot_combinaison(&c, 0x09, &mut balayage_essai)[0], Evenement::Touche { etendue: false, scan: 0x0F, .. })); }
  #[test] fn evenement_coupe_le_balayage() { assert_eq!(evenement(0xA5, 0xE038, false),
      Evenement::Touche { vk: 0xA5, scan: 0x38, etendue: true, relachee: false });
      assert_eq!(evenement(0x26, 0x0048, false), Evenement::Touche { vk: 0x26, scan: 0x48, etendue: true, relachee: false });
      assert_eq!(evenement(0x21, 0x0049, true), Evenement::Touche { vk: 0x21, scan: 0x49, etendue: true, relachee: true });
      assert_eq!(evenement(0x2E, 0x0053, false), Evenement::Touche { vk: 0x2E, scan: 0x53, etendue: true, relachee: false });
      assert_eq!(evenement(0x20, 0x0039, false), Evenement::Touche { vk: 0x20, scan: 0x39, etendue: false, relachee: false });
      assert_eq!(evenement(0x29, 0x0000, false), Evenement::Touche { vk: 0x29, scan: 0, etendue: false, relachee: false }); }
  #[test] fn chaque_lot_relache_ce_qu_il_enfonce() { for t in ["Tab", "Maj+Tab", "Ctrl+A", "Ctrl+Maj+Tab", "Alt+F4", "Ctrl+Alt+Haut"] {
      let c = analyser_touche(t).unwrap(); let lot = touches(&lot_combinaison(&c, vk(c.touche).unwrap(), &mut balayage_essai));
      let mut enfoncees = Vec::new();
      for (v, relachee) in lot { if relachee { assert_eq!(enfoncees.pop(), Some(v), "{t}") } else { enfoncees.push(v) } }
      assert!(enfoncees.is_empty(), "{t}"); } }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib touches`.
- [ ] **Étape 3 : écrire `touches.rs`.** Les noms se découpent sur `+`, sans égard à la casse ; `Ctrl`, `Maj`,
  `Alt` sont les modificateurs ; une lettre n'est admise qu'avec Ctrl. Une combinaison interdite se reconnaît sur
  la forme analysée (Win ou Windows en tout morceau ; Alt avec Tab ; Ctrl+Alt avec Suppr ; Alt ou Ctrl avec
  Echap), donc sous toute casse et tout ordre ; son message reprend le texte reçu, et elle se juge avant le nom
  inconnu. Le lot : Ctrl gauche, Maj gauche, Alt gauche enfoncées dans cet ordre, la touche enfoncée puis
  relâchée, les modificateurs relâchés dans l'ordre inverse ; chaque événement par `evenement(vk, balayage(vk),
  …)`, étendu selon la décision 9 (l'octet haut 0xE0, ou une touche virtuelle de 0x21 à 0x28, 0x2D ou 0x2E).
- [ ] **Étape 4 : voir vert**, la mutation du module, puis commit : `git add src/domaine/ .cargo/mutants.toml` et
  `git commit -m "feat(domaine): les touches, leurs combinaisons et les interdites"`.

### Tâche 5 : les textes

**Fichiers :** créer `src/domaine/texte.rs` ; modifier `src/domaine/mod.rs`.

**Interfaces :**
- Consomme : `Arret`, `echapper` (Tâche 2) ; `Evenement`, `evenement`, `vk`, et pour les tests
  `essai::{balayage_essai, touches}` (Tâche 4).
- Produit :
  ```rust
  pub const LONGUEUR_MAX: usize = 10_000;
  pub fn decoder(octets: &[u8]) -> Result<String, Arret>   // BOM, UTF-8 ou UTF-16LE, un saut de ligne final retiré, borné
  pub fn borner(texte: String) -> Result<String, Arret>    // Usage « texte trop long : … » au-delà de LONGUEUR_MAX caractères
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum Unite { Caractere(char), Entree, Tab }
  pub fn unites(texte: &str) -> Vec<Unite>
  pub fn lot_unicode(u: Unite, balayage: &mut dyn FnMut(u8) -> u16) -> Vec<Evenement>
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum Produit { Morte, Texte(String) }
  pub trait Disposition {
      fn scan(&mut self, unite: u16) -> i16;                // VkKeyScanExW, dans la disposition du fil focalisé
      fn produit(&mut self, vk: u8, etat: u8) -> Produit;   // ToUnicodeEx
      fn balayage(&mut self, vk: u8) -> u16;                // MapVirtualKeyExW, MAPVK_VK_TO_VSC_EX
  }
  pub fn lot_clavier(c: char, d: &mut dyn Disposition) -> Result<Vec<Evenement>, Arret>
  ```
  `Disposition` est un super-trait de `Bureau` (Tâche 8). `lot_unicode` donne Entrée et Tab par leurs touches
  virtuelles, un caractère par `KEYEVENTF_UNICODE`, une unité UTF-16 enfoncée puis relâchée, la paire de
  substitution dans le même lot.

- [ ] **Étape 1 : les tests, rouges.** `DispositionEssai` vit dans un `#[cfg(test)] pub(crate) mod essai` de
  `texte.rs`, que la Tâche 8 réemploie. Elle rend l'AZERTY du poste, relevé en lecture seule : `@` 0x0630, `/`
  0x01BF, `?` 0x01BC, `a` 0x0041, `A` 0x0141, `0` 0x0130, `\n` 0x020D, `\t` 0x0009, U+0007 0x0247, `ñ` -1, `^`
  0x00DD ; et des valeurs d'essai, qu'aucun AZERTY ne porte : `¬` 0x0442 (état 4), `÷` 0x0542 (état 5), `ß`
  0x0742 (état 7), `¤` 0x0842 (bit 8), `¶` 0x1042 (bit 16), `§` 0x2042 (bit 32). `produit(0xDD, 0)` vaut
  `Morte` ; tout autre produit vaut le caractère que `scan` a servi, sauf `produit(0x41, 0)`, que le champ
  `produit_de_a` règle à `x` dans le test qui le dit ; `balayage` est `balayage_essai` (Tâche 4).
  ```rust
  #[test] fn fichier_utf8_sans_bom() { assert_eq!(decoder("été".as_bytes()).unwrap(), "été"); }
  #[test] fn fichier_utf8_a_bom() { assert_eq!(decoder(b"\xEF\xBB\xBFab").unwrap(), "ab"); }
  #[test] fn fichier_utf16_a_bom() { assert_eq!(decoder(&[0xFF, 0xFE, b'a', 0, 0xE9, 0]).unwrap(), "aé"); }
  #[test] fn texte_illisible_sort_en_64() { assert_eq!(decoder(&[0xC3, 0x28]),
      Err(Arret::Usage("texte illisible : ni UTF-8 ni UTF-16 à BOM".into())));
      assert_eq!(decoder(&[0xFF, 0xFE, 0x3D, 0xD8]), Err(Arret::Usage("texte illisible : ni UTF-8 ni UTF-16 à BOM".into()))); }
  #[test] fn un_seul_saut_de_ligne_final_ignore() { assert_eq!(decoder(b"a\n\n").unwrap(), "a\n");
      assert_eq!(decoder(b"a\r\n").unwrap(), "a"); assert_eq!(decoder(b"a").unwrap(), "a"); }
  #[test] fn texte_de_10000_caracteres_passe_10001_refuse() { assert!(decoder("é".repeat(10_000).as_bytes()).is_ok());
      assert_eq!(decoder("é".repeat(10_001).as_bytes()),
          Err(Arret::Usage("texte trop long : 10001 caractères, 10000 au plus".into())));
      assert_eq!(borner("x".repeat(10_001)), Err(Arret::Usage("texte trop long : 10001 caractères, 10000 au plus".into()))); }
  #[test] fn taper_crlf_donne_un_seul_entree() { assert_eq!(unites("a\r\nb\rc\nd"), vec![Unite::Caractere('a'),
      Unite::Entree, Unite::Caractere('b'), Unite::Entree, Unite::Caractere('c'), Unite::Entree, Unite::Caractere('d')]); }
  #[test] fn tabulation_devient_tab() { assert_eq!(unites("\t"), vec![Unite::Tab]); }
  #[test] fn entree_par_sa_touche() { assert_eq!(touches(&lot_unicode(Unite::Entree, &mut balayage_essai)),
      vec![(0x0D, false), (0x0D, true)]); }
  #[test] fn taper_hors_du_plan_de_base_envoie_la_paire() { assert_eq!(lot_unicode(Unite::Caractere('😀'), &mut balayage_essai),
      vec![Evenement::Unicode { unite: 0xD83D, relachee: false }, Evenement::Unicode { unite: 0xD83D, relachee: true },
           Evenement::Unicode { unite: 0xDE00, relachee: false }, Evenement::Unicode { unite: 0xDE00, relachee: true }]); }
  #[test] fn lettre_decomposee_part_telle_quelle() { assert_eq!(unites("e\u{301}"),
      vec![Unite::Caractere('e'), Unite::Caractere('\u{301}')]); }
  #[test] fn entree_et_tab_sans_modificateur() {
      assert_eq!(touches(&lot_clavier('\n', &mut DispositionEssai::new()).unwrap()), vec![(0x0D, false), (0x0D, true)]);
      assert_eq!(touches(&lot_clavier('\t', &mut DispositionEssai::new()).unwrap()), vec![(0x09, false), (0x09, true)]); }
  #[test] fn comme_au_clavier_lettre_simple() { assert_eq!(touches(&lot_clavier('a', &mut DispositionEssai::new()).unwrap()),
      vec![(0x41, false), (0x41, true)]); }
  #[test] fn comme_au_clavier_majuscule_par_maj() { assert_eq!(touches(&lot_clavier('A', &mut DispositionEssai::new()).unwrap()),
      vec![(0xA0, false), (0x41, false), (0x41, true), (0xA0, true)]); }
  #[test] fn comme_au_clavier_etat_7_par_maj_et_altgr() { assert_eq!(touches(&lot_clavier('ß', &mut DispositionEssai::new()).unwrap()),
      vec![(0xA0, false), (0xA2, false), (0xA5, false), (0x42, false), (0x42, true), (0xA5, true), (0xA2, true), (0xA0, true)]); }
  #[test] fn comme_au_clavier_refuse_alt_sans_ctrl() { for c in ['¬', '÷'] {
      assert_eq!(lot_clavier(c, &mut DispositionEssai::new()), Err(Arret::Refus(format!("caractère absent du clavier : « {c} »")))); } }
  #[test] fn taper_arobase_par_altgr() { let l = lot_clavier('@', &mut DispositionEssai::new()).unwrap();
      assert_eq!(touches(&l), vec![(0xA2, false), (0xA5, false), (0x30, false), (0x30, true), (0xA5, true), (0xA2, true)]);
      assert_eq!(l[1], Evenement::Touche { vk: 0xA5, scan: 0x38, etendue: true, relachee: false }); }
  #[test] fn barre_par_maj_et_deux_points() { assert_eq!(touches(&lot_clavier('/', &mut DispositionEssai::new()).unwrap()),
      vec![(0xA0, false), (0xBF, false), (0xBF, true), (0xA0, true)]); }
  #[test] fn comme_au_clavier_refuse_un_caractere_absent() { assert_eq!(lot_clavier('ñ', &mut DispositionEssai::new()),
      Err(Arret::Refus("caractère absent du clavier : « ñ »".into()))); }
  #[test] fn comme_au_clavier_refuse_hors_du_plan_de_base() { assert_eq!(lot_clavier('😀', &mut DispositionEssai::new()),
      Err(Arret::Refus("caractère absent du clavier : « 😀 »".into()))); }
  #[test] fn comme_au_clavier_refuse_ctrl_sans_alt() { assert_eq!(lot_clavier('\u{7}', &mut DispositionEssai::new()),
      Err(Arret::Refus(r"caractère absent du clavier : « \u{0007} »".into()))); }
  #[test] fn comme_au_clavier_refuse_les_bits_8_16_32() { for c in ['¤', '¶', '§'] {
      assert_eq!(lot_clavier(c, &mut DispositionEssai::new()), Err(Arret::Refus(format!("caractère absent du clavier : « {c} »")))); } }
  #[test] fn comme_au_clavier_refuse_une_touche_morte() { assert_eq!(lot_clavier('^', &mut DispositionEssai::new()),
      Err(Arret::Refus("touche morte : « ^ »".into()))); }
  #[test] fn comme_au_clavier_refuse_un_produit_different() { let mut d = DispositionEssai::new(); d.produit_de_a = "x";
      assert_eq!(lot_clavier('a', &mut d), Err(Arret::Refus("caractère absent du clavier : « a »".into()))); }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib texte`.
- [ ] **Étape 3 : écrire `texte.rs`** selon la décision 9 de ce plan. L'UTF-16 à BOM se décode par
  `char::decode_utf16`, une paire orpheline étant « texte illisible ». `decoder` appelle `borner` ; la longueur
  se compte en caractères, après le retrait du saut de ligne final. Les caractères des messages passent par
  `echapper`.
- [ ] **Étape 4 : voir vert**, la mutation du module, puis commit : `git add src/domaine/ .cargo/mutants.toml` et
  `git commit -m "feat(domaine): les textes, leur décodage et la frappe par la disposition"`.

### Tâche 6 : le journal

**Fichiers :** créer `src/domaine/journal.rs` ; modifier `src/domaine/mod.rs`.

**Interfaces :**
- Consomme : `Code`, `echapper` (Tâche 2).
- Produit :
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub struct DateLocale { pub annee: u16, pub mois: u8, pub jour: u8, pub heure: u8, pub minute: u8, pub seconde: u8 }
  pub const SOUS_COMMANDES: [&str; 23] = ["fenetres", "activer", "touche", "taper", "focus", "arbre", "trouver",
      "focaliser", "invoquer", "choisir", "valeur", "defiler", "defilables", "capture", "molette", "clic", "taille",
      "lancer", "fermer", "nettoyer", "annonces", "aide", "version"];
  pub fn sous_commande(args: &[std::ffi::OsString]) -> &'static str
  #[derive(Clone, Debug, PartialEq)]
  pub struct Appel<'a> { pub date: DateLocale, pub session: Option<&'a str>, pub sous_commande: &'static str,
      pub processus: Option<&'a str>, pub code: Code, pub longueur: Option<usize> }
  pub fn ligne(appel: &Appel) -> String
  ```

- [ ] **Étape 1 : les tests, rouges.** `d(a, mo, j, h, mi, s)` forme une `DateLocale`, `appel()` un appel complet
  de référence ; `a(…)` vient de `cli::essai` (Tâche 3).
  ```rust
  #[test] fn ligne_forme_exacte() { assert_eq!(ligne(&Appel { date: d(2026, 11, 21, 14, 3, 9), session: Some("s1"),
      sous_commande: "taper", processus: Some("appli-essai"), code: Code::Fait, longueur: Some(12) }),
      "2026-11-21T14:03:09\ts1\ttaper\tappli-essai\t0\t12\n"); }
  #[test] fn champs_absents_vides() { assert_eq!(ligne(&Appel { date: d(2026, 1, 2, 3, 4, 5), session: None,
      sous_commande: "inconnue", processus: None, code: Code::Usage, longueur: None }),
      "2026-01-02T03:04:05\t\tinconnue\t\t64\t\n"); }
  #[test] fn ligne_echappe_une_tabulation_de_fenetre() { let l = ligne(&Appel { session: Some("a\tb\nc"), ..appel() });
      assert_eq!(l.matches('\t').count(), 5); assert_eq!(l.matches('\n').count(), 1); assert!(l.contains(r"a\tb\nc")); }
  #[test] fn sous_commande_d_une_liste_fermee() {
      assert_eq!(sous_commande(&a("pilote taper --texte x")), "taper");
      assert_eq!(sous_commande(&a("pilote secret-tape")), "inconnue");
      assert_eq!(sous_commande(&a("pilote help")), "inconnue");
      assert_eq!(sous_commande(&a("pilote --version")), "version"); assert_eq!(sous_commande(&a("pilote -V")), "version");
      assert_eq!(sous_commande(&a("pilote --help")), "aide"); assert_eq!(sous_commande(&a("pilote -h")), "aide");
      assert_eq!(sous_commande(&a("pilote")), "inconnue"); }
  #[test] fn ligne_d_un_appel_mal_forme_ne_porte_pas_ses_arguments() {
      let args = a("pilote taper --fenetre xyz --txt motdepasse");
      let l = ligne(&Appel { sous_commande: sous_commande(&args), processus: None, session: None, code: Code::Usage,
          longueur: None, date: d(2026, 1, 1, 0, 0, 0) });
      assert_eq!(l, "2026-01-01T00:00:00\t\ttaper\t\t64\t\n"); }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib journal`.
- [ ] **Étape 3 : écrire `journal.rs`.** `sous_commande` lit le premier argument après le programme : `--version`
  et `-V` donnent `version`, `--help` et `-h` donnent `aide`, un nom de la liste se rend, tout autre (`help`
  compris) donne `inconnue`. Chaque champ passe par `echapper` ; la date s'écrit sur quatre, puis deux chiffres.
- [ ] **Étape 4 : voir vert**, la mutation du module, puis commit : `git add src/domaine/ .cargo/mutants.toml` et
  `git commit -m "feat(domaine): la ligne du journal"`.

### Tâche 7 : les processus, la liste de refus et la fenêtre retenue

**Fichiers :** créer `src/domaine/processus.rs`, `src/domaine/refus.rs`, `src/domaine/fenetre.rs` ; modifier
`src/domaine/mod.rs`.

**Interfaces :**
- Consomme : `Cible` (Tâche 3), `Arret`, `Ligne`, `echapper` (Tâche 2).
- Produit :
  ```rust
  // processus.rs
  #[derive(Clone, Debug, PartialEq)]
  pub struct Processus { pub pid: u32, pub parent: u32, pub nom: String, pub creation: Option<u64> }
  pub fn meme_nom(nom_exe: &str, demande: &str) -> bool
  pub fn sans_exe(nom_exe: &str) -> &str
  pub fn arbre(racines: &[u32], tous: &[Processus]) -> std::collections::BTreeSet<u32>
  // refus.rs
  pub const REFUSES: [&str; 4] = ["consent", "LogonUI", "CredentialUIBroker", "SystemSettings"];
  pub const AUMID_PARAMETRES: &str =
      "windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel";
  pub const CLASSE_CADRE: &str = "ApplicationFrameWindow";
  pub fn processus_refuse(nom_exe: &str) -> bool
  pub fn est_keepassxc(nom_exe: &str) -> bool                                   // sans égard à la casse, .exe ou non
  pub fn fenetre_refusee(f: &FenetreLue, tous: &[Processus]) -> Option<String>   // le nom, sans .exe, qui refuse
  pub fn keepassxc_dans(f: &FenetreLue, tous: &[Processus]) -> bool             // P ou un enfant d'un autre processus
  // fenetre.rs
  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
  pub struct IdFenetre(pub u64);
  #[derive(Clone, Debug, PartialEq)]
  pub struct FenetreLue { pub id: IdFenetre, pub pid: u32, pub titre: String, pub classe: String, pub visible: bool,
      pub masquee: bool, pub outil: bool, pub proprietaire: Option<IdFenetre>, pub reduite: bool,
      pub aumid: Option<String>, pub pids_enfants: Vec<u32> }
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct PremierPlan { pub id: IdFenetre, pub pid: u32, pub proprietaires: Vec<IdFenetre> }  // la chaîne, 16 au plus
  #[derive(Clone, Debug, PartialEq)]
  pub struct Retenue { pub fenetre: FenetreLue, pub processus: Processus, pub arbre: std::collections::BTreeSet<u32> }
  pub fn candidate(f: &FenetreLue) -> bool
  pub fn retenir(cible: &Cible, fenetres: &[FenetreLue], tous: &[Processus]) -> Result<Retenue, Arret>
  pub fn accepte(pp: Option<&PremierPlan>, r: &Retenue, tous: &[Processus]) -> bool
  pub fn ligne_de(f: &FenetreLue, tous: &[Processus], premier_plan: bool) -> Ligne
  pub fn lister(fenetres: &[FenetreLue], tous: &[Processus], pp: Option<&PremierPlan>) -> Vec<Ligne>
  pub fn fil_focalise(focus: Option<IdFenetre>, active: Option<IdFenetre>, premier_plan: Option<IdFenetre>)
      -> Option<IdFenetre>                                 // la fenêtre dont le fil donne la disposition (décision 2)
  ```

- [ ] **Étape 1 : les tests, rouges.** Des constructeurs d'essai `p(pid, parent, nom, creation)` et
  `f(id, pid, titre)` (visible, ni masquée, ni outil, ni réduite, classe `Fenetre`, sans propriétaire, AUMID ni
  enfants), `pp(id, pid, proprietaires)`, `cible(f)`.
  ```rust
  #[test] fn meme_nom_sans_egard_a_la_casse_ni_a_exe() { assert!(meme_nom("Etabli.EXE", "etabli"));
      assert!(meme_nom("etabli.exe", "etabli.exe")); assert!(!meme_nom("etablis.exe", "etabli")); }
  #[test] fn arbre_suit_les_descendants() { let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", Some(11)),
      p(3, 2, "c.exe", Some(12))]; assert_eq!(arbre(&[1], &t), [1, 2, 3].into()); }
  #[test] fn un_pid_repris_n_est_pas_un_descendant() { let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", Some(5))];
      assert_eq!(arbre(&[1], &t), [1].into()); }
  #[test] fn creation_illisible_n_est_pas_un_descendant() { let t = [p(1, 0, "a.exe", Some(10)), p(2, 1, "b.exe", None)];
      assert_eq!(arbre(&[1], &t), [1].into()); }
  #[test] fn refuse_les_quatre_processus_sans_egard_a_la_casse() { for n in ["consent.exe", "LOGONUI.EXE",
      "CredentialUIBroker.exe", "systemsettings.exe"] { assert!(processus_refuse(n), "{n}"); }
      assert!(!processus_refuse("SystemSettingsBroker.exe")); }
  #[test] fn refuse_une_fenetre_dont_un_enfant_est_refuse() { let t = [p(10, 0, "appli.exe", Some(1)),
      p(7, 0, "SystemSettings.exe", Some(1))]; let mut w = f(1, 10, "A"); w.pids_enfants = vec![7];
      assert_eq!(fenetre_refusee(&w, &t), Some("SystemSettings".into())); }
  #[test] fn cadre_des_parametres_refuse_par_son_aumid() { let t = [p(5, 0, "ApplicationFrameHost.exe", Some(1))];
      let mut c = f(1, 5, "Paramètres"); c.classe = CLASSE_CADRE.into(); c.aumid = Some(AUMID_PARAMETRES.into());
      c.pids_enfants = vec![5]; assert_eq!(fenetre_refusee(&c, &t), Some("SystemSettings".into())); }
  #[test] fn cadre_sans_enfant_ni_aumid_lisible_refuse() { /* classe CLASSE_CADRE, aumid None, pids_enfants vide :
      Some("SystemSettings") */ }
  #[test] fn cadre_avec_ses_seules_filles_sans_aumid_refuse() { /* pids_enfants = [pid du cadre], aumid None :
      Some("SystemSettings") ; de même réduit */ }
  #[test] fn cadre_d_une_autre_application_passe() { /* aumid de la Calculatrice, enfants [5, 30], 30 = CalculatorApp.exe : None */ }
  #[test] fn parametres_reduits_restent_refuses() { /* cadre réduit, enfants [5] seulement, aumid AUMID_PARAMETRES :
      Some("SystemSettings") */ }
  #[test] fn keepassxc_par_p_ou_par_un_enfant() { /* W de KeePassXC.exe : true ; W d'explorer.exe avec un enfant de
      KeePassXC.exe : true ; aucun des deux : false */ }
  #[test] fn fenetre_candidate_exclut_masquee_outil_sans_titre_invisible() { let base = f(1, 10, "A");
      assert!(candidate(&base));
      let gestes: [fn(&mut FenetreLue); 4] = [|w| w.masquee = true, |w| w.outil = true, |w| w.titre.clear(), |w| w.visible = false];
      for g in gestes { let mut w = base.clone(); g(&mut w); assert!(!candidate(&w)); } }
  #[test] fn fil_focalise_se_replie_sur_l_active_puis_le_premier_plan() {
      let (f, a, p) = (Some(IdFenetre(1)), Some(IdFenetre(2)), Some(IdFenetre(3)));
      assert_eq!(fil_focalise(f, a, p), f); assert_eq!(fil_focalise(f, None, None), f);
      assert_eq!(fil_focalise(None, a, p), a); assert_eq!(fil_focalise(None, a, None), a);
      assert_eq!(fil_focalise(None, None, p), p); assert_eq!(fil_focalise(None, None, None), None); }
  #[test] fn retient_la_seule_fenetre_de_l_arbre() { let t = [p(100, 1, "appli-essai.exe", Some(5)), p(101, 100, "x.exe", Some(6))];
      let r = retenir(&cible("appli-essai"), &[f(1, 100, "Appli d'essai")], &t).unwrap();
      assert_eq!((r.fenetre.id, r.processus.pid, r.arbre.clone()), (IdFenetre(1), 100, [100, 101].into())); }
  #[test] fn fenetre_d_un_descendant_est_candidate() { /* racine « appli » 100, W au fils 101 : retenue, P = 101,
      arbre = {101} */ }
  #[test] fn aucune_fenetre_sort_en_2() { assert_eq!(retenir(&cible("absent"), &[], &[]).unwrap_err(),
      Arret::Introuvable("aucune fenêtre de « absent »".into())); }
  #[test] fn deux_fenetres_titrees_sans_option_titre_sortent_en_2() { assert_eq!(retenir(&cible("appli-essai"),
      &[f(1, 100, "Appli d'essai"), f(2, 100, "Dialogue d'essai")], &[p(100, 0, "appli-essai.exe", Some(1))]).unwrap_err(),
      Arret::Introuvable("plusieurs fenêtres de « appli-essai » : <Appli d'essai> <Dialogue d'essai> ; --titre ou --pid départage".into())); }
  #[test] fn titre_departage_par_egalite_exacte() { /* --titre "Dialogue d'essai" retient 2 ; --titre "Dialogue" :
      Introuvable « aucune fenêtre de « appli-essai » » */ }
  #[test] fn pid_garde_la_seule_racine_choisie() { /* deux racines « etabli » 100 et 200, une fenêtre chacune ;
      --pid 200 retient celle de 200, et l'arbre ne contient pas 100 */ }
  #[test] fn candidates_refusees_s_ecartent_avant_le_compte() { /* --fenetre ApplicationFrameHost : le cadre des
      Paramètres et celui de la Calculatrice ; la Calculatrice est retenue */ }
  #[test] fn seules_des_refusees_refusent_en_1() { assert_eq!(retenir(&cible("SystemSettings"), &[f(1, 7, "Paramètres")],
      &[p(7, 0, "SystemSettings.exe", Some(1))]).unwrap_err(), Arret::Refus("fenêtre refusée : processus SystemSettings".into())); }
  #[test] fn plusieurs_sans_citer_une_refusee() { /* deux fenêtres permises et une refusée du même arbre : le message
      cite les deux permises seulement */ }
  #[test] fn premier_plan_accepte_w_ou_ce_que_w_possede() { let (r, t) = retenue_avec_arbre(); // W = 1, P = 100, fils 101
      assert!(accepte(Some(&pp(1, 100, vec![])), &r, &t));
      assert!(accepte(Some(&pp(9, 101, vec![IdFenetre(1)])), &r, &t));
      assert!(accepte(Some(&pp(8, 100, vec![IdFenetre(9), IdFenetre(1)])), &r, &t)); }
  #[test] fn premier_plan_refuse_une_autre_fenetre_de_p() { let (r, t) = retenue_avec_arbre();
      assert!(!accepte(Some(&pp(2, 100, vec![])), &r, &t)); }
  #[test] fn premier_plan_refuse_un_dialogue_d_un_autre_arbre() { let (r, t) = retenue_avec_arbre();
      assert!(!accepte(Some(&pp(9, 555, vec![IdFenetre(1)])), &r, &t)); }
  #[test] fn premier_plan_refuse_un_dialogue_refuse() { /* possédée par W, d'un descendant de P nommé
      CredentialUIBroker.exe : false */ }
  #[test] fn premier_plan_refuse_un_dialogue_de_keepassxc() { /* possédée par W, d'un descendant de P nommé
      KeePassXC.exe : false */ }
  #[test] fn premier_plan_nul_refuse() { let (r, t) = retenue_avec_arbre(); assert!(!accepte(None, &r, &t)); }
  #[test] fn deux_racines_de_meme_nom_le_premier_plan_sur_l_autre_refuse() { /* --pid 100 retenu ; le premier plan
      sur la fenêtre de 200, même nom, sans propriétaire : false */ }
  #[test] fn fenetres_liste_sans_les_refusees_avec_le_premier_plan() { /* trois fenêtres de trois processus, dont
      une des Paramètres, le premier plan sur « zeta » : exactement
      ["[Fenetre] <A> processus=alpha pid=3", "[Fenetre] <Z> processus=zeta pid=2 premier-plan=oui"] */ }
  ```
- [ ] **Étape 2 : voir rouge** : `cargo test --lib -- processus refus fenetre`.
- [ ] **Étape 3 : écrire les trois modules** selon la décision 5 du plan maître et la décision 11 de ce plan :
  - un descendant compte si son parent est dans l'arbre et que les deux heures de création se lisent, celle du
    fils n'étant pas antérieure à celle du père ;
  - `fenetre_refusee` : le processus de la fenêtre est refusé ; ou le processus d'un de ses enfants d'un autre
    processus l'est ; ou, pour un cadre de classe `CLASSE_CADRE`, l'AUMID vaut celui des Paramètres, ou bien
    l'AUMID manque et aucun enfant d'un autre processus ne se lit (le nom rendu est alors `SystemSettings`) ;
  - `retenir` : les racines nommées par `--fenetre` (restreintes à `--pid` s'il est donné), leur arbre, les
    candidates de cet arbre, les refusées écartées (refus 1 s'il n'en restait que de refusées), `--titre`, puis
    le compte ; P est le processus de W, l'arbre retenu P et ses descendants ;
  - `accepte` : selon la décision 11 de ce plan ;
  - `fil_focalise` : la fenêtre focalisée, à défaut l'active, à défaut celle du premier plan, `None` si les trois
    manquent ;
  - le nom d'un processus dans un message ou une ligne est celui de son exécutable sans `.exe` ; `lister` prend
    les candidates de tous les processus, sauf les refusées, triées par nom, PID, puis titre.
- [ ] **Étape 4 : voir vert**, la mutation des trois modules, puis commit : `git add src/domaine/ .cargo/mutants.toml` et
  `git commit -m "feat(domaine): les processus, la liste de refus et la fenêtre retenue"`.

### Tâche 8 : le trait `Bureau`, le faux bureau et les gardes

**Fichiers :** créer `src/domaine/bureau.rs`, `src/domaine/gestes.rs`, `src/domaine/faux.rs` ; modifier
`src/domaine/mod.rs`.

**Interfaces :**
- Consomme : tout ce que produisent les Tâches 2 à 7.
- Produit :
  ```rust
  // bureau.rs
  pub use crate::domaine::texte::Produit; pub use crate::domaine::touches::Evenement;
  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct Moment(pub u64);   // ms monotones
  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)] pub struct Duree(pub u64);    // ms
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum EtatVerrou { Pris, Repris, Occupe }
  #[derive(Clone, Debug, PartialEq, Eq, Default)]
  pub struct EtatSession { pub bureau_d_entree: Option<String>, pub drapeaux: Option<i32>, pub etat: Option<i32> }
  #[derive(Clone, Debug, PartialEq)] pub struct ErreurSysteme(pub String);
  pub const WTS_VERROUILLE: i32 = 0; pub const WTS_DEVERROUILLE: i32 = 1; pub const WTS_INCONNU: i32 = -1;
  pub const WTS_ACTIF: i32 = 0;
  pub trait Bureau: crate::domaine::texte::Disposition {
      fn maintenant(&mut self) -> Moment;
      fn attendre(&mut self, duree: Duree);
      fn date_locale(&mut self) -> DateLocale;
      fn session(&mut self) -> Option<String>;
      fn ouvrir_journal(&mut self) -> Result<(), ErreurSysteme>;
      fn ajouter_au_journal(&mut self, ligne: &str) -> Result<(), ErreurSysteme>;
      fn lire_fichier(&mut self, chemin: &std::path::Path) -> Result<Vec<u8>, ErreurSysteme>;
      fn lire_entree(&mut self) -> Result<Vec<u8>, ErreurSysteme>;
      fn processus(&mut self) -> Result<Vec<Processus>, ErreurSysteme>;
      fn fenetres(&mut self) -> Result<Vec<FenetreLue>, ErreurSysteme>;
      fn premier_plan(&mut self) -> Option<PremierPlan>;
      fn est_reduite(&mut self, f: IdFenetre) -> bool;
      fn prendre_verrou(&mut self, delai: Duree) -> Result<EtatVerrou, ErreurSysteme>;
      fn etat_session(&mut self) -> EtatSession;
      fn niveau_integrite(&mut self, pid: Option<u32>) -> Option<u32>;   // None : pilote lui-même
      fn touches_enfoncees(&mut self) -> Vec<u8>;                         // parmi vk::MODIFICATEURS
      fn verr_maj(&mut self) -> bool;
      fn restaurer(&mut self, f: IdFenetre);
      fn demander_premier_plan(&mut self, f: IdFenetre);
      fn envoyer(&mut self, lot: &[Evenement]) -> u32;
  }
  // gestes.rs
  pub const ATTENTE_DU_VERROU: Duree = Duree(5_000);
  pub const CADENCE: Duree = Duree(5);
  pub fn executer(args: &[std::ffi::OsString], b: &mut dyn Bureau) -> Issue
  // faux.rs (#[cfg(test)])
  #[derive(Clone, Debug, PartialEq)]
  pub enum AppelFaux { OuvrirJournal, Ajout(String), LireFichier, LireEntree, Processus, Fenetres, PremierPlan,
      EstReduite, PrendreVerrou(Duree), EtatSession, Niveau(Option<u32>), TouchesEnfoncees, VerrMaj,
      Restaurer(IdFenetre), Demande(IdFenetre), Envoi(Vec<Evenement>), Attendre(Duree) }
  pub struct FauxBureau { pub appels: Vec<AppelFaux>, pub journal: Vec<String>, /* les réglages ci-dessous */ }
  impl FauxBureau { pub fn avec_appli() -> Self; pub fn envois(&self) -> Vec<Vec<Evenement>> }
  ```
  `FauxBureau::avec_appli()` : le processus 100 « appli-essai.exe », sa fenêtre 1 « Appli d'essai » au premier
  plan, des niveaux égaux (0x2000), le bureau `Default`, les drapeaux `WTS_DEVERROUILLE`, l'état `WTS_ACTIF`, le
  journal qui s'ouvre, le verrou libre, aucune touche enfoncée, Verr. Maj éteinte, l'AZERTY de la Tâche 5, un
  temps simulé que `attendre` avance, chaque envoi insérant tout son lot. Ses réglages, par champ public : le
  journal qui ne s'ouvre pas, ou dont l'ajout échoue ; le premier plan qui change après le n-ième envoi ; le
  premier plan accordé jamais, au mouvement nul ou après Alt ; la fenêtre réduite jusqu'à `restaurer`, ou pour
  toujours ; un enfant refusé, ou de `KeePassXC.exe`, qui paraît à la deuxième lecture des fenêtres (le refus
  rejugé) ; les touches enfoncées avant et après l'activation ; les modificateurs tenus N ms de temps simulé
  après l'envoi d'Alt, N infini compris ; un envoi qui insère moins ; une lecture de fenêtres qui panique ; les
  fichiers lisibles. Il réemploie `cli::essai`, `touches::essai` et `texte::essai::DispositionEssai`.

- [ ] **Étape 1 : les tests de l'ordre des gardes, rouges.** `x(&mut b, "touche --fenetre appli-essai Tab")`
  appelle `executer` sur `a(…)`, qui met `pilote` en tête ; `aucun_geste(&b)` vérifie l'absence de `Restaurer`,
  `Demande` et `Envoi`. Une invocation citée en commentaire sans `--fenetre` s'entend avec `--fenetre
  appli-essai`.
  ```rust
  #[test] fn lecture_n_active_rien() { let mut b = FauxBureau::avec_appli(); assert_eq!(x(&mut b, "fenetres").code, Code::Fait);
      assert!(aucun_geste(&b)); assert!(!b.appels.iter().any(|a| matches!(a, AppelFaux::PrendreVerrou(_)))); }
  #[test] fn journal_inaccessible_refuse_avant_tout_geste() { let mut b = FauxBureau::avec_appli(); b.journal_ferme = true;
      let i = x(&mut b, "touche --fenetre appli-essai Tab"); assert_eq!(i.code, Code::Refus);
      assert_eq!(i.erreur.as_deref(), Some("journal inaccessible : ouvert dans une autre application ?"));
      assert!(!b.appels.iter().any(|a| matches!(a, AppelFaux::Fenetres | AppelFaux::Processus))); assert!(aucun_geste(&b)); }
  #[test] fn touche_interdite_avant_le_journal() { let mut b = FauxBureau::avec_appli(); b.journal_ferme = true;
      let i = x(&mut b, "touche --fenetre appli-essai Alt+Tab");
      assert_eq!(i.erreur.as_deref(), Some("touche interdite : Alt+Tab")); assert!(aucun_geste(&b)); }
  #[test] fn touche_interdite_a_sa_ligne() { /* journal ouvert : 1, aucune lecture de fenêtres, une ligne de code 1 */ }
  #[test] fn usage_avant_la_touche_interdite() { /* « touche --fenetre appli-essai Alt+Tab --bidule » : 64, un
      message qui commence par « option inconnue : --bidule », aucun geste */ }
  #[test] fn texte_trop_long_avant_le_journal() { /* journal fermé, « taper --fenetre appli-essai --texte <10001 x> » :
      64, « texte trop long : 10001 caractères, 10000 au plus », aucun Fenetres, PrendreVerrou ni geste */ }
  #[test] fn taper_texte_de_10001_caracteres_sort_en_64() { /* « taper --fenetre appli-essai --texte <10001 x> » :
      64, le message, aucun Fenetres, PrendreVerrou ni geste */ }
  #[test] fn taper_un_fichier_illisible_sort_en_64() { /* lire_fichier en erreur : 64, « fichier illisible : f.txt »,
      aucun Fenetres ni geste */ }
  #[test] fn taper_une_entree_illisible_sort_en_64() { /* lire_entree en erreur : 64, « entrée standard illisible » */ }
  #[test] fn fenetre_refusee_avant_le_verrou() { /* la fenêtre de SystemSettings : 1, aucun PrendreVerrou, aucun geste */ }
  #[test] fn keepassxc_par_explorer_refuse_taper() { /* « taper --fenetre explorer », W à KeePassXC.exe lancé par
      explorer : 1, « KeePassXC : touche et taper sont refusés dans cette version », aucun PrendreVerrou ni geste */ }
  #[test] fn keepassxc_par_explorer_refuse_touche() { /* de même pour « touche … Tab » */ }
  #[test] fn keepassxc_enfant_de_w_refuse_taper() { /* W d'appli-essai, un enfant de KeePassXC.exe : 1, « KeePassXC :
      touche et taper sont refusés dans cette version », aucun PrendreVerrou ni geste */ }
  #[test] fn activer_vers_keepassxc_est_permis() { /* « activer --fenetre KeePassXC » : 0 */ }
  #[test] fn verrou_demande_5_s() { /* PrendreVerrou(Duree(5000)) parmi les appels */ }
  #[test] fn activer_prend_le_verrou() { /* « activer » : PrendreVerrou(Duree(5000)) */ }
  #[test] fn verrou_occupe_refuse_en_1_sans_rien_envoyer() { /* Occupe : 1, « un autre geste est en cours depuis 5 s »,
      aucun EtatSession, aucun geste */ }
  #[test] fn verrou_abandonne_compte_comme_pris() { /* Repris : 0 */ }
  #[test] fn poste_verrouille_par_cas() { for (bureau, drapeaux, etat, verrouille) in [
      (Some("Default"), Some(WTS_DEVERROUILLE), Some(WTS_ACTIF), false),
      (Some("Winlogon"), Some(WTS_DEVERROUILLE), Some(WTS_ACTIF), true),
      (None, Some(WTS_DEVERROUILLE), Some(WTS_ACTIF), true),
      (Some("Default"), Some(WTS_VERROUILLE), Some(WTS_ACTIF), true),
      (Some("Default"), Some(WTS_INCONNU), Some(WTS_ACTIF), false),
      (Some("Default"), None, None, false),
      (None, None, None, true),
      (Some("Default"), Some(WTS_DEVERROUILLE), Some(4), true),
      (Some("Default"), Some(WTS_INCONNU), Some(4), true)] {
      let mut b = FauxBureau::avec_appli(); b.session = EtatSession { bureau_d_entree: bureau.map(Into::into), drapeaux, etat };
      let i = x(&mut b, "touche --fenetre appli-essai Tab");
      if verrouille { assert_eq!(i.erreur.as_deref(), Some("poste verrouillé")); assert!(aucun_geste(&b)); }
      else { assert_eq!(i.code, Code::Fait); } } }
  #[test] fn niveau_plus_haut_refuse_en_1() { /* cible 0x3000, pilote 0x2000 : « fenêtre plus élevée que pilote », aucun geste */ }
  #[test] fn niveau_egal_ou_plus_bas_passe() { /* 0x2000 et 0x2000 ; 0x1000 et 0x2000 : 0 */ }
  #[test] fn niveau_illisible_refuse_en_1() { /* cible None : « niveau d'intégrité illisible » ; pilote None : de même */ }
  #[test] fn refuse_si_un_modificateur_est_enfonce() { /* touches_enfoncees [0xA0] avant : 1, « touche de
      modification enfoncée », aucun geste */ }
  #[test] fn modificateurs_revus_apres_l_activation() { /* aucune avant, Maj enfoncée après la Demande : 1, aucun Envoi
      de touche */ }
  #[test] fn verrou_avant_le_poste_verrouille() { /* Occupe et poste verrouillé : le message du verrou */ }
  #[test] fn poste_verrouille_avant_le_niveau() { /* poste verrouillé et cible plus haute : « poste verrouillé » */ }
  #[test] fn niveau_avant_les_modificateurs() { /* cible plus haute et Maj enfoncée : « fenêtre plus élevée que pilote » */ }
  #[test] fn modificateurs_avant_l_activation() { /* Maj enfoncée et fenêtre réduite : le message des modificateurs,
      aucun Restaurer */ }
  ```
- [ ] **Étape 2 : les tests de l'activation et de l'envoi, rouges.**
  ```rust
  #[test] fn activer_ne_fait_rien_si_le_premier_plan_est_accepte() { /* aucune Demande, aucun Envoi */ }
  #[test] fn activer_quand_une_autre_fenetre_de_p_a_le_premier_plan() { /* le premier plan sur la fenêtre 2 de P,
      sans propriétaire : une Demande(1), puis 0 */ }
  #[test] fn activer_essaie_le_mouvement_nul_puis_alt_entourant() { /* accordé après Alt : Envoi([SourisNulle]),
      Demande(1), puis l'Envoi d'Alt gauche enfoncée, 0xE8 enfoncée, 0xE8 relâchée, Alt gauche relâchée, puis
      Demande(1) ; code 0 */ }
  #[test] fn activer_attend_que_les_modificateurs_soient_relaches() { /* Alt tenue 30 ms après son envoi : des
      Attendre(Duree(10)) jusqu'au relâchement, puis la Demande ; Alt tenue pour toujours : 1, « touche de
      modification enfoncée », aucune Demande après l'Envoi d'Alt */ }
  #[test] fn activer_ne_frappe_jamais_alt_seule() { /* dans chaque scénario d'activation, aucun lot qui porte au
      moins une touche et dont toutes les touches sont des modificateurs */ }
  #[test] fn activer_restaure_une_fenetre_reduite() { /* Restaurer(1) avant toute Demande */ }
  #[test] fn activer_refuse_une_fenetre_qui_reste_reduite() { /* réduite pour toujours : 1, « Windows a refusé le
      premier plan », aucune Demande, au plus 2 s de temps simulé */ }
  #[test] fn activer_refuse_quand_windows_refuse() { /* jamais accordé : 1, « Windows a refusé le premier plan » */ }
  #[test] fn refus_rejuge_apres_l_activation() { /* un enfant SystemSettings paraît à la deuxième lecture des
      fenêtres : 1, « fenêtre refusée : processus SystemSettings », aucun Envoi du lot de Tab */ }
  #[test] fn keepassxc_rejuge_apres_l_activation() { /* « touche --fenetre appli-essai Tab », W réduite jusqu'à
      `restaurer`, un enfant de KeePassXC.exe qui paraît à la deuxième lecture des fenêtres : 1, « KeePassXC :
      touche et taper sont refusés dans cette version », aucun Envoi du lot de Tab */ }
  #[test] fn refus_rejuge_avant_les_modificateurs_revus() { /* W réduite jusqu'à `restaurer`, un enfant
      SystemSettings à la deuxième lecture des fenêtres, et Maj enfoncée après l'activation : « fenêtre refusée :
      processus SystemSettings », aucun Envoi du lot de Tab */ }
  #[test] fn activer_ecrit_la_ligne_de_la_fenetre() { /* sortie exactement
      ["[Fenetre] <Appli d'essai> processus=appli-essai pid=100 premier-plan=oui"] */ }
  #[test] fn premier_plan_verifie_avant_chaque_unite() { /* « taper --fenetre appli-essai --texte abc » : chaque
      Envoi suit un PremierPlan sans autre Envoi entre eux */ }
  #[test] fn la_frappe_attend_5_ms_entre_deux_unites() { /* « taper --fenetre appli-essai --texte abc » : un
      Attendre(Duree(5)) entre deux Envoi */ }
  #[test] fn taper_s_arrete_quand_le_premier_plan_change() { /* « taper --fenetre appli-essai --texte abcdefghij »,
      le premier plan passe à une fenêtre de pid 555 après le 3e envoi : 1, « le premier plan a changé : 3 unités
      sur 10 sont parties », trois envois */ }
  #[test] fn une_combinaison_part_en_un_seul_envoi() { /* « touche --fenetre appli-essai Ctrl+Maj+Tab » : un seul
      Envoi de six événements */ }
  #[test] fn touche_fois_repete_la_suite() { /* « touche --fenetre appli-essai Tab Entree --fois 2 » : quatre envois,
      Tab, Entree, Tab, Entree */ }
  #[test] fn envoi_incomplet_sort_en_3() { /* l'envoi insère 0 sur 2 : 3, « échec du système : SendInput a inséré 0
      événements sur 2 » */ }
  #[test] fn touche_barre_passe_par_la_disposition() { /* un Envoi : Maj gauche, 0xBF, 0xBF relâchée, Maj relâchée */ }
  #[test] fn comme_au_clavier_refuse_avant_tout_envoi() { /* « taper --fenetre appli-essai --comme-au-clavier --texte
      añ » : 1, « caractère absent du clavier : « ñ » », aucun Envoi de touche après l'activation */ }
  #[test] fn comme_au_clavier_refuse_quand_verr_maj_est_allumee() { /* verr_maj, « taper --fenetre appli-essai
      --comme-au-clavier --texte a » : 1, « Verr. Maj est allumée », aucun geste ; sans --comme-au-clavier, et pour
      « touche --fenetre appli-essai barre » : 0 */ }
  #[test] fn sous_commande_pas_encore_livree_sort_en_64() { /* « focus --fenetre x » : 64, « pas encore livrée :
      focus », une ligne « focus » de code 64 au journal */ }
  #[test] fn lecture_ratee_sort_en_3() { /* fenetres() en erreur « EnumWindows » : 3, « échec du système : EnumWindows » */ }
  ```
- [ ] **Étape 3 : les tests du journal dans `executer`, rouges.**
  ```rust
  #[test] fn chaque_appel_a_sa_ligne_meme_refuse() { /* un appel refusé : une ligne de code 1 */ }
  #[test] fn appel_mal_forme_a_sa_ligne_sans_ses_arguments() { /* « taper --fenetre xyz --txt secret » : 64, une
      seule ligne, « taper », sans « secret » ni « xyz » */ }
  #[test] fn aide_a_sa_ligne_si_le_journal_s_ouvre() { /* « --help » : 0, une ligne « aide » ; journal fermé : 0, aucune ligne */ }
  #[test] fn aucune_ligne_ne_porte_un_texte_tape() { /* « taper --fenetre appli-essai --texte secret-d-essai » : la
      ligne finit par « \tappli-essai\t0\t14\n » et ne contient pas « secret » */ }
  #[test] fn ajout_au_journal_rate_apres_un_geste_sort_en_3() { /* l'ajout échoue : 3, « échec du système : le
      journal n'a pas reçu sa ligne » */ }
  #[test] fn panique_sort_en_3_avec_sa_ligne() { /* fenetres() panique : 3, l'erreur « échec du système : panique »,
      une seule ligne au journal, « touche », de code 3 */ }
  ```
- [ ] **Étape 4 : voir rouge** : `cargo test --lib gestes`.
- [ ] **Étape 5 : écrire `bureau.rs`, `faux.rs` et `gestes.rs`.** `executer` enveloppe tout son corps dans
  `std::panic::catch_unwind(AssertUnwindSafe(…))`, puis suit, dans cet ordre, chaque garde portant en commentaire
  sa place : l'analyse (décision 5 de ce plan) ; le journal ; la fenêtre retenue, avec la liste de refus et la
  règle de KeePassXC (décision 3) ; le verrou (`ATTENTE_DU_VERROU`) ; le poste verrouillé (décision 10) ; le
  niveau d'intégrité (décision 1) ; les touches enfoncées ; Verr. Maj, pour `taper --comme-au-clavier` seulement
  (décision 9) ; l'activation (décision 8), qui restaure, puis le refus et la règle de KeePassXC rejugés sur W
  relu, puis les touches enfoncées de nouveau ; pour `taper --comme-au-clavier`, le calcul de tous les lots ;
  puis, pour chaque unité, le premier plan accepté (décision 11), l'envoi, et `CADENCE`. Les sondes lisent `maintenant` et appellent
  `attendre`, toutes bornées. Une `ErreurSysteme` d'une lecture de fenêtres, de processus ou du verrou fait sortir
  en 3, « échec du système : » puis son texte ; celle d'une lecture de texte est un usage (64). `fenetres` ne
  prend ni le verrou ni l'activation.
- [ ] **Étape 6 : voir vert**, `cargo test --workspace`, la mutation des trois modules, puis commit :
  `git add src/domaine/ .cargo/mutants.toml` et `git commit -m "feat(domaine): le bureau abstrait, les gardes, l'activation et la frappe"`.

### Tâche 9 : la couche Windows et `src/systeme.rs`

**Fichiers :** modifier `src/systeme.rs`, `src/bureau/mod.rs`, `src/lib.rs` ; créer `src/bureau/fenetres.rs`,
`src/bureau/processus.rs`, `src/bureau/clavier.rs`, `src/bureau/gardes.rs`.

**Interfaces :**
- Consomme : `Bureau`, `Disposition`, `executer` (Tâches 5 et 8) ; `fil_focalise` (Tâche 7).
- Produit : `pub fn lancer() -> u8` (Windows) ; dans `systeme.rs` : `chemin_journal() -> Result<PathBuf,
  ErreurSysteme>`, `ouvrir_journal() -> Result<File, ErreurSysteme>`, `maintenant() -> Moment`, `dormir(Duree)`,
  `date_locale() -> DateLocale`, `session() -> Option<String>`, `sensible_au_dpi()`.

Aucun test dans `src/bureau/` (décision 13 du plan maître) : cette tâche se prouve par la compilation et le lint
sur le poste, puis par les essais de bout en bout de la Tâche 11. Le seul test est le garde-fou de la sentinelle,
qui ne se voit rouge et vert que dans la sentinelle de la CI.

- [ ] **Étape 1 : le garde-fou.** Dans `src/systeme.rs` :
  ```rust
  #[ignore = "le garde-fou des chemins de données : la sentinelle de la CI le lance"]
  #[test]
  fn journal_tombe_dans_la_sentinelle() {
      let sentinelle = std::path::PathBuf::from(std::env::var_os("PILOTE_SENTINELLE")
          .expect("PILOTE_SENTINELLE manque : ce garde-fou ne tourne que dans la sentinelle de la CI"));
      let chemin = chemin_journal().unwrap();
      assert!(chemin.starts_with(&sentinelle), "{chemin:?} hors de {sentinelle:?}");
      assert_eq!(chemin, sentinelle.join(r"AppData\Local\pilote\journal.tsv"));
  }
  ```
- [ ] **Étape 2 : écrire `systeme.rs`.** `chemin_journal` : `SHGetKnownFolderPath(&FOLDERID_LocalAppData,
  KF_FLAG_DEFAULT, None)`, `CoTaskMemFree` du pointeur, puis `pilote\journal.tsv` ; il ne crée rien.
  `ouvrir_journal` : `chemin_journal`, `create_dir_all` du dossier `pilote`, puis `OpenOptions::new().append(true)
  .create(true).open` ; toute erreur rend `ErreurSysteme`. `maintenant` : `GetTickCount64` ; `dormir` :
  `std::thread::sleep` ; `date_locale` : `GetLocalTime` ; `session` : `CLAUDE_CODE_SESSION_ID`, absente ou vide
  donnant `None` ; `sensible_au_dpi` : `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`,
  son erreur ignorée.
- [ ] **Étape 3 : écrire la couche.** `BureauWindows` réalise `Bureau` et `Disposition` par des appels directs,
  sans condition qui décide :
  - le journal : `ouvrir_journal` garde le `File` de `systeme::ouvrir_journal` ; `ajouter_au_journal` y écrit la
    ligne en un seul `write_all` ;
  - `fenetres` : `EnumWindows` (le rappel rend toujours vrai et collecte), puis, par fenêtre, `IsWindowVisible`,
    `DwmGetWindowAttribute(DWMWA_CLOAKED)` (un `u32` non nul : masquée), `GetWindowLongPtrW(GWL_EXSTYLE)` et
    `WS_EX_TOOLWINDOW`, `GetWindowTextW`, `GetClassNameW`, `GetWindow(GW_OWNER)` dont toute erreur vaut « sans
    propriétaire », `IsIconic`, `GetWindowThreadProcessId`, l'AUMID par `SHGetPropertyStoreForWindow` et
    `PKEY_AppUserModel_ID` (vide ou en erreur : `None`), les PID des enfants par `EnumChildWindows(Some(hwnd),
    …)`, jamais avec `None` ;
  - `processus` : `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)`, `Process32FirstW`/`Process32NextW` (`dwSize`
    posé, la fin arrivant en erreur), l'heure de création par `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` et
    `GetProcessTimes` (`None` en erreur) ;
  - `niveau_integrite` : `OpenProcessToken(TOKEN_QUERY)` (sur `GetCurrentProcess()` pour `None`),
    `GetTokenInformation(TokenIntegrityLevel)` en deux appels, le dernier sous-identifiant du SID ; `None` en toute
    erreur ;
  - `premier_plan` : `GetForegroundWindow`, `None` si nulle, son PID, et la chaîne de ses propriétaires par
    `GetWindow(GW_OWNER)` répété, 16 au plus ;
  - `restaurer` : `ShowWindow(SW_RESTORE)` ; `demander_premier_plan` : `SetForegroundWindow` ; `est_reduite` :
    `IsIconic` ;
  - `envoyer` : un seul `SendInput` du lot, `cbsize = size_of::<INPUT>()` ; `Touche` et `Unicode` en
    `INPUT_KEYBOARD` (`INPUT_MOUSE` est la valeur par défaut, 0) ; `Touche` en `KEYBDINPUT` (`wVk`, `wScan`,
    `KEYEVENTF_EXTENDEDKEY` si étendue, `KEYEVENTF_KEYUP` si relâchée) ; `Unicode` en `KEYEVENTF_UNICODE`, plus
    `KEYEVENTF_KEYUP` si relâchée (`wVk` 0, `wScan` l'unité) ; `SourisNulle` en `INPUT_MOUSE`, `MOUSEINPUT {
    dwFlags: MOUSEEVENTF_MOVE, ..Default::default() }` ;
  - la disposition : le HKL du fil focalisé, relu à chaque appel : `GetGUIThreadInfo(0)` (`cbSize` posé) donne
    `hwndFocus` et `hwndActive`, `GetForegroundWindow` la troisième, et `fil_focalise` (Tâche 7) choisit ; puis
    `GetWindowThreadProcessId` et `GetKeyboardLayout` de ce fil. Si `fil_focalise` rend `None`, la disposition
    ne se lit pas : `scan` rend -1, et la frappe refuse ; `scan` : `VkKeyScanExW` ; `produit` : `ToUnicodeEx(vk, balayage, &etat, &mut tampon, 4,
    hkl)`, où l'état porte 0x80 sur VK_SHIFT et VK_LSHIFT pour le bit 1, sur VK_CONTROL et VK_LCONTROL pour le
    bit 2, sur VK_MENU et VK_RMENU pour le bit 4, et rien d'autre (VK_CAPITAL à 0) ; un résultat négatif rend
    `Morte`, sinon les unités écrites ; `balayage` : `MapVirtualKeyExW(MAPVK_VK_TO_VSC_EX)` ;
  - `touches_enfoncees` : `GetAsyncKeyState(vk) < 0` sur `vk::MODIFICATEURS` ; `verr_maj` :
    `GetKeyState(VK_CAPITAL) & 1` ;
  - `prendre_verrou` : `CreateMutexW(None, false, w!("Local\\pilote-gestes"))`, puis
    `WaitForSingleObject(h, delai)` : `WAIT_OBJECT_0` → `Pris`, `WAIT_ABANDONED` → `Repris`, `WAIT_TIMEOUT` →
    `Occupe` ; le handle reste dans `BureauWindows`, que `Drop` relâche (`ReleaseMutex` s'il est pris, puis
    fermeture), du même fil ;
  - `etat_session` : `OpenInputDesktop` et `GetUserObjectInformationW(UOI_NAME)` (`HANDLE(h.0)`) ;
    `WTSQuerySessionInformationW(WTS_CURRENT_SERVER_HANDLE, WTS_CURRENT_SESSION, WTSSessionInfoEx)`, les
    drapeaux et l'état de `WTSInfoExLevel1`, puis `WTSFreeMemory` ; chaque lecture ratée donne `None` ;
  - l'horloge, la date et la session par `systeme.rs` ; `lire_fichier` par `std::fs::read`, `lire_entree` par
    `std::io::stdin().read_to_end`.
- [ ] **Étape 4 : écrire `lancer`.** `std::panic::set_hook(Box::new(|_| {}))`, `systeme::sensible_au_dpi()`, puis
  `executer(&std::env::args_os().collect::<Vec<_>>(), &mut BureauWindows::new())` ; l'issue s'écrit (chaque ligne
  de sa sortie sur la sortie standard, son erreur sur la sortie d'erreur) et son code se rend.
- [ ] **Étape 5 : vérifier sur le poste** : `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo test --workspace` et `cargo build -p pilote`, qui ne lancent
  pas `pilote`. Linux se vérifie au job `verifier` du premier envoi. Commit : `git add src/lib.rs src/systeme.rs
  src/bureau/ src/domaine/` (le domaine, si clippy y a corrigé quelque chose) et
  `git commit -m "feat(bureau): la couche Windows, le journal et le lancement"`.

### Tâche 10 : l'application d'essai

**Fichiers :** modifier `essai/appli/Cargo.toml`, `essai/appli/src/main.rs`, `Cargo.lock` ; créer
`essai/appli/src/page.html`.

**Interfaces :**
- Produit : l'exécutable `appli-essai`, ses options (décision 12 de ce plan), le fichier d'événements : une ligne
  par événement, `<titre de la fenêtre>\t<JSON>`, écrite en un seul `write_all` sur un `File` non tamponné. Le
  JSON porte `genre` (`pret`, `touche`, `saisie`, `focus`, `permis`), `id`, `key`, `code`, `maj` et `valeur` ; le
  voleur écrit `vole`, le teneur `verrou pris`.

- [ ] **Étape 1 : la page.** `page.html`, servie par le protocole `essai://localhost/` :
  - dans cet ordre de tabulation : `depart` (un bouton, focalisé au chargement, sauf `?focus=<id>`), `champ` (un
    champ de texte), `recherche` (un champ, `aria-keyshortcuts="/"`), `zone` (une zone de texte), `motdepasse`
    (un champ de mot de passe), une case `case`, une liste `<select id="liste">` de trois options ;
  - trois groupes radio, `Couleur`, `Taille` et `Forme`, chacun avec un bouton « Toutes » ; deux groupes
    emboîtés nommés tous deux `Options` ; une liste `defilee` de 50 lignes dans un cadre qui défile ; une région
    `annonce` (`aria-live="polite"`) que le bouton `annoncer` remplit ;
  - les boutons `ouvrir` (le dialogue), `geler` (l'hôte figé 8 s) et `figer` (une boucle JavaScript de 8 s) ;
  - un écouteur de `keydown` sur le document, en capture : la touche `/`, sans Ctrl, Alt ni Méta, hors d'un
    champ de texte, donne le focus à `recherche` et empêche l'action par défaut, comme le prédicat de pupitre ;
  - le code :
    ```js
    const dire = (genre, e) => window.ipc.postMessage(JSON.stringify({ genre, id: e.target.id || e.target.tagName,
      key: e.key ?? '', code: e.code ?? '', maj: !!e.shiftKey, valeur: e.target.value ?? '' }));
    addEventListener('keydown', e => dire('touche', e), true);
    addEventListener('input', e => dire('saisie', e), true);
    addEventListener('focusin', e => dire('focus', e), true);
    addEventListener('DOMContentLoaded', () => {
      const id = new URLSearchParams(location.search).get('focus') || 'depart';
      document.getElementById(id).focus();
      window.ipc.postMessage(JSON.stringify({ genre: 'pret' }));
    });
    ```
- [ ] **Étape 2 : `main.rs`.** `essai/appli/Cargo.toml` reçoit, sous `[target.'cfg(windows)'.dependencies]`,
  `wry = "=0.57.1"`, `tao = "=0.37.2"` et `windows` (version et features de la Pile). Le squelette, aux appels
  relevés dans les sources du registre :
  ```rust
  #![cfg_attr(windows, windows_subsystem = "windows")]
  #[cfg(not(windows))]
  fn main() -> std::process::ExitCode { eprintln!("appli-essai ne tourne que sous Windows"); std::process::ExitCode::from(64) }
  #[cfg(windows)]
  fn main() -> std::process::ExitCode { fenetre::lancer() }
  #[cfg(windows)]
  mod fenetre {
      use std::{borrow::Cow, cell::RefCell, collections::HashMap, fs::File, io::Write, path::PathBuf, rc::Rc};
      use tao::event::{Event, WindowEvent};
      use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
      use tao::window::{Window, WindowBuilder, WindowId};
      use wry::http::{header::CONTENT_TYPE, Request, Response};
      use wry::{WebContext, WebView, WebViewBuilder, WebViewBuilderExtWindows};
      const PAGE: &str = include_str!("page.html");
      type Journal = Rc<RefCell<File>>;
      enum Ordre { Ouvrir, Geler(u64), Touche }
      // lancer : si le premier argument est --tenir-le-verrou, le second est le fichier, et rien d'autre ne se lit
      // (le mode du teneur) ; sinon <dossier> <fichier> puis les options ; File::create du fichier d'événements ;
      // EventLoopBuilder::<Ordre>::with_user_event().build() ; create_proxy() ;
      // WebContext::new(Some(PathBuf::from(dossier))) ; ouvrir("Appli d'essai", …) ; si --dialogue,
      // ouvrir("Dialogue d'essai", …) ; event_loop.run(…) : Ordre::Ouvrir ouvre le dialogue, Ordre::Geler(s)
      // dort s secondes sur ce fil, Ordre::Touche renouvelle AllowSetForegroundWindow si --permettre-premier-plan ;
      // CloseRequested retire la fenêtre, et la dernière fermée sort.
      fn ouvrir(titre: &str, cible: &EventLoopWindowTarget<Ordre>, contexte: &mut WebContext,
          proxy: &EventLoopProxy<Ordre>, journal: &Journal, focus: &str) -> Result<(Window, WebView), String> {
          let fenetre = WindowBuilder::new().with_title(titre).build(cible).map_err(|e| e.to_string())?;
          let (proxy, journal, titre) = (proxy.clone(), Rc::clone(journal), titre.to_owned());
          let vue = WebViewBuilder::new_with_web_context(contexte)
              .with_custom_protocol("essai".into(), |_id, _requete| Response::builder()
                  .header(CONTENT_TYPE, "text/html; charset=utf-8").body(Cow::Borrowed(PAGE.as_bytes())).unwrap())
              .with_url(format!("essai://localhost/?focus={focus}"))
              .with_browser_accelerator_keys(false).with_default_context_menus(false).with_devtools(false)
              .with_ipc_handler(move |requete: Request<String>| {
                  let corps = requete.body();
                  match corps.as_str() {
                      "ouvrir" => { let _ = proxy.send_event(Ordre::Ouvrir); return; }
                      c if c.starts_with("geler ") => {
                          if let Ok(s) = c["geler ".len()..].parse() { let _ = proxy.send_event(Ordre::Geler(s)); }
                          return;
                      }
                      _ => {}
                  }
                  if corps.contains("\"genre\":\"touche\"") { let _ = proxy.send_event(Ordre::Touche); }
                  let ligne = format!("{titre}\t{corps}\n");
                  let _ = journal.borrow_mut().write_all(ligne.as_bytes());
              })
              .build(&fenetre).map_err(|e| e.to_string())?;
          Ok((fenetre, vue))
      }
  }
  ```
  Les boutons `ouvrir` et `geler` envoient `ouvrir` et `geler 8` par `window.ipc.postMessage`, que le
  gestionnaire traduit en `Ordre`. Les options :
  - `--reduite` : `set_minimized(true)` sur la fenêtre principale après sa création ;
  - `--permettre-premier-plan` : `AllowSetForegroundWindow(ASFW_ANY)` au départ, puis à chaque `Ordre::Touche`,
    chaque résultat écrit en ligne `{"genre":"permis","valeur":"<vrai|faux>"}` ;
  - `--voler-premier-plan <fichier> <n>` : la fenêtre principale se crée `with_visible(false)` ; la boucle passe en
    `ControlFlow::WaitUntil` toutes les 20 ms, compte les lignes `"genre":"saisie"` du fichier ; à la n-ième, la
    fenêtre se montre (`set_visible(true)`) et `SetForegroundWindow` sur son `hwnd()` se répète toutes les 20 ms
    jusqu'à ce que `GetForegroundWindow` la rende (5 s au plus), puis la ligne `vole` s'écrit ;
  - le mode `--tenir-le-verrou <fichier>`, reconnu en premier argument, sans `<dossier> <fichier>` ni autre
    option : sans boucle tao, `CreateMutexW(None, false, w!("Local\\pilote-gestes"))`,
    `WaitForSingleObject(h, INFINITE)`, la ligne `verrou pris` dans le fichier, puis un sommeil sans fin ;
  - une option inconnue sort en 64. `Window::set_focus()` n'est jamais appelé.
- [ ] **Étape 3 : vérifier sur le poste** : `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo test --workspace` et `cargo build -p appli-essai`, sans jamais
  lancer `appli-essai`. Commit :
  `git add Cargo.lock essai/appli/Cargo.toml essai/appli/src/main.rs essai/appli/src/page.html` et
  `git commit -m "feat(essai): l'application d'essai, sa page et ses options"`.

### Tâche 11 : le job `windows`, la sentinelle et les essais de bout en bout

**Fichiers :** créer `tests/commun/mod.rs`, `tests/essais_clavier.rs`, `tests/essais_gardes.rs` ; modifier
`tests/statique.rs`, `.github/workflows/ci.yml`, et, selon l'étape 7, `.cargo/mutants.toml`, `src/domaine/`,
`essai/appli/Cargo.toml` et `essai/appli/src/main.rs`.

**Interfaces :**
- Consomme : l'exécutable `pilote` (`CARGO_BIN_EXE_pilote`), `appli-essai.exe` dans le même dossier.
- Produit (`tests/commun/mod.rs`, sous `#![cfg(windows)]` puis `#![allow(dead_code, reason = "chaque fichier
  d'essai n'emploie qu'une partie du module commun")]`) :
  ```rust
  pub fn exiger_la_ci()                                   // panique si GITHUB_ACTIONS ne vaut pas « true »
  pub fn dossier_jetable(nom: &str) -> PathBuf            // %RUNNER_TEMP%\pilote-essais\<nom>-<pid>, vidé puis créé
  pub struct Profil { pub racine: PathBuf }               // AppData\Roaming, AppData\Local, Temp créés
  impl Profil { pub fn neuf(nom: &str) -> Profil; pub fn variables(&self) -> Vec<(&'static str, OsString)> }
  pub struct Appli { pub nom: String, pub pid: u32, pub evenements: PathBuf, /* l'enfant, le profil */ }
  impl Appli { pub fn attendre(&self, motif: &str, delai: Duration) -> bool; pub fn lignes(&self) -> Vec<String> }
  impl Drop for Appli { /* tue l'enfant, l'attend */ }
  pub fn lancer_appli(nom: &str, options: &[&str]) -> Appli     // copie sous <nom>.exe, sauf « appli-essai »
  pub struct Teneur { /* l'enfant, son profil */ }
  impl Teneur { pub fn tuer(self) }                       // tue l'enfant et l'attend
  impl Drop for Teneur { /* tue l'enfant s'il vit encore, l'attend */ }
  pub fn tenir_le_verrou() -> Teneur
  pub struct Sortie { pub code: i32, pub sortie: Vec<u8>, pub erreur: Vec<u8>, pub duree: Duration }
  pub fn pilote(profil: &Profil, args: &[&str], entree: Option<&[u8]>) -> Sortie
  pub enum Shell { Powershell51, GitBash }
  pub fn shell(shell: Shell, script: &str, profil: &Profil) -> Sortie
  pub fn exiger_la_disposition_francaise(appli: &Appli, profil: &Profil)
  pub fn est_reduite(pid: u32) -> bool
  pub fn reduire_les_parametres() -> Option<String>      // le titre lu du cadre des Paramètres, réduit
  pub fn fenetre_sans_titre_de(processus: &str) -> bool  // une fenêtre de premier niveau visible et sans titre
  pub struct VerrMaj; impl VerrMaj { pub fn allumer() -> VerrMaj }       // Drop l'éteint
  ```
  Chaque fonction publique, sauf `exiger_la_ci` elle-même, et chaque constructeur commencent par
  `exiger_la_ci()`. `lancer_appli` pose `Appli::nom` au nom reçu. `tenir_le_verrou` crée `Profil::neuf("teneur")`,
  le garde dans `Teneur`, lance exactement `appli-essai --tenir-le-verrou <profil.racine>\verrou.txt` sous
  `profil.variables()`, puis attend la ligne `verrou pris` dans ce fichier par sondage, 10 s au plus, sinon
  panique avec ce qu'il a lu. `reduire_les_parametres` trouve le cadre des Paramètres avant de le réduire (par son
  AUMID ou son enfant `SystemSettings`, qui se détache une fois le cadre réduit), lit son titre,
  `ShowWindow(SW_MINIMIZE)`, et rend ce titre ; `None` s'il ne le trouve pas. Chaque processus lancé
  reçoit `Profil::variables()` : `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, `TEMP`, `TMP`, et `PATH`, qui commence
  par le dossier de `CARGO_BIN_EXE_pilote`. `lancer_appli` panique avec « appli-essai.exe manque à côté de
  pilote : cargo build -p appli-essai » si le fichier n'existe pas, et attend la ligne `pret` 20 s au plus, sinon
  panique avec les lignes lues. `shell` lance `%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe
  -NoProfile -Command <script>` ou `%ProgramFiles%\Git\bin\bash.exe -c <script>`, et panique si le fichier
  manque. `exiger_la_disposition_francaise` fait, dans cet ordre, cinq preuves, chacune avec sa panique :
  1. `pilote activer --fenetre <appli.nom>` sort en 0, sinon « activer a rendu <code> : <erreur> » ;
  2. le PID de `GetForegroundWindow` est celui de l'appli, sinon « premier plan sur le PID <p>, pas sur l'appli
     <q> » ;
  3. `GetGUIThreadInfo(0).hwndFocus` n'est pas nul, sinon « aucune fenêtre focalisée dans l'appli » ;
  4. le processus de `hwndFocus` est `msedgewebview2.exe`, sinon « fenêtre focalisée hors de msedgewebview2 :
     <classe> <processus> » ;
  5. le HKL de son fil vaut 0x040C040C, `VkKeyScanExW('@')` 0x0630 et `VkKeyScanExW('/')` 0x01BF, sinon
     « disposition non française sur le runner : HKL 0x…, '@' 0x…, '/' 0x… ».

- [ ] **Étape 1 : les règles de la décision 13 du plan maître dans le contrôle statique, rouges.** Dans
  `tests/statique.rs`, cinq règles :
  - A : chaque `#[test]` de `tests/essais_*.rs` porte, sur la ligne d'avant, `#[ignore = "il pilote le bureau :
    la CI le lance"]` ; faute : « <chemin>:<ligne> : #[test] sans #[ignore = "il pilote le bureau : la CI le
    lance"] » ;
  - B : chaque `#[test]` hors de `src/domaine/`, de `tests/statique.rs` et de `tests/essais_*.rs` (que A couvre)
    porte, sur la ligne d'avant, un `#[ignore = …]` ; faute : « <chemin>:<ligne> : #[test] sans #[ignore] » ;
  - C : aucun fichier de `tests/` hors de `tests/commun/mod.rs` et de `tests/statique.rs` ne nomme les mots de
    lancement `CARGO_BIN_EXE`, `Command::new`, `std::process` ni `spawn`, des préfixes bornés à gauche seulement
    (`CARGO_BIN_EXE_pilote` est vu) ; faute : « <chemin>:<ligne> contient « <mot> » » ;
  - D : aucun `#[test]` sous `src/bureau/` ; faute : « <chemin>:<ligne> : #[test] dans la couche Windows » ;
  - E : `BureauWindows` n'est nommé que sous `src/bureau/`, dans `src/lib.rs` et dans `tests/statique.rs` ;
    faute : « <chemin>:<ligne> contient « BureauWindows » ».

  Les mots de lancement forment la `const MOTS_DE_LANCEMENT`, qu'un test à table parcourt.
  ```rust
  #[test] fn refuse_un_essai_sans_ignore() { refuse("tests/essais_clavier.rs", "#[test]\nfn a() {}",
      "tests/essais_clavier.rs:1 : #[test] sans #[ignore = \"il pilote le bureau : la CI le lance\"]"); }
  #[test] fn refuse_un_test_sans_ignore_hors_du_domaine() { refuse("src/systeme.rs", "#[test]\nfn a() {}",
      "src/systeme.rs:1 : #[test] sans #[ignore]"); }
  #[test] fn admet_un_test_ignore_hors_du_domaine() { admet("src/systeme.rs", "#[ignore = \"x\"]\n#[test]\nfn a() {}"); }
  #[test] fn chaque_mot_de_lancement_a_sa_plantation() { for mot in MOTS_DE_LANCEMENT {
      refuse("tests/autre.rs", &format!("let x = {mot};"), &format!("tests/autre.rs:1 contient « {mot} »")); }
      refuse("tests/essais_gardes.rs", "env!(\"CARGO_BIN_EXE_pilote\")",
          "tests/essais_gardes.rs:1 contient « CARGO_BIN_EXE »"); }
  #[test] fn admet_un_lancement_dans_le_module_commun() { admet("tests/commun/mod.rs", "Command::new(\"x\")"); }
  #[test] fn refuse_un_test_dans_la_couche_windows() { refuse("src/bureau/clavier.rs", "#[ignore = \"x\"]\n#[test]\nfn a() {}",
      "src/bureau/clavier.rs:2 : #[test] dans la couche Windows"); }
  #[test] fn refuse_le_bureau_reel_hors_de_sa_place() { refuse("src/domaine/gestes.rs", "BureauWindows::new()",
      "src/domaine/gestes.rs:1 contient « BureauWindows »"); }
  ```
  Voir rouge (`cargo test --test statique`), écrire les règles, voir vert.
- [ ] **Étape 2 : les essais du clavier** (`tests/essais_clavier.rs`, `#![cfg(windows)]`, `mod commun;`, chaque
  test `#[ignore = "il pilote le bureau : la CI le lance"]`) :
  - `taper_des_accents_en_unicode` : l'appli `--focus champ`, `taper --texte "Élève à l'œuvre : ça coûte 5 €"` ;
    la dernière `saisie` de `champ` porte ce texte ;
  - `taper_hors_du_plan_de_base_envoie_la_paire` : de même, `😀` ;
  - `taper_crlf_donne_un_seul_entree` : `--focus zone`, `--fichier` d'un texte `a\r\nb` ; la valeur vaut `a\nb`
    (`"valeur":"a\nb"` dans le JSON) ;
  - `taper_arobase_par_altgr` : `--focus champ`, `exiger_la_disposition_francaise`, puis `taper
    --comme-au-clavier --texte @` ; la valeur vaut `@`, et la `touche` porte `"code":"Digit0"` ;
  - `touche_barre_declenche_le_raccourci` : focus par défaut sur `depart`, la disposition exigée, `touche barre` ;
    un `focus` de `recherche` suit, et la `touche` porte `"key":"/"` ;
  - `touche_tab_deplace_le_focus` : depuis `depart`, `touche Tab` ; un `focus` de `champ` suit ;
  - `comme_au_clavier_refuse_un_caractere_absent` : `--focus champ`, `ñ` ; 1, le message, aucune `saisie` ;
  - `comme_au_clavier_refuse_quand_verr_maj_est_allumee` : sous `VerrMaj::allumer()` ; 1, le message ;
  - `taper_depuis_powershell_5_1_un_fichier_utf16` : `--focus champ` ; `shell(Shell::Powershell51, …)` vérifie
    `$PSVersionTable.PSVersion.Major -eq 5`, écrit `été` par `Out-File` (UTF-16 à BOM), appelle `pilote taper
    --fenetre appli-essai --fichier <f>`, et sort de `$LASTEXITCODE` après chaque appel ; puis il pose
    `[Console]::OutputEncoding = [Text.Encoding]::UTF8`, lit `$e = (& pilote activer --fenetre absent 2>&1 |
    ForEach-Object { "$_" }) -join ''`, et sort en erreur si `$LASTEXITCODE -ne 2` ou si `$e -ne 'aucune fenêtre
    de « absent »'` ; le script sort en 0, et la valeur vaut `été` ;
  - `taper_depuis_git_bash_l_entree_standard` : `--focus champ`, la disposition exigée ; `<F>` est un chemin du
    dossier jetable, écrit avec des `/`, que le test met dans le script ; `shell(Shell::GitBash, "case $(uname -s) in MINGW*) ;; *)
    exit 99 ;; esac && printf 'été' | pilote taper --fenetre appli-essai - && pilote touche --fenetre appli-essai
    Maj+Tab && pilote touche --fenetre appli-essai barre && { pilote activer --fenetre absent 2>'<F>'; test $? -eq
    2; }", …)` sort en 0 (les accolades gardent la chaîne de `&&`) ; dans l'ordre : la `saisie` `été` de `champ`,
    un `focus` de `depart`, puis un `focus` de `recherche` et la `touche` `"key":"/"` ; les octets de `<F>` valent
    exactement l'UTF-8 de « aucune fenêtre de « absent » » suivi de `\n`, sans BOM.
- [ ] **Étape 3 : les essais des gardes** (`tests/essais_gardes.rs`, même forme) :
  - `activer_une_fenetre_d_arriere_plan` : `appli-essai`, puis une copie `autre` ; `activer --fenetre
    appli-essai` ; `fenetres` donne `premier-plan=oui` à la ligne `processus=appli-essai` ;
  - `activer_restaure_une_fenetre_reduite` : lancée `--reduite` ; après `activer`, `est_reduite` est faux, et
    `fenetres` lui donne le premier plan ;
  - `activer_puis_taper_ne_perd_aucun_caractere` : `--focus champ` ; un premier `activer`, `fenetres` montre
    `premier-plan=oui` ; puis `activer` et `taper --texte bonjour` ; la valeur vaut `bonjour` ;
  - `taper_refuse_quand_une_autre_fenetre_vole_le_premier_plan` : `appli-essai --permettre-premier-plan --focus
    champ`, puis, sa ligne `pret` reçue, une copie `voleur --voler-premier-plan <fichier d'appli-essai> 100` ;
    `fenetres` montre le premier plan à « Appli d'essai » ; puis un `taper` de 10 000 caractères ; la ligne
    `vole` paraît, le code est 1, le message commence par « le premier plan a changé : » et compte k unités sur
    10000 avec k < 10000, et la valeur reçue fait au plus k caractères. Si la première exécution montre que le
    runner ne cède jamais le premier plan au voleur, s'arrêter et le dire à Edouard : le test du domaine en tient
    alors lieu, avec son accord, et la PR le déclare ;
  - `deux_fenetres_titrees_sans_option_titre_sortent_en_2` : `--dialogue` ; `touche Tab` sort en 2 et nomme
    `<Appli d'essai>` et `<Dialogue d'essai>` ; avec `--titre "Appli d'essai"`, 0, et le `keydown` Tab paraît
    sous « Appli d'essai », aucun sous « Dialogue d'essai » ;
  - `touche_avec_une_liste_ouverte_vise_la_fenetre_principale` : `--focus liste`, `touche Alt+Bas` (0), puis
    `fenetre_sans_titre_de("msedgewebview2")` est vrai, `touche Echap` (0), et elle devient fausse. Si la liste
    ouverte ne paraît pas comme une fenêtre, s'arrêter et le dire à Edouard : l'essai ne perd cette preuve
    qu'avec son accord, et la PR le déclare ;
  - `copie_systemsettings_refusee` : une copie `SystemSettings`, et une `appli-essai` permise à côté ; `activer`
    et `taper` vers la copie sortent en 1, « fenêtre refusée : processus SystemSettings », et `fenetres` n'a
    aucune ligne `processus=SystemSettings` ; pendant que la copie tourne, `touche --fenetre appli-essai Tab` sort
    en 0, et `fenetres` porte la ligne `processus=appli-essai` (une énumération des enfants de tout le bureau
    refuserait tout) ;
  - `copie_keepassxc_refuse_taper_et_touche` : une copie `KeePassXC` ; `taper` et `touche Tab` sortent en 1 avec
    le message de la décision 3, et aucun `keydown` ne paraît ;
  - `parametres_reduits_restent_refuses` : `start ms-settings:` par `shell(Shell::Powershell51, …)`, la fenêtre
    attendue 15 s, puis `reduire_les_parametres()` rend le titre lu du cadre (« Settings » sur un runner en
    anglais) ; `activer --fenetre ApplicationFrameHost` sort en 1, l'erreur valant exactement « fenêtre refusée :
    processus SystemSettings », ou en 2, l'erreur ne contenant pas `<titre>`, le titre lu entre chevrons comme le
    message de code 2 cite les fenêtres ; aucune ligne de `fenetres` ne commence par `[Fenetre] <titre>
    processus=ApplicationFrameHost` (le titre échappé). Si la première exécution montre que le
    runner n'ouvre pas les Paramètres (`None`), l'essai se retire, la PR le dit, et le test du domaine en tient
    lieu ;
  - `deux_gestes_en_meme_temps_le_second_attend_5_s_puis_refuse` : `tenir_le_verrou()`, puis `touche Tab` sur
    l'appli ; 1, « un autre geste est en cours depuis 5 s », une durée d'au moins 5 s et de moins de 9 s ;
  - `un_verrou_abandonne_est_repris` : `tenir_le_verrou()` (sa ligne reçue), puis `Teneur::tuer`, puis
    `touche Tab` sort en 0 ;
  - `journal_tombe_dans_le_profil_redirige` : après `fenetres` puis `taper --texte secret-d-essai`, le fichier
    `<profil>\AppData\Local\pilote\journal.tsv` a deux lignes de six champs, `fenetres` puis `taper`, la seconde
    finissant par `\t0\t14`, et aucune ne contient `secret` ;
  - `journal_verrouille_refuse_avant_tout_geste` : dans un profil neuf, le test crée le dossier
    `<profil>\AppData\Local\pilote` (`create_dir_all`), puis ouvre `journal.tsv` par
    `OpenOptions::new().write(true).create(true).share_mode(0)` (`std::os::windows::fs::OpenOptionsExt`), et le
    garde ouvert pendant l'appel ; `touche Tab` sort en 1, « journal inaccessible : ouvert dans une autre
    application ? », et aucun `keydown` ne paraît ;
  - `sortie_et_erreurs_en_utf8_sans_bom` : `activer --fenetre absent` ; l'erreur reçue vaut exactement les octets
    UTF-8 de « aucune fenêtre de « absent » » suivis d'un saut de ligne, sans BOM ; `fenetres`, l'appli lancée,
    écrit une sortie qui se lit en UTF-8 et ne commence pas par un BOM ;
  - `version_porte_le_commit` : `pilote --version` sort en 0 et écrit `pilote 0.1.0 (` puis les 12 premiers
    chiffres de `GITHUB_SHA` et `)`.
- [ ] **Étape 4 : le job `windows`.** Dans `ci.yml`, après `verifier` :
  ```yaml
    windows:
      runs-on: windows-latest
      timeout-minutes: 40 # provisoire : mesuré à la Tâche 12
      permissions:
        contents: read
      steps:
        - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        - name: Installer la chaîne de rust-toolchain.toml
          run: rustup toolchain install
        - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
        - name: Lint
          run: cargo clippy --workspace --all-targets -- -D warnings
        - name: Tests
          run: cargo test --workspace
        - name: Construire pilote et l'application d'essai
          run: cargo build -p pilote -p appli-essai
        - name: La disposition française
          shell: powershell
          run: |
            Set-WinUserLanguageList (New-WinUserLanguageList fr-FR) -Force
            Set-WinDefaultInputMethodOverride -InputTip '040C:0000040C'
            Get-WinUserLanguageList
            Get-WinDefaultInputMethodOverride
        - name: La sentinelle des chemins de données, puis les essais
          run: |
            $cargo = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
            $rustup = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE '.rustup' }
            $essai = Join-Path $env:RUNNER_TEMP 'sentinelle-essai'
            New-Item -ItemType Directory -Force (Join-Path $essai 'a') | Out-Null
            (New-Item -ItemType File (Join-Path $essai 'a\cache')).Attributes = 'Hidden'
            if (-not (Get-ChildItem -LiteralPath $essai -Recurse -File -Force)) { throw "l'essai ne voit pas le fichier caché" }
            $s = Join-Path $env:RUNNER_TEMP 'sentinelle'
            New-Item -ItemType Directory -Force (Join-Path $s 'AppData\Roaming'), (Join-Path $s 'AppData\Local') | Out-Null
            $env:CARGO_HOME = $cargo
            $env:RUSTUP_HOME = $rustup
            $env:USERPROFILE = $s
            $env:APPDATA = Join-Path $s 'AppData\Roaming'
            $env:LOCALAPPDATA = Join-Path $s 'AppData\Local'
            $env:PILOTE_SENTINELLE = $s
            cargo test --workspace -- --include-ignored --test-threads=1
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            $paru = Get-ChildItem -LiteralPath $s -Recurse -File -Force
            if ($paru) { $paru | ForEach-Object { $_.FullName.Substring($s.Length) }; throw 'un fichier a paru dans la sentinelle' }
        - name: La version suit le commit
          run: |
            $env:CARGO_HOME = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
            $env:RUSTUP_HOME = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE '.rustup' }
            $p = Join-Path $env:RUNNER_TEMP 'profil-version'
            New-Item -ItemType Directory -Force (Join-Path $p 'AppData\Roaming'), (Join-Path $p 'AppData\Local') | Out-Null
            $env:USERPROFILE = $p
            $env:APPDATA = Join-Path $p 'AppData\Roaming'
            $env:LOCALAPPDATA = Join-Path $p 'AppData\Local'
            git switch -c essai-version
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            foreach ($tour in 1, 2) {
              git -c user.name=ci -c user.email=ci@invalid commit --allow-empty -m "essai $tour"
              if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
              # Au premier tour, la référence se compacte : le second commit recrée son fichier sans toucher
              # HEAD ni packed-refs, et seul logs/HEAD le dit à build.rs.
              if ($tour -eq 1) { git pack-refs --all; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } }
              cargo build -p pilote
              if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
              $attendu = git rev-parse --short=12 HEAD
              $version = & .\target\debug\pilote.exe --version
              if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
              if ($version -ne "pilote 0.1.0 ($attendu)") { throw "tour ${tour} : « $version », attendu $attendu" }
            }
        - name: Déposer les essais
          if: failure()
          uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
          with:
            name: pilote-essais
            path: ${{ runner.temp }}/pilote-essais/
            if-no-files-found: warn
  ```
- [ ] **Étape 5 : commit**, après `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` et `cargo test --workspace` (les essais restent ignorés) : `git add tests/
  .github/workflows/ci.yml` puis
  `git commit -m "test: les essais de bout en bout du clavier et des gardes, et le job windows"`.
- [ ] **Étape 6 : le premier envoi** : `git push origin docs/tranche1` (ou le nom de la branche de la tranche),
  puis la PR ouverte en brouillon ; lire `verifier`, `windows` et `mutants` (`gh pr checks`, puis `gh run view
  --log-failed`). Un essai qui échoue se lit dans ses artefacts, se corrige, et ne se relance jamais sans
  changement.
- [ ] **Étape 7 : les suites du premier envoi**, chacune commitée avec ses chemins, puis poussée :
  - si `exiger_la_disposition_francaise` échoue sur la disposition (sa cinquième preuve), essayer dans l'ordre,
    un envoi chaque fois : `SystemParametersInfoW(SPI_SETDEFAULTINPUTLANG)` dans l'étape de la disposition (par
    un petit script PowerShell qui l'appelle) ; puis `LoadKeyboardLayoutW("0000040C", KLF_ACTIVATE)` dans
    l'application d'essai, sa fenêtre au premier plan, avant la WebView, derrière une option `--azerty`, la
    feature `Win32_UI_Input_KeyboardAndMouse` ajoutée à `windows` dans `essai/appli/Cargo.toml` (`appli-essai`
    déclare la feature de l'API qu'il appelle, sans compter sur celles que tao et wry posent) : `git add
    essai/appli/Cargo.toml essai/appli/src/main.rs tests/commun/mod.rs
    .github/workflows/ci.yml`. Si aucun ne prend, s'arrêter et demander à Edouard (décision 16 du plan maître) ;
  - si elle échoue sur sa quatrième preuve (la fenêtre focalisée hors de `msedgewebview2`), s'arrêter et le dire à
    Edouard : la décision 2 repose sur ce fait, et les remèdes de la disposition n'y peuvent rien ; sur ses trois
    premières, l'essai se corrige comme tout essai qui échoue (étape 6) ;
  - chaque mutant survivant du job `mutants` se tue par un test du domaine ; un mutant équivalent ou qui boucle
    s'exclut par son nom dans `.cargo/mutants.toml`, avec sa raison (`git add src/domaine/ .cargo/mutants.toml`).

### Tâche 12 : les ADR, `CLAUDE.md`, les délais et la règle de protection

**Fichiers :** créer `docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md`,
`docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md` ; modifier
`docs/decisions/0002-garde-fous-dans-le-code.md`, `docs/decisions/0004-interface-sortie-codes-et-selecteur.md`
(le statut seul), `CLAUDE.md`, `.github/workflows/ci.yml` et `.github/workflows/mutants-complet.yml` (les délais),
et `docs/registre.md` si une leçon paraît.

- [ ] **Étape 1 : ADR-0005.** `# ADR-0005 — Les garde-fous : la fenêtre retenue et son processus, jugés dans le
  code`, statut « accepté le <date à laquelle Edouard a accepté ce plan> », contexte amont : ADR-0002, le plan
  maître du lot 1 (décisions 5, 6 et 9), ce plan (décisions 1, 3, 5, 8, 10, 11 et 13). Il reprend tout ADR-0002,
  avec la fenêtre retenue, le premier plan accepté, ce qu'on lit borné à W, la règle de KeePassXC étendue à
  `touche` et son refus entier d'ici la tranche 2, le journal et son ouverture, l'activation, la cadence, la
  comparaison des niveaux d'intégrité, le poste verrouillé.
- [ ] **Étape 2 : ADR-0006.** `# ADR-0006 — L'interface : une ligne par résultat, des codes de sortie fixes,
  précisés`, même statut, contexte amont : ADR-0004, le plan maître (décisions 7, 8, 11 et 12, et les candidates
  de la décision 5), ce plan (décisions 4, 6 et 7). Il reprend tout ADR-0004, avec le vocabulaire, les
  échappements, les codes précisés, le tableau des messages, les bornes, l'attente, les options de `focus`, le
  sens de `--dans`, les candidates.
- [ ] **Étape 3 : les anciens ADR.** Leur seul statut change : « remplacé par ADR-0005 le <date> », « remplacé par
  ADR-0006 le <date> ».
- [ ] **Étape 4 : `CLAUDE.md`.** L'état (« Tranche 1 du lot 1 faite : le socle, le clavier, l'application
  d'essai ») ; les commandes :
  ```
  cargo fmt --all --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace                  # le domaine ; les essais de bout en bout restent ignorés
  cargo build -p pilote -p appli-essai
  cargo install --locked --path .         # depuis un clone à jour : git fetch, puis main..origin/main vide
  ```
  la règle : « Ne jamais lancer `cargo test -- --ignored` ni `--include-ignored` sur le poste, ni `pilote` ou
  `appli-essai` hors de `--version` et `--help` avant l'essai sur le poste : les essais de bout en bout pilotent
  le bureau, et la CI seule les lance. » ; dans « Ce qui fait foi », les modules (`src/domaine/`, `src/bureau/`,
  `src/systeme.rs`), le trait `Bureau` (`src/domaine/bureau.rs`), l'ordre des gardes (`src/domaine/gestes.rs`,
  ADR-0005) et la sortie (ADR-0006) ; les six contrôles requis (`socle / secrets`, `socle / confidentialite`,
  `texte-pr`, `verifier`, `windows`, `mutants`) ; l'invariant 3, « hors de la fenêtre retenue » au lieu de « hors
  de son arbre de processus » ; les renvois d'ADR-0002 devenus ADR-0005.
- [ ] **Étape 5 : les délais.** Lire les durées du premier passage à froid de `verifier`, `windows` et `mutants`
  (`gh run view <id> --json jobs`), puis écrire chaque délai à deux fois la plus longue, en minutes entières,
  avec son commentaire (« deux fois N min M s, le passage à froid du <date> »). `mutants-complet.yml` prend celui
  de `mutants`.
- [ ] **Étape 6 : le registre**, par `/consigner`, si une leçon de la tranche dépasse son code.
- [ ] **Étape 7 : commit** : `git add docs/decisions/ CLAUDE.md .github/workflows/ci.yml
  .github/workflows/mutants-complet.yml` (et `docs/registre.md` s'il change), puis
  `git commit -m "docs: ADR-0005 et ADR-0006, CLAUDE.md et les délais de la CI"`, et l'envoi.
- [ ] **Étape 8 : la revue de la branche entière**, par une session neuve, sur la tête commitée : chaque règle
  d'ADR-0002 et d'ADR-0004 retrouvée dans ADR-0005 et ADR-0006, chaque test de ce plan présent, chaque mot que la
  tranche périme cherché dans tout le dépôt. Ses constats confirmés se corrigent, se commitent, et se relisent
  sur la nouvelle tête.
- [ ] **Étape 9 : la PR quitte le brouillon** (`gh pr ready`), une fois la revue de la branche faite, ses
  corrections relues sur la nouvelle tête, et `gh pr checks` tout vert.
- [ ] **Étape 10 : la règle de protection préparée.** Hors du dépôt, dans le dossier temporaire de la session,
  depuis Git Bash :
  ```sh
  gh api repos/EdouardZemb/pilote/rulesets/27788996 --jq '{name, target, enforcement, bypass_actors, conditions, rules: [.rules[] | if .type == "required_status_checks" then .parameters.required_status_checks |= (. + ([{context:"verifier"},{context:"windows"},{context:"mutants"}] - .)) else . end]}' > regle-main.json
  ```
  Relire que les six contextes y sont. La ligne d'Edouard, après la fusion :
  `gh api -X PUT repos/EdouardZemb/pilote/rulesets/27788996 --input <chemin>/regle-main.json`. Ensuite, la
  session lit `gh api repos/EdouardZemb/pilote/rules/branches/main` et y vérifie les six contextes.

---

## La PR de la tranche

Elle dit « Partie de #2 » et porte « ## Écarts à la spec », chaque écart sous la forme de la méthode : ceux de la
tranche 1 que nomme la décision 20 du plan maître, datés du 21/11/2026 ; et le refus entier de KeePassXC
(décision 3 de ce plan ; promesse : ADR-0002, « KeePassXC », et le design, décision 10), daté du jour où Edouard
a accepté ce plan. Elle rappelle les changements au plan maître (décisions 1, 2, 3, 11 et 14), acceptés avec ce
plan, et nomme la dépendance `windows` d'`appli-essai`. Après la fusion : la règle de protection par le geste
d'Edouard, puis `gh workflow run mutants-complet.yml`.
