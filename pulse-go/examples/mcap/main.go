package main

import (
	"context"
	"fmt"
	"time"

	"github.com/machanirobotics/pulse"
	"github.com/machanirobotics/pulse/options"
)

// ChatMessage represents a message in an LLM chat room (same as simple example)
type ChatMessage struct {
	MessageID string `json:"message_id" pulse:"attribute:message.id"`
	RoomID    string `json:"room_id" pulse:"attribute:room.id"`
	UserID    string `json:"user_id" pulse:"attribute:user.id"`
	Language  string `json:"language" pulse:"attribute:message.language"`
	Type      string `json:"type" pulse:"attribute:message.type"` // text, speech, llm_response

	// Additional data
	Content   string `json:"content"`
	Timestamp int64  `json:"timestamp"`
}

func main() {
	ctx := context.Background()

	// Configure service options - SAME NAME as simple example
	serviceOpts := options.ServiceOptions{
		Name:        "llm-chat-example",
		Description: "LLM chat with MCAP and OTLP logging",
		Version:     "1.0.0",
		Environment: options.Development,
	}

	// Configure Pulse with OTLP and MCAP
	pulseOpts := options.PulseOptions{
		Telemetry: options.DefaultTelemetry(),
	}
	pulseOpts.Telemetry.OTLP.Enabled = true
	pulseOpts.Telemetry.OTLP.Host = "localhost"
	pulseOpts.Telemetry.OTLP.Port = 4317

	// Enable Foxglove MCAP logging - save in examples/mcap directory
	pulseOpts.Foxglove = options.FoxgloveOptions{
		Enabled:  true,
		McapPath: "examples/mcap/llm-chat-example.mcap",
	}

	// Initialize Pulse framework
	p, err := pulse.New(ctx, serviceOpts, pulseOpts)
	if err != nil {
		panic(err)
	}
	defer func() {
		if err := p.Close(ctx); err != nil {
			fmt.Printf("Error closing pulse: %v\n", err)
		}
	}()

	// Log messages - these will be written to:
	// 1. Console (stdout)
	// 2. OpenTelemetry → Loki (visible in Grafana dashboard)
	// 3. MCAP file at examples/mcap/llm-chat-example.mcap (open in Foxglove Studio)

	p.Logger.Info("MCAP + OTLP Chat Example Started")
	p.Logger.Info("Logs will appear in Grafana dashboard AND MCAP file")

	// Generate chat messages with attributes - cycling through all log levels
	users := []string{"user-alice", "user-carlos", "user-yuki", "user-marie"}
	languages := []string{"en", "es", "ja", "fr"}
	messageTypes := []string{"text", "speech", "llm_response"}

	for i := 0; i < 20; i++ {
		userIdx := i % len(users)

		msg := ChatMessage{
			MessageID: fmt.Sprintf("msg-mcap-%03d", i),
			RoomID:    "room-ai-chat",
			UserID:    users[userIdx],
			Language:  languages[userIdx],
			Type:      messageTypes[i%len(messageTypes)],
			Content:   fmt.Sprintf("Message %d from %s", i, users[userIdx]),
			Timestamp: time.Now().Unix(),
		}

		// Cycle through all log levels
		switch i % 5 {
		case 0:
			p.Logger.Debug("DEBUG: Chat message", msg)
		case 1:
			p.Logger.Info("INFO: Chat message", msg)
		case 2:
			p.Logger.Warn("WARN: Chat message", msg)
		case 3:
			if err := p.Logger.Error("ERROR: Chat message", msg); err != nil {
				return
			}
		case 4:
			p.Logger.Info("INFO: Chat message", msg)
		}

		// Sleep to simulate real-time chat
		time.Sleep(300 * time.Millisecond)
	}

	p.Logger.Info("MCAP example completed!")
	p.Logger.Info("Check:")
	p.Logger.Info("  1. Grafana dashboard: http://localhost:3000")
	p.Logger.Info("  2. MCAP file: examples/mcap/llm-chat-example.mcap")
	p.Logger.Info("  3. Open MCAP in Foxglove Studio to visualize")

	// The MCAP file will be properly closed when p.Close(ctx) is called
}
