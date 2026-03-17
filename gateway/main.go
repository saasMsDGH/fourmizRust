package main

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"time"

	pb "gateway/proto"

	"github.com/gorilla/websocket"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true }, // Allow all origins for the simulation
}

type Gateway struct {
	grpcClient pb.SimulationServiceClient
}

func main() {
	// Wait for the Rust engine to start up
	time.Sleep(2 * time.Second)

	// Connect to Rust gRPC Engine
	// In production (k8s), this should be the engine service DNS name e.g. "engine:50051"
	conn, err := grpc.Dial("localhost:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("did not connect: %v", err)
	}
	defer conn.Close()

	client := pb.NewSimulationServiceClient(conn)
	gw := &Gateway{grpcClient: client}

	http.HandleFunc("/ws", gw.handleWebSocket)
	http.HandleFunc("/api/command", gw.handleCommand)

	log.Println("Gateway listening on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}

func (gw *Gateway) handleWebSocket(w http.ResponseWriter, r *http.Request) {
	ws, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("Upgrade error: %v", err)
		return
	}
	defer ws.Close()

	// Request 60 updates per second stream from Rust
	req := &pb.StreamRequest{
		TargetTps: 60,
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	stream, err := gw.grpcClient.StreamState(ctx, req)
	if err != nil {
		log.Printf("Stream request error: %v", err)
		return
	}

	for {
		state, err := stream.Recv()
		if err != nil {
			log.Printf("Stream receive error: %v", err)
			break
		}

		// Convert state to JSON and send over WebSocket
		if err := ws.WriteJSON(state); err != nil {
			log.Printf("WebSocket write error: %v", err)
			break
		}
	}
}

type CommandPayload struct {
	Command int32   `json:"command"`
	X       *float32 `json:"x,omitempty"`
	Y       *float32 `json:"y,omitempty"`
	Amount  *int32   `json:"amount,omitempty"`
}

func (gw *Gateway) handleCommand(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS")
	w.Header().Set("Access-Control-Allow-Headers", "Content-Type")

	if r.Method == http.MethodOptions {
		w.WriteHeader(http.StatusOK)
		return
	}

	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var payload CommandPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		http.Error(w, "Bad request", http.StatusBadRequest)
		return
	}

	log.Printf("Reçu commande du client API [%d] - x:%v y:%v amount:%v", payload.Command, payload.X, payload.Y, payload.Amount)

	req := &pb.CommandRequest{
		Command: pb.CommandType(payload.Command),
		X:       payload.X,
		Y:       payload.Y,
		Amount:  payload.Amount,
	}

	res, err := gw.grpcClient.SendCommand(context.Background(), req)
	if err != nil {
		http.Error(w, "Engine error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(res)
}
