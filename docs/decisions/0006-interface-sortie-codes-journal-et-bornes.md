# ADR-0006 — L'interface : une ligne par résultat, des codes de sortie fixes, précisés

- **Statut** : accepté le 2026-11-21
- **Contexte amont** : [ADR-0004](0004-interface-sortie-codes-et-selecteur.md), qu'il remplace ; le plan maître du lot 1, `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md`, décisions 7, 8, 11 et 12, et ce que la décision 5 dit des candidates ; le plan de sa tranche 1, `.claude/plans/lot1-tranche1-le-socle-le-clavier-et-l-application-d-essai.md`, décisions 4, 6 et 7, et, pour la frappe par la disposition du clavier, 2 et 9
- **Amendé** le 2026-11-28 : tranche 2 du lot 1 — un changement amende cet ADR en place ; une valeur entre guillemets dès qu'elle porte un caractère blanc d'Unicode resté tel quel après l'échappement

## Contexte

Une session lit ce que rend `pilote`, enchaîne ses appels selon leurs codes de sortie, et désigne
les éléments d'une fenêtre par un sélecteur. Elle l'appelle depuis PowerShell 5.1 comme depuis Git
Bash. Un format flou, un code qui change de sens ou un sélecteur qui choisit au hasard ferait agir
la session sur le mauvais élément.

ADR-0004 a fixé une ligne par résultat, des codes qui ont chacun un sens et un sélecteur exact. Le
plan maître du lot 1 et le plan de sa tranche 1 ont dit ce qu'il laissait ouvert : le vocabulaire et
les échappements de la sortie, les codes précisés et chaque message mot pour mot, les candidates,
les bornes, l'attente, les options de `focus`, le sens de `--dans`, et la frappe par la disposition
du clavier. Comme un ADR ne se réécrit pas, celui-ci reprend tout ADR-0004 avec ces précisions,
qu'Edouard a acceptées avec ces plans, le 2026-11-21.

## Options

1. **Du JSON seulement.** Précis, mais lourd à lire pour une session dans un terminal, et le
   design le laisse optionnel.
2. **Du texte libre, comme le script de l'essai.** Lisible, mais sans forme stable.
3. **Les erreurs de `clap` telles qu'il les écrit.** Rien à traduire, mais elles sont en anglais,
   sur plusieurs lignes, et sortent en 2, le code d'« introuvable ».
4. **Une ligne de texte par résultat, d'une forme fixe, des codes de sortie qui ont chacun un sens,
   et des messages d'une ligne, en français, que les tests vérifient mot pour mot.**

## Décision

L'option 4.

### La sortie

- Une ligne par résultat, sur la sortie standard : le type entre crochets, le nom entre chevrons,
  puis des champs `clé=valeur`, par exemple `[Edit] <Chercher dans le texte des idées> id=recherche
  raccourci=/`.
- Le type d'un élément est celui de l'automatisation, sans « ControlType » (`Edit`, `Button`,
  `Group`, `RadioButton`, `Document`…) ; les autres lignes ont leur type : `Fenetre`, `Capture`,
  `Processus`, `Dossier`, `Annonce`, `Ecoute`.
- Les clés sont en ASCII, sans accent : `id`, `classe`, `valeur`, `raccourci` (la touche déclarée),
  `acces`, `annonce` (`polite` ou `assertive`), `etat` (une liste, séparée par des virgules, parmi
  `coche`, `decoche`, `indetermine`, `choisi`, `inactif`, `deplie`, `replie`, `lecture-seule`),
  `dans`, `rang`, `motdepasse=oui` (qui remplace la valeur), `processus`, `pid`,
  `premier-plan=oui`.
- `arbre` indente ses lignes dans l'ordre du document, et dit le réglage d'annonce d'un élément qui
  en porte un (`annonce=polite`). `focus` et `trouver` disent le groupe qui contient l'élément
  (`dans=<nom>`) : trois boutons « Toutes » de trois groupes s'y distinguent.
- La sortie est en UTF-8, sans BOM. Un refus ou une erreur d'usage se dit en une ligne, en français,
  sur la sortie d'erreur ; rien ne s'écrit alors sur la sortie standard.

**Les échappements.** Dans un nom, `\` s'écrit `\\`, un retour à la ligne `\n`, un retour chariot
`\r`, une tabulation `\t`, `>` s'écrit `\>`, et tout autre caractère de contrôle, U+2028 et U+2029
compris, `\u{XXXX}`, en chiffres hexadécimaux majuscules, quatre au moins. Une valeur porte les
mêmes échappements, et s'écrit entre guillemets si elle contient une espace — tout caractère blanc
d'Unicode resté tel quel après l'échappement, U+00A0, U+2007 et U+202F compris — ou un guillemet,
ses guillemets en `\"` ; une valeur vide s'écrit `""`, une valeur absente n'a pas sa clé. Chaque
message échappe de même les noms, les titres, les chemins et les caractères qu'il cite, chaque titre
entre chevrons. `--nom` et `--titre` reçoivent le texte brut, sans échappement.

**Les sorties de la tranche 1.**

- `fenetres` écrit une ligne par fenêtre, `[Fenetre] <titre> processus=<nom> pid=<pid>`, le nom
  sans `.exe` (`processus=""` s'il ne se lit pas), plus `premier-plan=oui` pour celle qui l'a. Il
  liste les candidates de tous les processus, sauf celles que la liste de refus refuse
  ([ADR-0005](0005-garde-fous-fenetre-retenue-et-premier-plan.md)), rangées par nom de processus
  sans égard à la casse (puis tel quel), puis par PID, puis par titre, octet par octet.
- `activer` écrit la ligne de la fenêtre retenue, qui porte `premier-plan=oui` si W elle-même a le
  premier plan.
- `touche` et `taper` n'écrivent rien sur la sortie standard.

### Les codes de sortie

| Code | Sens |
|---|---|
| 0 | fait |
| 1 | refus : fenêtre refusée, à la retenue comme après l'activation ; fenêtre pas au premier plan, ou que Windows refuse d'y mettre ; pendant la frappe, un envoi que Windows n'insère pas du tout quand le premier plan a changé ; poste verrouillé ; touche interdite ; fenêtre plus élevée, ou niveau d'intégrité illisible ; touche de modification enfoncée ; journal inaccessible ; focus hors de la fenêtre visée ; point de clic ou de molette hors de la fenêtre retenue ; les refus de KeePassXC pour `taper` et `touche` ; `valeur` sur un champ de mot de passe ; fenêtre réduite pour une capture ; exécutable refusé pour `lancer` ; dossier encore utilisé, ou dont le chemin n'est plus `%TEMP%\pl-<n>`, pour `nettoyer` ; un autre geste en cours après 5 s d'attente ; par la disposition du clavier, un caractère absent du clavier ou une touche morte ; avec `--comme-au-clavier`, Verr. Maj allumée |
| 2 | introuvable : fenêtre ou élément, ou plusieurs candidats sans `--titre`, `--pid` ou `--rang` ; la fenêtre retenue disparue après l'activation ; avec `--attendre`, rien de trouvé dans le délai |
| 3 | le geste a échoué : l'élément n'offre pas le geste demandé, une capture toute noire, un envoi de la frappe que Windows n'insère qu'en partie, ou pas du tout quand le premier plan est encore accepté, ou un envoi de l'activation qu'il n'insère qu'en partie, chaque insertion partielle après le relâchement tenté, une erreur du système que rien ne prévoit, un délai de l'automatisation dépassé, une ligne du journal perdue après un geste, une panique rattrapée ; le message, « échec du système : … », dit lequel |
| 64 | usage : une option inconnue, manquante ou mal formée, une valeur hors de ses bornes, une touche inconnue, un texte illisible, vide, trop long ou qui porte un caractère de contrôle, un fichier ou une entrée standard illisibles, une sous-commande pas encore livrée |

Une erreur d'usage que l'analyse des arguments rendrait en 2 sort en 64. Chaque refus dit sa raison,
et chaque essai d'un refus vérifie ce message, pas seulement le code.

### Les messages

Les messages que les tests vérifient, mot pour mot :

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
| 64 | `texte trop long : plus de 40006 octets, 10000 caractères au plus` |
| 64 | `caractère de contrôle : U+<XXXX>` |
| 64 | `texte vide` |
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

- Chaque message part sur la sortie d'erreur, en une ligne ; ses noms, ses titres, ses chemins et
  ses caractères y sont échappés, `<c>` étant le caractère échappé.
- Les titres de « plusieurs fenêtres » s'écrivent chacun entre chevrons, rangés comme les lignes de
  `fenetres`.
- `<arg>` est le nom que `clap` donne à l'argument (`--fenetre <FENETRE>`), tel quel ; le texte tapé
  (une option ou une sous-commande inconnues) s'y échappe.
- `<XXXX>` est le point de code du caractère de contrôle, en chiffres hexadécimaux majuscules,
  quatre au moins : `U+001B`, `U+0085`.
- Un échec du système nomme ce qui a échoué : par exemple « échec du système : SendInput a inséré
  <k> événements sur <n> ; relâchement : <m> sur <p> », après le relâchement tenté d'un envoi inséré
  en partie (ADR-0005, « Les lots »), ou sans sa seconde moitié quand le début inséré n'avait rien
  laissé enfoncé ; « échec du système : le journal n'a pas reçu sa ligne » ; « échec du système :
  panique ».

### L'analyse

- Toutes les sous-commandes du lot s'analysent, bornes comprises, sans lire aucune fenêtre :
  `clap`, puis, pour `touche`, les touches interdites, jugées sur tous les arguments (1), puis les
  inconnues (64) ; pour `taper`, le texte lu, décodé, admis et borné quelle que soit sa source (64).
  Une touche inconnue placée avant une interdite ne la cache pas : l'ordre des arguments ne change
  ni le code ni le message.
- L'aide (`--help`) et la version (`--version`, « pilote <version> (<commit>) », le commit en 12
  chiffres hexadécimaux, ou « inconnu ») s'écrivent sur la sortie standard, en 0. `pilote` seul sort
  en 64, « sous-commande manquante ».
- Une sous-commande que `clap` accepte et que la version ne livre pas encore est une commande
  valide : elle exige le journal, comme toute commande valide (1 sinon), puis sort en 64, « pas
  encore livrée : <sous-commande> », avec sa ligne au journal.

### Le journal, vu de la session

Le journal et sa ligne sont ceux d'ADR-0005. Ce qu'une session en voit : une commande valide qui ne
peut pas ouvrir le journal sort en 1, « journal inaccessible : ouvert dans une autre application ? »,
avant toute lecture de fenêtre et tout geste ; l'aide, la version et un refus de l'analyse (l'usage,
une touche interdite, un texte illisible, vide ou trop long) gardent leur code, que le journal s'ouvre ou non ; après
un geste, une ligne qui ne parvient pas au journal fait sortir en 3, « échec du système : le journal
n'a pas reçu sa ligne ».

### La fenêtre

- Chaque sous-commande qui touche une fenêtre la nomme par `--fenetre <processus>`, le nom du
  processus sans `.exe`, comparé sans égard à la casse ni au suffixe ; il refuse un caractère de
  contrôle et plus de 260 caractères (64). `--titre <texte>` départage les fenêtres d'un même
  processus par égalité exacte, et vise un dialogue précis ; `--pid <nombre>` départage deux
  processus de même nom, comme deux établis titrés « pupitre ».
- **Les candidates** sont les fenêtres de premier niveau, visibles, non masquées par DWM, ni
  fenêtres d'outil (`WS_EX_TOOLWINDOW`) ni sans titre, qui appartiennent au processus nommé par
  `--fenetre` ou à l'un de ses descendants. Une liste ouverte ou une bulle de WebView2, sans titre,
  n'en est donc pas. Une fenêtre réduite ou possédée en est une : `capture` refuse une fenêtre
  réduite (1), `activer` la restaure. Une fenêtre masquée n'en est pas (2).
- Les candidates que la liste de refus refuse s'écartent avant le compte, et ne sont jamais citées ;
  s'il n'en restait que de refusées, l'outil refuse (1). Il en faut alors exactement une, ou
  `--titre` ou `--pid` pour en garder une. La fenêtre retenue et le premier plan accepté sont ceux
  d'ADR-0005.

### Le sélecteur

- `--nom` compare en égalité exacte, après normalisation NFC, les espaces insécables comptées pour
  des espaces ; `--contient` compare en sous-chaîne ; `--type` filtre sur le type ; `--id` sur
  l'identifiant d'automatisation ; `--classe` sur la classe.
- `--dans <nom>` garde les éléments dont un ancêtre, de tout type hors la fenêtre et le document,
  porte ce nom, à toute profondeur : deux groupes emboîtés de même nom comptent tous deux. Le champ
  `dans=` nomme le plus proche ancêtre nommé de type `Group`, ou, à défaut, le plus proche ancêtre
  nommé hors la fenêtre et le document.
- Plusieurs éléments trouvés sans `--rang` font refuser, avec le code 2 et le nombre de candidats.
- `--rang` compte à partir de 1.
- `trouver` liste tous les candidats, avec leur rang.

Le choix d'un élément se fait dans le domaine pur, sur l'arbre que la couche Windows a lu
([ADR-0003](0003-domaine-pur-et-couche-windows.md)).

### L'attente

`--attendre <ms>`, sur `focus`, `trouver`, `valeur` et `arbre`, sonde toutes les 100 ms jusqu'à
trouver ce qu'il attend, au plus ce délai : une session ne peut pas dormir entre deux appels.

- `trouver` et `valeur` attendent l'élément de leur sélecteur, `valeur --egale <texte>` sa valeur.
- `focus` accepte les options du sélecteur, et attend que l'élément focalisé y réponde, ou, sans
  elles, que le focus soit dans W.
- `arbre` attend le groupe de `--dans`, ou un document non vide.
- Toute sous-commande qui lit l'arbre ou le focus sonde jusqu'à 2 s un document encore vide, même
  sans `--attendre`, par une seule fonction du domaine.

### Les gestes de l'automatisation

`invoquer` n'emploie que le motif Invoke. `choisir` bascule une fois (Toggle), sonde l'état (100 ms,
au plus 2 s), et ne bascule une seconde fois que si l'état a changé sans être coché, au plus deux
bascules, sinon 3 ; il choisit (SelectionItem), puis sonde de même ; il ne fait rien d'un élément
déjà coché ou choisi. `valeur` lit le motif Value et refuse un champ de mot de passe (1). Un élément
qui n'offre pas le motif fait sortir en 3. `arbre` montre tous les éléments de la vue de contrôle ;
`--max` compte des lignes, et un arbre coupé le dit sur la sortie d'erreur.

### Les bornes

Au-delà, l'usage refuse (64) : `--attendre` jusqu'à 30 000 ms ; `--fois` de 1 à 100 ; `--max` à
400 par défaut, de 1 à 10 000 ; un texte à taper de 1 à 10 000 caractères, quelle que soit sa
source, et un fichier ou une entrée standard de 40 006 octets au plus ; `--crans` de -50 à 50, sauf 0 ; `--pendant` de 1 à 300 s ; `--equivalent` de 100 à 400 ;
`--a` de 0 à 100. `--crans` admet un nombre négatif, et les options de texte (`--texte`, `--nom`,
`--contient`, `--titre`, `--id`, `--classe`, `--dans`, `--egale`) une valeur qui commence par un
tiret.

Les 40 006 octets sont ceux du plus long texte admis : la BOM de l'UTF-16LE (2 octets), 10 000
caractères hors du plan de base (4 octets chacun) et le CRLF final que le décodage retire
(4 octets) ; l'UTF-8 n'en demande jamais plus (3, puis 4 par caractère, puis 2). `pilote` lit un
fichier ou l'entrée standard 40 007 octets au plus, et refuse le texte dès qu'ils dépassent 40 006,
sans lire la suite (« texte trop long : plus de 40006 octets, 10000 caractères au plus ») : un
fichier énorme ne se charge jamais en mémoire.

### Les touches et les textes à taper

- **Les textes** passent par `--texte`, par `--fichier`, qui lit l'UTF-8 avec ou sans BOM et
  l'UTF-16LE à BOM, ou par l'entrée standard (`-`). Un fichier ou l'entrée standard perd un seul
  saut de ligne final, celui qu'ajoutent le shell ou l'éditeur : CRLF, LF ou CR seul ; `--texte`
  part tel quel, et son saut final donne un Entrée. Un texte se compte en caractères, des
  valeurs scalaires d'Unicode. CRLF, CR seul et LF seul donnent chacun un Entrée, la tabulation un
  Tab, et tout autre caractère part tel quel, sans normalisation : une lettre décomposée reste deux
  caractères. Sans la disposition du clavier, chaque caractère part en unités UTF-16, les deux
  moitiés d'une paire de substitution dans le même envoi.
- **Un caractère de contrôle refuse**, quelle que soit la source, avant tout geste et après le
  retrait du saut final, sauf la tabulation, CR et LF : envoyé en Unicode, il arriverait à la
  fenêtre comme une commande, non comme du texte (0x08 efface, 0x16 colle, 0x03 interrompt un
  terminal). Le premier du texte décide : NUL sort en « texte illisible : ni UTF-8 ni UTF-16 à
  BOM », puisqu'il est la trace d'un UTF-16 sans BOM lu comme de l'UTF-8 ; tout autre en « caractère
  de contrôle : U+<XXXX> ». Un texte vide, après ce retrait, refuse aussi (« texte vide ») : il
  n'aurait rien à taper, et l'activation ne servirait à rien.
- **Les touches ont des noms**, qu'aucun shell ne réécrit, sans égard à la casse : `Tab`, `Echap`,
  `Entree`, `Espace`, `Retour`, `Suppr`, `Inser`, `Haut`, `Bas`, `Gauche`, `Droite`, `Debut`, `Fin`,
  `PageHaut`, `PageBas`, `F1` à `F12`, une lettre avec Ctrl ; `Ctrl`, `Maj` et `Alt` les précèdent,
  dans tout ordre, joints par `+`. `barre` pour `/` et `interrogation` pour `?`, seuls, se tapent par
  la disposition du clavier, comme `taper --comme-au-clavier`, et déclenchent ainsi le raccourci que
  la page écoute. `--fois` frappe la suite de touches autant de fois.
- **La frappe par la disposition** prend la disposition du fil focalisé : celui de la fenêtre
  focalisée (`GetGUIThreadInfo`), à défaut de l'active, puis de la fenêtre au premier plan, puis
  `GetKeyboardLayout` de ce fil ; dans une WebView2, ce fil est celui de `msedgewebview2`. `\n` et
  `\t` donnent Entrée et Tab, sans modificateur ; tout autre caractère prend la touche et l'état que
  rend `VkKeyScanExW`. Les états acceptés forment une liste fermée : 0 (la touche seule), 1 (Maj
  gauche), 6 (AltGr : Ctrl gauche puis Alt droite), 7 (Maj gauche, Ctrl gauche, Alt droite) ; puis la
  touche, puis les relâchements dans l'ordre inverse. Tout autre état refuse (« caractère absent du
  clavier ») : -1, Ctrl sans Alt, Alt sans Ctrl, l'un des bits 8, 16, 32, 64 ou 128 ; un caractère
  hors du plan de base aussi. Puis `ToUnicodeEx`, l'état du clavier intact, doit produire le seul
  caractère demandé ; une touche morte refuse (« touche morte »). Rien ne part que la disposition ne
  produise exactement.
- **Chaque événement de touche** porte la touche virtuelle et son code de balayage
  (`MapVirtualKeyExW`, `MAPVK_VK_TO_VSC_EX`) ; il est étendu quand l'octet haut du balayage vaut 0xE0,
  ou quand la touche est l'une de PageHaut, PageBas, Fin, Debut, Gauche, Haut, Droite, Bas, Inser et
  Suppr (0x21 à 0x28, 0x2D, 0x2E), dont `MapVirtualKeyExW` rend le code du pavé numérique, sans 0xE0.

### Ce qui précise ou change le plan de la tranche

Précisé pendant l'exécution de la tranche, les 21 et 22/11/2026.

- Un retour chariot seul à la fin d'un fichier ou de l'entrée standard est retiré, comme CRLF et
  LF : les trois formes donnent un Entrée, et un éditeur peut finir un fichier par l'une ou l'autre.
- Un texte qui porte un caractère de contrôle autre que la tabulation, CR et LF refuse en 64, et un
  texte vide de même : ni l'un ni l'autre n'a de texte à taper.
- Un fichier ou l'entrée standard se lisent 40 007 octets au plus, et plus de 40 006 refusent en 64
  sans lire la suite : la borne en caractères se tient sans charger un fichier énorme.
- Un envoi inséré en partie relâche au mieux ce que son début a enfoncé, et le message le dit
  (« relâchement : <m> sur <p> »).
- Pendant la frappe, un envoi que Windows n'insère pas du tout sort en 1, « le premier plan a
  changé », si le premier plan relu n'est plus accepté, et en 3 sinon.
- Les états acceptés par `--comme-au-clavier` forment la liste fermée {0, 1, 6, 7}, les bits 64 et
  128 refusés : Windows ne définit pas ces bits, et rien ne part que la disposition ne produise
  exactement.
- L'ordre des fenêtres compare le nom sans égard à la casse, puis le PID, puis le titre octet par
  octet, et range de même les titres du message « plusieurs fenêtres » : une même liste rend
  toujours le même ordre, dans la sortie comme dans le message.
- Une fenêtre W disparue après l'activation sort en 2, « aucune fenêtre » : la fenêtre visée n'est
  plus là, comme si la retenue ne l'avait pas trouvée.
- `activer` rejuge la liste de refus, mais ni KeePassXC ni les touches enfoncées : `activer` vers
  KeePassXC est permis, et rien ne se frappe après l'activation.
- La ligne d'`activer` ne porte pas `premier-plan=oui` quand un dialogue de W tient le premier
  plan : ce champ dit quelle fenêtre a le premier plan, et c'est alors le dialogue, accepté pour W.
- Une sous-commande que `clap` accepte mais que la tranche ne livre pas exige le journal (1 sinon),
  puis sort en 64 avec sa ligne : c'est une commande valide, et toute commande valide exige le
  journal avant de faire quoi que ce soit.
- Limite connue : une écriture ratée sur la sortie standard ne change pas le code de sortie, et un
  `fenetres` peut sortir en 0 sans sa ligne : l'issue est jugée avant d'être écrite.

### Ce que la tranche 2 précise ou change

Précisé par le plan de la tranche 2 du lot 1, qu'Edouard a accepté le 28/11/2026, et pendant son
exécution.

- Un changement amende cet ADR en place, selon `docs/modele/documentation.md`, et non plus par un
  nouvel ADR (« Conséquences »).
- Une valeur s'écrit entre guillemets dès que son texte échappé porte un caractère blanc d'Unicode,
  U+00A0, U+2007 et U+202F compris, et non plus la seule espace ASCII : la ligne d'un élément y fait
  passer des textes que l'application pilotée choisit (`id`, `classe`, `valeur`, `raccourci`,
  `acces`, `dans`), et `id=a<U+00A0>motdepasse=oui`, sans guillemets, se lirait comme une clé
  forgée. La tabulation, le retour et les autres caractères que l'échappement écrit (`\t`, `\n`,
  `\u{XXXX}`) ne changent pas de forme (« Les échappements »).

## Conséquences

- La compétence décrit ce format, ces codes et ces messages ; un test du domaine garde chaque forme
  de ligne et chaque message.
- Une sous-commande pas encore livrée a déjà son usage, ses bornes et sa ligne au journal : la livrer
  ne change pas ce que la session écrit.
- **Difficile à changer** : tout ce que les sessions lisent, le format des lignes, le sens des codes
  et les messages. Un changement amende cet ADR en place (`docs/modele/documentation.md`), avec la
  compétence dans la même PR dès qu'elle existe.

## Confirmation

Les règles et les tests qui les tiennent, nommés par leur chemin :

- **Une valeur entre guillemets dès qu'elle porte un blanc ou un guillemet** (« Les
  échappements ») : `src/domaine/sortie.rs`, `valeur_entre_guillemets_si_blanc_d_unicode`,
  `valeur_entre_guillemets_si_espace_ou_guillemet`, `valeur_vide_s_ecrit_guillemets` et
  `valeur_echappe_ses_caracteres` : l'espace, U+00A0, U+2007, U+202F, U+3000 et le guillemet mettent
  la valeur entre guillemets, ces blancs gardés tels quels ; une tabulation, écrite `\t`, et un
  retour, écrit `\n`, n'en mettent pas ; vide, elle s'écrit `""`.

## Révision

Si un usage par un programme, et non par une session, demande une sortie JSON (une piste).
