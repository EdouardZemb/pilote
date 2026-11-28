# pilote

Piloter une application de bureau de Windows depuis une session Claude : le clavier, l'automatisation de Windows et la capture de la seule fenêtre visée

<!-- Pour une application : une capture d'écran, à fond opaque. -->

## Pourquoi

Une session de Claude Code en ligne de commande n'a aucun outil pour piloter une application de
bureau. `pilote` lui en donne un, sous Windows : il tape au clavier du poste, lit ce que lit un
lecteur d'écran (le focus, les noms, les valeurs, les raccourcis, les zones annoncées), agit par
l'automatisation de Windows et capture la seule fenêtre visée. Il sert d'abord aux essais installés
et aux contrôles d'accessibilité des projets d'Edouard. Il ne lit ni ne tape jamais un secret, ne
capture jamais l'écran entier, et ne remplace pas Claude in Chrome pour une page web.

L'état du projet est dans [CLAUDE.md](CLAUDE.md#état).

## Démarrer

<!-- La commande qui lance le projet. -->

Les autres commandes sont dans [CLAUDE.md](CLAUDE.md#commandes).

## Où lire quoi

| Document | À lire quand |
|---|---|
| [CLAUDE.md](CLAUDE.md) | avant de travailler ici |
| [La méthode](docs/modele/methode.md) | pour la façon de travailler, commune aux projets nés de `modele-projet` |
| [Les pistes](docs/pistes.md) | pour ce qui est noté sans être engagé |
| [Le registre](docs/registre.md) | pour ce que les lots ont appris |
| [Les décisions](docs/decisions/) | pour ce qui a été décidé, et pourquoi |

## Contribuer

Projet personnel : aucune contribution n'est attendue.

## Licence

[MIT](LICENSE)
