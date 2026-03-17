.PHONY: all dev frontend engine gateway install proto clean

# Default target runs everything
all: dev

# Run all services concurrently
dev:
	@echo "Lancement de tous les services (Frontend, Engine, Gateway)..."
	@make -j3 frontend engine gateway

# Install dependencies
install:
	@echo "Installation des dépendances Frontend..."
	cd frontend && npm install
	@echo "Installation des dépendances Gateway..."
	cd gateway && go mod tidy
	@echo "Dépendances installées."

# Start the React Vite frontend
frontend:
	@echo "Démarrage du Frontend..."
	cd frontend && npm run dev

# Start the Rust Game Engine
engine:
	@echo "Démarrage du Game Engine..."
	cd engine && cargo run

# Start the Go Gateway
gateway:
	@echo "Démarrage de la Gateway..."
	cd gateway && go run main.go

# Generate Protobufs (si nécessaire manuellement pour la gateway)
proto-go:
	@echo "Génération des fichiers protobuf Go..."
	cd gateway && protoc --go_out=. --go-grpc_out=. ../proto/antsimulator.proto

# Build du moteur Rust (build.rs gère normalement les protos automatiquement)
proto-rust:
	cd engine && cargo build
