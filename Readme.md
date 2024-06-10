# Instructions

If you're not on a Linux OS, it's probably better to run this in a Linux VM, as it hasn't been tested on other OSes.

Full setup commands on a Debian-based host
```
sudo apt update && sudo apt install bubblewrap python3 jq curl build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/izissise/freeitw2024.git
cd freeitw2024
cargo run
```

The API will listen on port 3000. There are example API requests with curl/jq in the client directory.
```
$ ./client/hello.sh
[+] Install lambda
What's your name: Hugues
Hello Hugues
Exit status exit status: 0
```

# Technical choices

In src/lambda_app.rs and src/sandbox.rs, you can find the definitions and implementations of the Lambda and Sandbox structs.

Both use enum_dispatch to make it easy to add new implementations. It also simplifies function calls since enum_dispatch automatically adds the trait implementation on the enum and dispatches to the correct one.

`LambdaAppKind` and `SandboxKind` are stored in the 'global' API state:
```
struct ApiState {
    lambdas: HashMap<String, Arc<LambdaApp>>,
    sandboxs: HashMap<String, Arc<Sandbox>>,
}
```
The user has to give them a name, hence the use of `String` for the key. We wrap the enums in Arc so when we later retrieve them, we can simply copy the `Arc` and not lock the entire state during an HTTP request's lifetime.

The `ApiState` struct is initialized using `Arc<RwLock<ApiState>>`. RwLock allows for multiple concurrent readers, reducing parallel GET requests bottleneck.

We need the `Arc` to make the struct `Send`, allowing `tokio` to use multiple executor threads to respond to requests.

lambda_exec receives an already spawned Child process from the Lambda's trait. Most of the code there is to allow streaming the HTTP request and response directly from/to the child's standard IO.

I chose `bubblewrap` as the sandbox, akin to Docker it uses cgroups to isolate processes from the host. It is the jail engine behind flatpak.

There is also a `Host` sandbox implementation, which is used to set up a Python virtual environment and install pandas at startup.


I used clippy pedantic to make the code more robust.

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
