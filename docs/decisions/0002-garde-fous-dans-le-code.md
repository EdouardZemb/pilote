# ADR-0002 — Les garde-fous : dans le code, sans aucun fichier de réglage

- **Statut** : accepté le 2026-11-21
- **Contexte amont** : le design du 2026-11-21, décisions 5 et 9 à 12, et « Les choix de la session de pupitre » ([le brief](../00-brief-initial.md)) ; [le cadrage](../cadrage.md)

## Contexte

Un outil qui tape et clique pour une session peut taper dans la mauvaise fenêtre, montrer une autre
fenêtre dans une capture, laisser passer une valeur secrète, ou servir à contourner les règles de
sécurité des sessions : ce sont les anti-buts du design. Edouard a réduit les refus de l'outil au
second tour, et laissé le reste à la compétence.

## Options

1. **Les refus dans un fichier de réglage.** Souple, mais une session qui lit ou écrit ce fichier
   pourrait élargir ou réduire ce que l'outil refuse.
2. **Une liste blanche des applications permises.** Plus fermée, mais il faudrait toucher l'outil à
   chaque nouvelle application d'essai, et l'usage est une règle de la compétence, pas une serrure.
3. **Une liste de refus courte, et quelques serrures, en dur dans le code.**

## Décision

L'option 3. Rien ne configure l'outil : aucun fichier qu'une session lirait ne peut élargir ni
réduire ce qu'il refuse.

- **La liste de refus** : les invites sécurisées de Windows (`consent`, `LogonUI`,
  `CredentialUIBroker`) et les Paramètres de Windows (`SystemSettings`), cherchés aussi dans les
  fenêtres enfants, puisque les Paramètres tournent dans une fenêtre d'`ApplicationFrameHost`. Un
  processus refusé ne paraît pas dans `fenetres`.
- **Les valeurs secrètes ne sortent jamais** : un élément que l'automatisation marque comme mot de
  passe (`IsPassword`) ne rend sa valeur ni par `valeur`, ni par `arbre`, ni par `trouver`, quelle
  que soit l'application.
- **KeePassXC** : `capture` refuse ses fenêtres ; `taper` refuse quand le focus est dans un de ses
  champs de mot de passe.
- **La fenêtre visée** est l'arbre de processus du processus nommé. Le premier plan se vérifie avant
  chaque touche et chaque caractère ; un clic ou un cran de molette doit tomber sur la fenêtre
  visée. Lire n'active rien.
- **Le poste verrouillé et l'application élevée** font refuser, au lieu d'échouer en silence.
- **Les touches interdites** : la touche Windows et ses combinaisons, Ctrl+Alt+Suppr, Alt+Tab, et
  toute touche qui change de fenêtre ; Alt+F4 reste permis.
- **Aucun `SetValue`** : un champ se remplit en tapant.
- **Les captures** ne prennent que la fenêtre visée, par `PrintWindow`, jamais une copie de l'écran,
  et l'outil n'en garde aucune copie.
- **Le journal** garde, pour chaque appel, la date, la session, la sous-commande, le processus
  visé, le code de sortie et la longueur du texte tapé, jamais le texte.
- **Un seul appel à la fois** sur le poste : deux sessions ne tapent jamais en même temps.

Les règles que l'outil ne peut pas tenir seul (l'usage, les secrets, le terminal, le navigateur, ce
qui reste à Edouard) sont dans la compétence. Reconnaître à coup sûr un terminal qui héberge une
session Claude n'est pas possible : une serrure fragile tromperait.

## Conséquences

- Changer un refus demande une PR, relue, avec ses tests.
- Une application élevée ne se pilote pas, même à la demande d'Edouard : la session non élevée est
  un choix du poste.
- **Difficile à changer** : rien de technique ; c'est une frontière de sécurité, qui ne s'élargit
  qu'avec l'accord d'Edouard.

## Révision

Au premier contournement constaté, ou si Edouard élargit ou réduit l'usage.
