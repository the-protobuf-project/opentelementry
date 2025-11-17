package options

// TelemetryOptions defines the configuration for the unified telemetry service
// that integrates OpenTelemetry for logging, metrics, and tracing.
type TelemetryOptions struct {
	Logging LoggingTelemetryOptions `json:"logging"` // Logging telemetry options
	Metrics MetricsTelemetryOptions `json:"metrics"` // Metrics telemetry options
	Tracing TracingTelemetryOptions `json:"tracing"` // Tracing telemetry options
	OTLP    OTLPOptions             `json:"otlp"`    // OTLP exporter options
}

// LoggingTelemetryOptions defines the configuration for OpenTelemetry logging
type LoggingTelemetryOptions struct {
	Enabled bool `json:"enabled"` // Enable logging
}

// MetricsTelemetryOptions defines the configuration for OpenTelemetry metrics
type MetricsTelemetryOptions struct {
	Enabled               bool `json:"enabled"`               // Enable metrics
	ExportIntervalSeconds int  `json:"exportIntervalSeconds"` // Export interval in seconds
}

// TracingTelemetryOptions defines the configuration for OpenTelemetry tracing
type TracingTelemetryOptions struct {
	Enabled bool `json:"enabled"` // Enable tracing
}

// OTLPOptions defines the settings for OTLP exporter
type OTLPOptions struct {
	Host    string `json:"host"`    // OTLP collector host (e.g., "localhost")
	Port    int    `json:"port"`    // OTLP collector port (e.g., 4317 for gRPC)
	Enabled bool   `json:"enabled"` // Enable OTLP export (if false, uses stdout)
}
