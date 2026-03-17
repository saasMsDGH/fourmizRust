// Ces imports ramènent les mêmes objets utilisés dans main.rs pour la synchronisation et la concurrence.
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
// Un "wrapper" pour transformer un canal receveur `rx` en Stream compatible gRPC qui envoie des données en continu.
use tokio_stream::wrappers::ReceiverStream;
// Structures de base de la bibliothèque `tonic` pour traiter les requêtes HTTP/2 et leurs statuts.
use tonic::{Request, Response, Status};

// Les composants générés par protobuf, représentant nos messages et fonctions réseaux
use crate::pb::simulation_service_server::SimulationService;
use crate::pb::{CommandRequest, CommandResponse, GameState, StreamRequest};
use crate::simulation::SimulationEngine;

// Le composant SimService va définir les données requises pour répondre aux requêtes gRPC.
// Il possède l'accès au moteur de simulation (engine) et l'émetteur du canal de commande (tx).
pub struct SimService {
    pub engine: Arc<RwLock<SimulationEngine>>,
    pub tx: mpsc::Sender<CommandRequest>,
}

// `#` et `[]` en Rust s'appellent des attributs. Cette macro permet d'utiliser des requêtes `async` (non bloquantes) dans ce Trait.
#[tonic::async_trait]
// Le block implémentant le `Trait` SimulationService (interface générée par protobuf).
impl SimulationService for SimService {
    // Définit le type du flux HTTP/2 de sortie.
    type StreamStateStream = ReceiverStream<Result<GameState, Status>>;

    // Implémentation de la fonction envoyant une "Commande" en provenance du client (ex: Frontend/Gateway).
    async fn send_command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        // Extraction de l'objet de la requête
        let cmd = request.into_inner();

        // Forward command to the simulation engine via channel
        // On essaye d'envoyer la commande dans le canal.
        if let Err(e) = self.tx.send(cmd).await {
            // Si le canal est fermé ou plein, on affiche une erreur et on retourne le statut "Erreur Interne"
            log::error!("Failed to send command to engine: {}", e);
            return Err(Status::internal("Engine is not responding"));
        }

        // Si tout se passe bien, on génère une Réponse avec le message de succès.
        Ok(Response::new(CommandResponse {
            success: true,
            message: "Command dispatched".to_string(),
        }))
    }

    // Implémentation du système Server-Streaming (`stream_state`).
    // Ouvre une connexion continue où le serveur gRPC envoie des données sans arrêt.
    async fn stream_state(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamStateStream>, Status> {
        let req = request.into_inner();
        
        // On récupère le nombre de données voulues par seconde (Target TPS) transmises par le client.
        // Si aucune ou "0" n'est envoyée, on limite par défaut l'envoi vers le client à 30 TPS 
        // pour économiser la bande passante avec le Gateway.
        let tps = if req.target_tps > 0 {
            req.target_tps as u64
        } else {
            30
        };

        // Création d'un canal de communication asynchrone pour que le thread parallèle envoie au streamer
        let (tx, rx) = mpsc::channel(128);
        
        // On prend un clône de la référence au moteur principal.
        let engine = Arc::clone(&self.engine);

        // Tâche asynchrone (spawn) qui va s'occuper d'envoyer l'état du jeu régulièrement au client.
        tokio::spawn(async move {
            // Définit le délai entre chaque envoi en fonction des TPS cible (ex: 30 TPS => 33.3ms)
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000 / tps));
            loop {
                // Fait une pause jusqu'au prochain tick défini par l'intervalle.
                interval.tick().await;
                
                // Un bloc `{}` est utilisé pour limiter la portée de fonctionnement du "Read Lock" (verrou en lecture)
                let state = {
                    let e = engine.read().await; // Bloque et emprunte l'accès complet au lecteur
                    e.get_game_state() // Appelle la création du GameState (les données actuelles)
                }; // Ici le `lock` est automatiquement libéré pour ne pas bloquer les autres
                
                // Transmet le GameState au Stream principal grâce au canal
                if tx.send(Ok(state)).await.is_err() {
                    // Si `tx` ne peut pas adresser `rx` (ex: client déconnecté), on sort de la boucle.
                    break;
                }
            }
        });

        // On retourne la structure ReceiverStream, encapsulant la partie réception du canal asynchrone.
        // Tonic enverra automatiquement par le réseau tout ce qui passe dans `rx`.
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
