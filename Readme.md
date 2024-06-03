# TODO

- Qemu VM?
- Docker container?


# Test technique RUST

# Objectif
    Créer un backend en Rust qui permet d'exécuter du code Python et d'en faire une API.

# Détails du projet
    - Un "configurateur" envoie du code Python
        Le backend doit permettre à un utilisateur (configurateur) d'envoyer du code Python.
        Ce code Python doit être capable d'utiliser la bibliothèque pandas.
        Sur réception du code Python, le backend doit créer une API basée sur ce code.
    - Un "utilisateur" exécute cette API
        Le backend doit permettre à un autre utilisateur d'exécuter cette API.

# Figures imposées
    - Utilisation de Axum pour le framework web.
    - Le code Python doit pouvoir utiliser la bibliothèque pandas.
    - Suivre les principes du clean code / YAGNI : implémenter uniquement ce qui est nécessaire pour que cela fonctionne.

# Extra Points
    - Développer un front-end en Vue3 (sinon, des appels cURL ou Python suffisent).
    - Mise en place d'une sandbox pour exécuter le code Python en toute sécurité. Justifiez la stratégie de sandboxing choisie.

# Livrables
    - Le code source du projet, organisé de manière claire.
    - Instructions pour exécuter le projet localement.
    - Un court document expliquant les choix techniques et les éventuels compromis faits lors du développement.

# Critères d'évaluation
    - Respect des consignes et des figures imposées.
    - Qualité du code (propreté, lisibilité, structuration).
    - Fonctionnalité et robustesse de l'application.
    - Originalité et pertinence des solutions proposées pour le sandboxing (le cas échéant).
    - Documentation et instructions fournies.
