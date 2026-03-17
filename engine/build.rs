// Ce script "build.rs" est exécuté automatiquement par Cargo avant la compilation du code principal.
// Il sert généralement à générer du code Rust à partir de ressources externes (ici un fichier .proto).
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `tonic_build` est une bibliothèque pour générer le code gRPC à partir des schémas Protobuf.
    tonic_build::configure()
        // Indique que nous voulons générer le code pour faire tourner un serveur gRPC.
        .build_server(true)
        // Indique que nous ne voulons pas générer le code client gRPC (notre serveur Rust n'appelle pas de client ici).
        .build_client(false) // We only need the server in Rust
        // Lance la compilation en fournissant le fichier `.proto` d'entrée et les répertoires d'inclusion (`"../proto"`).
        // Le `?` à la fin permet de remonter automatiquement les erreurs si la génération échoue.
        .compile_protos(&["../proto/antsimulator.proto"], &["../proto"])?;
    
    // Si on arrive ici, l'exécution s'est déroulée sans erreur. 
    // On retourne `Ok(())` (l'équivalent de "success" pour un type `Result`).
    Ok(())
}
