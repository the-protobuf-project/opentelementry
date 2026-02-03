package pulse

import (
	"context"
	"net"
	"strings"

	"github.com/machanirobotics/pulse/pulse-go/internal/foxglove"
	"github.com/machanirobotics/pulse/pulse-go/internal/logging"
	"github.com/machanirobotics/pulse/pulse-go/internal/metrics"
	"github.com/machanirobotics/pulse/pulse-go/internal/profiling"
	"github.com/machanirobotics/pulse/pulse-go/internal/telemetry"
	"github.com/machanirobotics/pulse/pulse-go/internal/tracing"
	"github.com/machanirobotics/pulse/pulse-go/options"
)

// Span is a type alias for tracing.Span to avoid exposing internal packages
type Span = tracing.Span

// Pulse is the main framework struct that provides access to all telemetry services.
// It supports both the legacy logging interface and the new unified OpenTelemetry-based telemetry.
type Pulse struct {
	// Logger is the main logging client.
	Logger *logging.Logger
	// Metrics is the main metrics client.
	Metrics *metrics.Metrics
	// Tracing is the main tracing client.
	Tracing *tracing.Tracing
	// Profiler is the main profiler client.
	Profiler *profiling.Profiler

	// Unified OpenTelemetry-based telemetry
	telemetry *telemetry.Telemetry
	// Unified MCAP writer for both logs and metrics
	unifiedMcap *foxglove.UnifiedMcapWriter
	// Internal context for background operations
	ctx context.Context
}

// Builder provides a fluent API for configuring and creating a Pulse instance.
type Builder struct {
	serviceOpts options.ServiceOptions
	pulseOpts   options.PulseOptions
	configPath  string
	err         error
}

// New creates a new Pulse Builder with sensible defaults.
// Use the builder methods to configure, then call Build() to create the Pulse instance.
//
// Example:
//
//	p, err := pulse.New().
//	    WithService("my-service", "1.0.0").
//	    WithConfig("config.yaml").
//	    Build()
func New() *Builder {
	// Auto-discover and load config on creation
	pulseOpts, serviceOpts, _ := options.LoadConfigWithDefaults("")
	return &Builder{
		serviceOpts: *serviceOpts,
		pulseOpts:   *pulseOpts,
	}
}

// WithConfig loads configuration from a specific file (YAML, JSON, or TOML).
// Use this to override the auto-discovered config.
// Environment variables with PULSE_ prefix override file values.
func (b *Builder) WithConfig(configPath string) *Builder {
	if b.err != nil {
		return b
	}
	if configPath == "" {
		return b // Already loaded via auto-discovery
	}
	b.configPath = configPath
	pulseOpts, serviceOpts, err := options.LoadConfigWithDefaults(configPath)
	if err != nil {
		b.err = err
		return b
	}
	b.pulseOpts = *pulseOpts
	b.serviceOpts = *serviceOpts
	return b
}

// WithService sets the service name and version.
func (b *Builder) WithService(name, version string) *Builder {
	if b.err != nil {
		return b
	}
	b.serviceOpts.Name = name
	b.serviceOpts.Version = version
	return b
}

// WithDescription sets the service description.
func (b *Builder) WithDescription(description string) *Builder {
	if b.err != nil {
		return b
	}
	b.serviceOpts.Description = description
	return b
}

// WithEnvironment sets the deployment environment.
func (b *Builder) WithEnvironment(env options.Environment) *Builder {
	if b.err != nil {
		return b
	}
	b.serviceOpts.Environment = env
	return b
}

// WithOTLP configures the OTLP endpoint. Auto-detects if local or remote
// and configures TLS accordingly.
func (b *Builder) WithOTLP(host string, port int) *Builder {
	if b.err != nil {
		return b
	}
	b.pulseOpts.Telemetry.OTLP.Host = host
	b.pulseOpts.Telemetry.OTLP.Port = port
	b.pulseOpts.Telemetry.OTLP.Enabled = true

	// Auto-configure based on host
	b.pulseOpts.Telemetry.OTLP.Secure = !isLocalHost(host)
	return b
}

// WithOTLPHeaders sets custom headers for OTLP requests (e.g., Authorization).
func (b *Builder) WithOTLPHeaders(headers map[string]string) *Builder {
	if b.err != nil {
		return b
	}
	b.pulseOpts.Telemetry.OTLP.Headers = headers
	return b
}

// WithMCAP enables MCAP file logging at the specified path.
func (b *Builder) WithMCAP(path string) *Builder {
	if b.err != nil {
		return b
	}
	b.pulseOpts.Foxglove.Enabled = true
	b.pulseOpts.Foxglove.McapPath = path
	return b
}

// WithProfiling enables continuous profiling with Pyroscope.
func (b *Builder) WithProfiling(serverAddress string) *Builder {
	if b.err != nil {
		return b
	}
	b.pulseOpts.Profiling.Enabled = true
	b.pulseOpts.Profiling.ServerAddress = serverAddress
	return b
}

// WithTracing enables distributed tracing.
func (b *Builder) WithTracing() *Builder {
	if b.err != nil {
		return b
	}
	b.pulseOpts.Tracing.Enabled = true
	b.pulseOpts.Telemetry.Tracing.Enabled = true
	return b
}

// WithAttributes sets global attributes that are added to all telemetry data.
// Use this for constant identifiers like robot.id, device.id, fleet.id, etc.
// These attributes will appear on all logs, metrics, and traces.
//
// Example:
//
//	p, err := pulse.New().
//	    WithService("robot-controller", "1.0.0").
//	    WithAttributes(map[string]string{
//	        "robot.id":    "robot-001",
//	        "fleet.id":    "fleet-alpha",
//	        "location.id": "warehouse-1",
//	    }).
//	    Build()
func (b *Builder) WithAttributes(attrs map[string]string) *Builder {
	if b.err != nil {
		return b
	}
	if b.serviceOpts.Attributes == nil {
		b.serviceOpts.Attributes = make(map[string]string)
	}
	for k, v := range attrs {
		b.serviceOpts.Attributes[k] = v
	}
	return b
}

// WithAttribute sets a single global attribute.
// Convenience method for adding one attribute at a time.
func (b *Builder) WithAttribute(key, value string) *Builder {
	if b.err != nil {
		return b
	}
	if b.serviceOpts.Attributes == nil {
		b.serviceOpts.Attributes = make(map[string]string)
	}
	b.serviceOpts.Attributes[key] = value
	return b
}

// Build creates the Pulse instance with the configured options.
func (b *Builder) Build() (*Pulse, error) {
	if b.err != nil {
		return nil, b.err
	}

	// Auto-configure OTLP settings based on host
	autoConfigureOTLP(&b.pulseOpts.Telemetry.OTLP)

	ctx := context.Background()

	// Initialize unified telemetry service
	tel, err := telemetry.New(ctx, b.serviceOpts, b.pulseOpts.Telemetry)
	if err != nil {
		return nil, err
	}

	// Initialize unified MCAP writer if Foxglove is enabled
	var unifiedMcap *foxglove.UnifiedMcapWriter
	if b.pulseOpts.Foxglove.Enabled && b.pulseOpts.Foxglove.McapPath != "" {
		unifiedMcap, err = foxglove.NewUnifiedMcapWriter(b.serviceOpts, b.pulseOpts.Foxglove)
		if err != nil {
			return nil, err
		}
	}

	p := &Pulse{
		ctx:         ctx,
		telemetry:   tel,
		unifiedMcap: unifiedMcap,
		Logger:      logging.NewLogger(b.serviceOpts, b.pulseOpts.Logging, unifiedMcap, tel.GetLogger()),
		Metrics:     metrics.NewMetrics(b.serviceOpts, unifiedMcap, tel.GetMetrics()),
		Tracing:     tracing.NewTracing(b.serviceOpts, b.pulseOpts.Tracing, unifiedMcap, tel.GetTracer()),
		Profiler:    profiling.NewProfiler(b.serviceOpts, b.pulseOpts.Profiling, unifiedMcap),
	}

	return p, nil
}

// isLocalHost checks if the host is a local address (localhost, 127.x.x.x, or private IP).
func isLocalHost(host string) bool {
	host = strings.ToLower(host)
	if host == "localhost" || host == "" {
		return true
	}

	ip := net.ParseIP(host)
	if ip == nil {
		// Not an IP, check if it's a domain
		return false
	}

	// Check for loopback (127.x.x.x)
	if ip.IsLoopback() {
		return true
	}

	// Check for private IPs (10.x.x.x, 172.16-31.x.x, 192.168.x.x)
	if ip.IsPrivate() {
		return true
	}

	return false
}

// autoConfigureOTLP automatically configures OTLP settings based on the host.
func autoConfigureOTLP(otlp *options.OTLPOptions) {
	if otlp.Host == "" {
		return
	}

	isLocal := isLocalHost(otlp.Host)

	// Auto-set secure based on host (unless explicitly set)
	if !otlp.Secure && !isLocal {
		// Remote host on standard OTLP ports - check if it needs TLS
		if otlp.Port == 443 || otlp.Port == 4318 {
			otlp.Secure = true
			otlp.UseHTTP = true // Port 443 typically needs HTTP
		} else if otlp.Port == 4317 {
			// Standard gRPC port - may or may not need TLS
			// Keep as-is, user should configure if needed
		}
	}
}

// NewLegacy creates a new Pulse instance using the legacy API (for backward compatibility).
// Deprecated: Use New().WithConfig().Build() instead.
func NewLegacy(ctx context.Context, serviceOpts options.ServiceOptions, opts options.PulseOptions) (*Pulse, error) {
	// Auto-configure OTLP settings
	autoConfigureOTLP(&opts.Telemetry.OTLP)

	// Initialize unified telemetry service
	tel, err := telemetry.New(ctx, serviceOpts, opts.Telemetry)
	if err != nil {
		return nil, err
	}

	// Initialize unified MCAP writer if Foxglove is enabled
	var unifiedMcap *foxglove.UnifiedMcapWriter
	if opts.Foxglove.Enabled && opts.Foxglove.McapPath != "" {
		unifiedMcap, err = foxglove.NewUnifiedMcapWriter(serviceOpts, opts.Foxglove)
		if err != nil {
			return nil, err
		}
	}

	p := &Pulse{
		ctx:         ctx,
		telemetry:   tel,
		unifiedMcap: unifiedMcap,
		Logger:      logging.NewLogger(serviceOpts, opts.Logging, unifiedMcap, tel.GetLogger()),
		Metrics:     metrics.NewMetrics(serviceOpts, unifiedMcap, tel.GetMetrics()),
		Tracing:     tracing.NewTracing(serviceOpts, opts.Tracing, unifiedMcap, tel.GetTracer()),
		Profiler:    profiling.NewProfiler(serviceOpts, opts.Profiling, unifiedMcap),
	}

	return p, nil
}

// Close gracefully shuts down all telemetry services.
// Uses the internal context created during Build().
func (p *Pulse) Close() error {
	// Stop profiler first to flush remaining data
	if p.Profiler != nil {
		if err := p.Profiler.Stop(); err != nil {
			// Log error but continue with shutdown
			if p.Logger != nil {
				p.Logger.Warn("Failed to stop profiler", map[string]interface{}{"error": err.Error()})
			}
		}
	}

	// Close unified MCAP writer first (before logger tries to log about it)
	if p.unifiedMcap != nil {
		_ = p.unifiedMcap.Close() // Ignore error during shutdown
	}

	// Close metrics (no-op since unified writer is already closed)
	if p.Metrics != nil {
		_ = p.Metrics.Close() // Ignore error during shutdown
	}

	// Close logger (no-op since unified writer is already closed)
	if p.Logger != nil {
		_ = p.Logger.Close() // Ignore error during shutdown
	}

	// Close tracing (no-op since unified writer is already closed)
	if p.Tracing != nil {
		_ = p.Tracing.Close() // Ignore error during shutdown
	}

	if p.telemetry != nil {
		ctx := p.ctx
		if ctx == nil {
			ctx = context.Background()
		}
		return p.telemetry.Shutdown(ctx)
	}
	return nil
}
