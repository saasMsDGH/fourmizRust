// Ce module `pb` encapsule le code Rust auto-généré par `tonic_build` (via build.rs).
pub mod pb {
    // `include_proto!` est une macro qui importe le code généré spécifique au package "antsimulator".
    tonic::include_proto!("antsimulator");
}

// Déclaration des modules de notre projet : `grpc` pour le réseau, `simulation` pour le moteur de jeu.
mod grpc;
mod simulation;

// Importation de la structure du serveur (auto-générée).
use pb::simulation_service_server::SimulationServiceServer;
// `Arc` (Atomic Reference Counted) et `RwLock` (Read-Write Lock) permettent un accès sécurisé et concurrentiel aux données.
use std::sync::Arc;
// `mpsc` (Multi-Producer, Single-Consumer) est un canal pour envoyer des messages entre threads asynchrones.
use tokio::sync::{mpsc, RwLock};
// L'outil serveur de `tonic` pour configurer et lancer le serveur gRPC.
use tonic::transport::Server;

// Importations des composants internes de nos modules
use grpc::SimService;
use simulation::SimulationEngine;

// La macro `tokio::main` transforme la fonction main asynchrone en une exécution gérée par le runtime asynchrone Tokio.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise le système de logs (par exemple `RUST_LOG=info`).
    env_logger::init();
    // Définit l'adresse et le port d'écoute du serveur. `?` fera crasher si l'adresse est invalide.
    let addr = "0.0.0.0:50051".parse()?;

    // Création du moteur de simulation. 
    // `RwLock` permet de bloquer la lecture/écriture en garantissant la thread-safety.
    // `Arc` permet de partager la propriété de ce moteur à travers plusieurs threads (runtime tokio).
    let engine = Arc::new(RwLock::new(SimulationEngine::new()));
    
    // Création d'un canal (channel) asynchrone d'une capacité de 100 messages en attente.
    // `tx` (transmetteur) et `rx` (receveur).
    let (tx, mut rx) = mpsc::channel(100);

    // Initialisation de notre service gRPC personnalisé contenant l'accès au moteur et l'émetteur de commandes.
    let sim_service = SimService {
        engine: Arc::clone(&engine), // On clone la *référence* (pas le moteur lui-même)
        tx,
    };

    // On prépare une instance supplémentaire de la référence pour le thread (tâche=spawn) de simulation
    let engine_clone = Arc::clone(&engine);
    // On lance une tâche asynchrone en arrière-plan (qui s'exécutera à l'infini).
    tokio::spawn(async move {
        // Main simulation loop 100 TPS (Ticks Per Second)
        // Intervalle réglé sur 10 millisecondes (soit 1/100 seconde)
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));
        loop {
            // Fait une pause jusqu'au prochain tick (garantissant les 100 TPS de manière fluide)
            interval.tick().await;

            // Process commands
            // Récupère toutes les commandes envoyées par les clients gRPC dans le canal sans bloquer (`try_recv`).
            while let Ok(cmd) = rx.try_recv() {
                // `write().await` obtient l'accès exclusif en écriture au moteur de jeu.
                let mut e = engine_clone.write().await;
                e.process_command(cmd);
            }

            // Tick simulation
            // Toujours verrouiller en écriture avant de déclencher la mise à jour (tick)
            let mut e = engine_clone.write().await;
            e.tick(); // Fait avancer le moteur d'un "tour"
        }
    });

    // Un log pour informer que le serveur démarre
    log::info!("Simulation Engine gRPC server listening on {}", addr);

    // Initialisation et démarrage final du serveur gRPC avec Tonic.
    Server::builder()
        // Enregistre notre service auprès du serveur
        .add_service(SimulationServiceServer::new(sim_service))
        // Démarrage de l'écoute. La fonction `serve` est bloquante et maintiendra le programme en vie.
        .serve(addr)
        .await?; // Attente indéfinie tant que le serveur tourne

    Ok(())
}
