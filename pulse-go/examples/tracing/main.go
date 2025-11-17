package main

import (
	"context"
	"errors"
	"fmt"
	"os"
	"time"

	"github.com/machanirobotics/pulse"
	"github.com/machanirobotics/pulse/options"
)

// Malenia Conversation Pipeline - AI Assistant Tracing Example
// This example demonstrates distributed tracing for a conversational AI assistant named Malenia.
// The pipeline includes 7 main components, each traced as a separate span with detailed events:
// 1. Input Processing - Validate and normalize user input
// 2. Context Retrieval - Fetch conversation history and user context
// 3. Intent Classification - Determine user intent and extract entities
// 4. Knowledge Search - Search knowledge base for relevant information
// 5. Response Generation - Generate AI response using LLM
// 6. Response Validation - Validate and filter response
// 7. Output Formatting - Format response for delivery

// ConversationRequest represents the input for the conversation pipeline
type ConversationRequest struct {
	RequestID     string `pulse:"trace:request.id"`
	UserID        string `pulse:"trace:user.id"`
	SessionID     string `pulse:"trace:session.id"`
	UserMessage   string `pulse:"trace:input.message"`
	MessageLength int    `pulse:"trace:input.length"`
	Timestamp     string `pulse:"trace:request.timestamp"`
}

// InputProcessingRequest represents input validation and normalization
type InputProcessingRequest struct {
	RequestID   string `pulse:"trace:request.id"`
	RawInput    string `pulse:"trace:input.raw"`
	InputLength int    `pulse:"trace:input.length"`
	Language    string `pulse:"trace:input.language"`
}

// InputProcessingResponse represents processed input
type InputProcessingResponse struct {
	RequestID        string  `pulse:"trace:request.id"`
	ProcessedInput   string  `pulse:"trace:input.processed"`
	IsValid          bool    `pulse:"trace:input.valid"`
	TokenCount       int     `pulse:"trace:input.tokens"`
	DetectedLanguage string  `pulse:"trace:input.detected_language"`
	ProcessingTimeMs float64 `pulse:"trace:processing_time_ms"`
}

// ContextRetrievalRequest represents fetching conversation context
type ContextRetrievalRequest struct {
	RequestID string `pulse:"trace:request.id"`
	UserID    string `pulse:"trace:user.id"`
	SessionID string `pulse:"trace:session.id"`
	MaxTurns  int    `pulse:"trace:context.max_turns"`
}

// ContextRetrievalResponse represents retrieved context
type ContextRetrievalResponse struct {
	RequestID        string   `pulse:"trace:request.id"`
	HistoryTurns     int      `pulse:"trace:context.history_turns"`
	UserPreferences  []string `pulse:"trace:context.preferences"`
	ContextTokens    int      `pulse:"trace:context.tokens"`
	CacheHit         bool     `pulse:"trace:context.cache_hit"`
	ProcessingTimeMs float64  `pulse:"trace:processing_time_ms"`
}

// IntentClassificationRequest represents intent detection
type IntentClassificationRequest struct {
	RequestID string `pulse:"trace:request.id"`
	Message   string `pulse:"trace:intent.message"`
	Context   string `pulse:"trace:intent.context"`
}

// IntentClassificationResponse represents detected intent
type IntentClassificationResponse struct {
	RequestID        string   `pulse:"trace:request.id"`
	Intent           string   `pulse:"trace:intent.name"`
	Confidence       float64  `pulse:"trace:intent.confidence"`
	Entities         []string `pulse:"trace:intent.entities"`
	EntityCount      int      `pulse:"trace:intent.entity_count"`
	ProcessingTimeMs float64  `pulse:"trace:processing_time_ms"`
}

// KnowledgeSearchRequest represents knowledge base search
type KnowledgeSearchRequest struct {
	RequestID string  `pulse:"trace:request.id"`
	Query     string  `pulse:"trace:search.query"`
	Intent    string  `pulse:"trace:search.intent"`
	TopK      int     `pulse:"trace:search.top_k"`
	Threshold float64 `pulse:"trace:search.threshold"`
}

// KnowledgeSearchResponse represents search results
type KnowledgeSearchResponse struct {
	RequestID        string   `pulse:"trace:request.id"`
	ResultCount      int      `pulse:"trace:search.result_count"`
	DocumentIDs      []string `pulse:"trace:search.document_ids"`
	AvgRelevance     float64  `pulse:"trace:search.avg_relevance"`
	CacheHit         bool     `pulse:"trace:search.cache_hit"`
	ProcessingTimeMs float64  `pulse:"trace:processing_time_ms"`
}

// ResponseGenerationRequest represents LLM response generation
type ResponseGenerationRequest struct {
	RequestID        string  `pulse:"trace:request.id"`
	UserMessage      string  `pulse:"trace:llm.user_message"`
	SystemPrompt     string  `pulse:"trace:llm.system_prompt"`
	Context          string  `pulse:"trace:llm.context"`
	KnowledgeContext string  `pulse:"trace:llm.knowledge"`
	ModelName        string  `pulse:"trace:llm.model"`
	Temperature      float64 `pulse:"trace:llm.temperature"`
	MaxTokens        int     `pulse:"trace:llm.max_tokens"`
}

// ResponseGenerationResponse represents generated response
type ResponseGenerationResponse struct {
	RequestID        string  `pulse:"trace:request.id"`
	Response         string  `pulse:"trace:llm.response"`
	TokensPrompt     int     `pulse:"trace:llm.tokens_prompt"`
	TokensCompletion int     `pulse:"trace:llm.tokens_completion"`
	TokensTotal      int     `pulse:"trace:llm.tokens_total"`
	FinishReason     string  `pulse:"trace:llm.finish_reason"`
	ProcessingTimeMs float64 `pulse:"trace:processing_time_ms"`
}

// ResponseValidationRequest represents response validation
type ResponseValidationRequest struct {
	RequestID      string `pulse:"trace:request.id"`
	Response       string `pulse:"trace:validation.response"`
	ResponseLength int    `pulse:"trace:validation.length"`
}

// ResponseValidationResponse represents validation results
type ResponseValidationResponse struct {
	RequestID        string  `pulse:"trace:request.id"`
	IsValid          bool    `pulse:"trace:validation.is_valid"`
	IsSafe           bool    `pulse:"trace:validation.is_safe"`
	HasPII           bool    `pulse:"trace:validation.has_pii"`
	ToxicityScore    float64 `pulse:"trace:validation.toxicity_score"`
	ProcessingTimeMs float64 `pulse:"trace:processing_time_ms"`
}

// OutputFormattingRequest represents output formatting
type OutputFormattingRequest struct {
	RequestID string `pulse:"trace:request.id"`
	Response  string `pulse:"trace:output.raw_response"`
	Format    string `pulse:"trace:output.format"`
}

// OutputFormattingResponse represents formatted output
type OutputFormattingResponse struct {
	RequestID        string  `pulse:"trace:request.id"`
	FormattedOutput  string  `pulse:"trace:output.formatted"`
	OutputLength     int     `pulse:"trace:output.length"`
	Markdown         bool    `pulse:"trace:output.markdown"`
	ProcessingTimeMs float64 `pulse:"trace:processing_time_ms"`
}

func main() {
	// Run the Malenia conversation pipeline example
	runMaleniaConversationPipeline()

	// Run error handling example
	runMaleniaWithError()
}

func runMaleniaConversationPipeline() {
	ctx := context.Background()

	// Get OTLP host from environment or use default
	otlpHost := "localhost"
	if envHost := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"); envHost != "" {
		otlpHost = envHost
	}

	// Initialize Pulse with tracing enabled
	k, err := pulse.New(ctx, options.ServiceOptions{
		Name:        "malenia-conversation-service",
		Description: "Malenia AI Assistant with distributed tracing",
		Version:     "1.0.0",
		Environment: options.Production,
	}, options.PulseOptions{
		Telemetry: options.TelemetryOptions{
			Logging: options.LoggingTelemetryOptions{
				Enabled: true,
			},
			Metrics: options.MetricsTelemetryOptions{
				Enabled:               true,
				ExportIntervalSeconds: 10,
			},
			Tracing: options.TracingTelemetryOptions{
				Enabled: true,
			},
			OTLP: options.OTLPOptions{
				Host:    otlpHost,
				Port:    4317,
				Enabled: true,
			},
		},
		Tracing: options.TracingOptions{
			Enabled: true,
		},
	})
	if err != nil {
		panic(err)
	}
	defer func() {
		_ = k.Close(ctx)
	}()

	k.Logger.Info("=== Malenia Conversation Pipeline ===", nil)
	k.Logger.Info("Processing user conversation with 7 traced components", nil)

	// Simulate a user conversation
	conversationReq := ConversationRequest{
		RequestID:     "conv-12345",
		UserID:        "user-789",
		SessionID:     "session-abc",
		UserMessage:   "What are the best practices for distributed tracing in microservices?",
		MessageLength: 73,
		Timestamp:     time.Now().Format(time.RFC3339),
	}

	// Process the conversation pipeline
	err = processMaleniaConversation(ctx, k, conversationReq)
	if err != nil {
		_ = k.Logger.Error("Conversation failed", map[string]interface{}{"error": err.Error()})
	} else {
		k.Logger.Info("✅ Conversation completed successfully!", nil)
	}

	// Give time for spans to be exported
	time.Sleep(2 * time.Second)
}

// processMaleniaConversation orchestrates the complete conversation pipeline with 7 traced components
func processMaleniaConversation(ctx context.Context, k *pulse.Pulse, req ConversationRequest) error {
	// Create root span for the entire conversation pipeline
	return k.Tracing.Trace(ctx, "MaleniaConversationPipeline", req, func(ctx context.Context, span *pulse.Span) error {
		span.AddEvent("conversation_started")
		span.SetAttribute("assistant.name", "Malenia")
		span.SetAttribute("assistant.version", "1.0.0")

		var totalTime float64

		// Component 1: Input Processing
		span.AddEvent("component_1_input_processing")
		inputResp, err := processInput(ctx, k, InputProcessingRequest{
			RequestID:   req.RequestID,
			RawInput:    req.UserMessage,
			InputLength: req.MessageLength,
			Language:    "en",
		})
		if err != nil {
			return fmt.Errorf("input processing failed: %w", err)
		}
		span.SetAttribute("input.valid", inputResp.IsValid)
		span.SetAttribute("input.tokens", inputResp.TokenCount)
		totalTime += inputResp.ProcessingTimeMs

		// Component 2: Context Retrieval
		span.AddEvent("component_2_context_retrieval")
		contextResp, err := retrieveContext(ctx, k, ContextRetrievalRequest{
			RequestID: req.RequestID,
			UserID:    req.UserID,
			SessionID: req.SessionID,
			MaxTurns:  10,
		})
		if err != nil {
			return fmt.Errorf("context retrieval failed: %w", err)
		}
		span.SetAttribute("context.history_turns", contextResp.HistoryTurns)
		span.SetAttribute("context.cache_hit", contextResp.CacheHit)
		totalTime += contextResp.ProcessingTimeMs

		// Component 3: Intent Classification
		span.AddEvent("component_3_intent_classification")
		intentResp, err := classifyIntent(ctx, k, IntentClassificationRequest{
			RequestID: req.RequestID,
			Message:   inputResp.ProcessedInput,
			Context:   "conversation_history",
		})
		if err != nil {
			return fmt.Errorf("intent classification failed: %w", err)
		}
		span.SetAttribute("intent.name", intentResp.Intent)
		span.SetAttribute("intent.confidence", intentResp.Confidence)
		totalTime += intentResp.ProcessingTimeMs

		// Component 4: Knowledge Search
		span.AddEvent("component_4_knowledge_search")
		searchResp, err := searchKnowledge(ctx, k, KnowledgeSearchRequest{
			RequestID: req.RequestID,
			Query:     inputResp.ProcessedInput,
			Intent:    intentResp.Intent,
			TopK:      5,
			Threshold: 0.7,
		})
		if err != nil {
			return fmt.Errorf("knowledge search failed: %w", err)
		}
		span.SetAttribute("search.result_count", searchResp.ResultCount)
		span.SetAttribute("search.avg_relevance", searchResp.AvgRelevance)
		totalTime += searchResp.ProcessingTimeMs

		// Component 5: Response Generation
		span.AddEvent("component_5_response_generation")
		responseResp, err := generateResponse(ctx, k, ResponseGenerationRequest{
			RequestID:        req.RequestID,
			UserMessage:      inputResp.ProcessedInput,
			SystemPrompt:     "You are Malenia, a helpful AI assistant specializing in software engineering and distributed systems.",
			Context:          fmt.Sprintf("History: %d turns", contextResp.HistoryTurns),
			KnowledgeContext: fmt.Sprintf("Found %d relevant documents", searchResp.ResultCount),
			ModelName:        "gpt-4-turbo",
			Temperature:      0.7,
			MaxTokens:        500,
		})
		if err != nil {
			return fmt.Errorf("response generation failed: %w", err)
		}
		span.SetAttribute("llm.tokens_total", responseResp.TokensTotal)
		span.SetAttribute("llm.finish_reason", responseResp.FinishReason)
		totalTime += responseResp.ProcessingTimeMs

		// Component 6: Response Validation
		span.AddEvent("component_6_response_validation")
		validationResp, err := validateResponse(ctx, k, ResponseValidationRequest{
			RequestID:      req.RequestID,
			Response:       responseResp.Response,
			ResponseLength: len(responseResp.Response),
		})
		if err != nil {
			return fmt.Errorf("response validation failed: %w", err)
		}
		span.SetAttribute("validation.is_valid", validationResp.IsValid)
		span.SetAttribute("validation.is_safe", validationResp.IsSafe)
		totalTime += validationResp.ProcessingTimeMs

		// Component 7: Output Formatting
		span.AddEvent("component_7_output_formatting")
		outputResp, err := formatOutput(ctx, k, OutputFormattingRequest{
			RequestID: req.RequestID,
			Response:  responseResp.Response,
			Format:    "markdown",
		})
		if err != nil {
			return fmt.Errorf("output formatting failed: %w", err)
		}
		span.SetAttribute("output.length", outputResp.OutputLength)
		span.SetAttribute("output.markdown", outputResp.Markdown)
		totalTime += outputResp.ProcessingTimeMs

		span.AddEvent("conversation_completed")
		span.SetAttribute("pipeline.total_time_ms", totalTime)
		span.SetAttribute("pipeline.components_completed", 7)
		span.SetAttribute("pipeline.success", true)

		return nil
	})
}

// Component 1: Input Processing
func processInput(ctx context.Context, k *pulse.Pulse, req InputProcessingRequest) (*InputProcessingResponse, error) {
	_, span := k.Tracing.Start(ctx, "InputProcessing", req)
	defer span.End()

	span.AddEvent("validating_input")
	time.Sleep(15 * time.Millisecond)

	span.AddEvent("normalizing_text")
	time.Sleep(20 * time.Millisecond)

	span.AddEvent("detecting_language")
	time.Sleep(10 * time.Millisecond)

	span.AddEvent("tokenizing")
	time.Sleep(25 * time.Millisecond)

	response := &InputProcessingResponse{
		RequestID:        req.RequestID,
		ProcessedInput:   req.RawInput,
		IsValid:          true,
		TokenCount:       18,
		DetectedLanguage: "en",
		ProcessingTimeMs: 70.0,
	}

	span.AddEvent("input_processing_complete")
	span.SetOK()

	return response, nil
}

// Component 2: Context Retrieval
func retrieveContext(ctx context.Context, k *pulse.Pulse, req ContextRetrievalRequest) (*ContextRetrievalResponse, error) {
	_, span := k.Tracing.Start(ctx, "ContextRetrieval", req)
	defer span.End()

	span.AddEvent("checking_cache")
	time.Sleep(5 * time.Millisecond)

	span.AddEvent("fetching_conversation_history")
	time.Sleep(40 * time.Millisecond)

	span.AddEvent("loading_user_preferences")
	time.Sleep(30 * time.Millisecond)

	span.AddEvent("aggregating_context")
	time.Sleep(15 * time.Millisecond)

	response := &ContextRetrievalResponse{
		RequestID:        req.RequestID,
		HistoryTurns:     5,
		UserPreferences:  []string{"technical", "detailed", "examples"},
		ContextTokens:    450,
		CacheHit:         true,
		ProcessingTimeMs: 90.0,
	}

	span.AddEvent("context_retrieval_complete")
	span.SetOK()

	return response, nil
}

// Component 3: Intent Classification
func classifyIntent(ctx context.Context, k *pulse.Pulse, req IntentClassificationRequest) (*IntentClassificationResponse, error) {
	_, span := k.Tracing.Start(ctx, "IntentClassification", req)
	defer span.End()

	span.AddEvent("loading_classifier_model")
	time.Sleep(20 * time.Millisecond)

	span.AddEvent("extracting_features")
	time.Sleep(30 * time.Millisecond)

	span.AddEvent("running_classification")
	time.Sleep(50 * time.Millisecond)

	span.AddEvent("extracting_entities")
	time.Sleep(35 * time.Millisecond)

	response := &IntentClassificationResponse{
		RequestID:        req.RequestID,
		Intent:           "technical_question",
		Confidence:       0.94,
		Entities:         []string{"distributed_tracing", "microservices", "best_practices"},
		EntityCount:      3,
		ProcessingTimeMs: 135.0,
	}

	span.AddEvent("intent_classification_complete")
	span.SetOK()

	return response, nil
}

// Component 4: Knowledge Search
func searchKnowledge(ctx context.Context, k *pulse.Pulse, req KnowledgeSearchRequest) (*KnowledgeSearchResponse, error) {
	_, span := k.Tracing.Start(ctx, "KnowledgeSearch", req)
	defer span.End()

	span.AddEvent("generating_query_embedding")
	time.Sleep(45 * time.Millisecond)

	span.AddEvent("searching_vector_index")
	time.Sleep(120 * time.Millisecond)

	span.AddEvent("filtering_by_relevance")
	time.Sleep(20 * time.Millisecond)

	span.AddEvent("ranking_results")
	time.Sleep(30 * time.Millisecond)

	response := &KnowledgeSearchResponse{
		RequestID:        req.RequestID,
		ResultCount:      5,
		DocumentIDs:      []string{"doc-101", "doc-202", "doc-303", "doc-404", "doc-505"},
		AvgRelevance:     0.87,
		CacheHit:         false,
		ProcessingTimeMs: 215.0,
	}

	span.AddEvent("knowledge_search_complete")
	span.SetOK()

	return response, nil
}

// Component 5: Response Generation
func generateResponse(ctx context.Context, k *pulse.Pulse, req ResponseGenerationRequest) (*ResponseGenerationResponse, error) {
	_, span := k.Tracing.Start(ctx, "ResponseGeneration", req)
	defer span.End()

	span.AddEvent("preparing_prompt")
	time.Sleep(20 * time.Millisecond)

	span.AddEvent("tokenizing_input")
	time.Sleep(25 * time.Millisecond)

	span.AddEvent("calling_llm_api")
	time.Sleep(180 * time.Millisecond)

	span.AddEvent("parsing_response")
	time.Sleep(15 * time.Millisecond)

	span.AddEvent("detokenizing_output")
	time.Sleep(10 * time.Millisecond)

	response := &ResponseGenerationResponse{
		RequestID:        req.RequestID,
		Response:         "Distributed tracing in microservices involves several best practices: 1) Use correlation IDs across all services, 2) Implement context propagation, 3) Trace critical paths, 4) Set appropriate sampling rates, 5) Monitor span durations and error rates.",
		TokensPrompt:     520,
		TokensCompletion: 95,
		TokensTotal:      615,
		FinishReason:     "stop",
		ProcessingTimeMs: 250.0,
	}

	span.AddEvent("response_generation_complete")
	span.SetOK()

	return response, nil
}

// Component 6: Response Validation
func validateResponse(ctx context.Context, k *pulse.Pulse, req ResponseValidationRequest) (*ResponseValidationResponse, error) {
	_, span := k.Tracing.Start(ctx, "ResponseValidation", req)
	defer span.End()

	span.AddEvent("checking_content_safety")
	time.Sleep(40 * time.Millisecond)

	span.AddEvent("detecting_pii")
	time.Sleep(30 * time.Millisecond)

	span.AddEvent("calculating_toxicity_score")
	time.Sleep(25 * time.Millisecond)

	span.AddEvent("validating_format")
	time.Sleep(15 * time.Millisecond)

	response := &ResponseValidationResponse{
		RequestID:        req.RequestID,
		IsValid:          true,
		IsSafe:           true,
		HasPII:           false,
		ToxicityScore:    0.02,
		ProcessingTimeMs: 110.0,
	}

	span.AddEvent("response_validation_complete")
	span.SetOK()

	return response, nil
}

// Component 7: Output Formatting
func formatOutput(ctx context.Context, k *pulse.Pulse, req OutputFormattingRequest) (*OutputFormattingResponse, error) {
	_, span := k.Tracing.Start(ctx, "OutputFormatting", req)
	defer span.End()

	span.AddEvent("applying_markdown_formatting")
	time.Sleep(20 * time.Millisecond)

	span.AddEvent("adding_citations")
	time.Sleep(15 * time.Millisecond)

	span.AddEvent("adding_metadata")
	time.Sleep(10 * time.Millisecond)

	span.AddEvent("finalizing_output")
	time.Sleep(10 * time.Millisecond)

	response := &OutputFormattingResponse{
		RequestID:        req.RequestID,
		FormattedOutput:  req.Response + "\n\n*Powered by Malenia AI*",
		OutputLength:     len(req.Response) + 25,
		Markdown:         true,
		ProcessingTimeMs: 55.0,
	}

	span.AddEvent("output_formatting_complete")
	span.SetOK()

	return response, nil
}

// Error handling example
func runMaleniaWithError() {
	ctx := context.Background()

	k, err := pulse.New(ctx, options.ServiceOptions{
		Name:        "malenia-conversation-service",
		Version:     "1.0.0",
		Environment: options.Development,
	}, options.PulseOptions{
		Telemetry: options.TelemetryOptions{
			Tracing: options.TracingTelemetryOptions{Enabled: true},
			OTLP:    options.OTLPOptions{Host: "localhost", Port: 4317, Enabled: true},
		},
		Tracing: options.TracingOptions{Enabled: true},
	})
	if err != nil {
		panic(err)
	}
	defer func() { _ = k.Close(ctx) }()

	k.Logger.Info("=== Malenia Pipeline with Error Handling ===", nil)

	conversationReq := ConversationRequest{
		RequestID:     "conv-error-test",
		UserID:        "user-123",
		SessionID:     "session-xyz",
		UserMessage:   "Test error handling",
		MessageLength: 19,
		Timestamp:     time.Now().Format(time.RFC3339),
	}

	// Simulate pipeline with error
	err = k.Tracing.Trace(ctx, "MaleniaConversationPipeline", conversationReq, func(ctx context.Context, span *pulse.Span) error {
		// Input processing succeeds
		_, err := processInput(ctx, k, InputProcessingRequest{
			RequestID:   conversationReq.RequestID,
			RawInput:    conversationReq.UserMessage,
			InputLength: conversationReq.MessageLength,
			Language:    "en",
		})
		if err != nil {
			return err
		}

		// Simulate LLM rate limit error
		span.AddEvent("llm_rate_limit_exceeded")
		return errors.New("LLM service rate limit exceeded - please retry")
	})

	if err != nil {
		_ = k.Logger.Error("❌ Conversation failed with error (automatically recorded in trace)", map[string]interface{}{"error": err.Error()})
	}

	time.Sleep(2 * time.Second)
}
