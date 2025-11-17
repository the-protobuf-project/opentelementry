package pulse

import (
	"context"

	"github.com/machanirobotics/pulse/internal/foxglove"
	"github.com/machanirobotics/pulse/internal/logging"
	"github.com/machanirobotics/pulse/internal/metrics"
	"github.com/machanirobotics/pulse/internal/profiling"
	"github.com/machanirobotics/pulse/internal/telemetry"
	"github.com/machanirobotics/pulse/internal/tracing"
	"github.com/machanirobotics/pulse/options"
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
}

// New creates a new Pulse instance with both legacy and unified telemetry services.
// The unified telemetry service (Telemetry) is the recommended approach for new applications.
func New(ctx context.Context, serviceOpts options.ServiceOptions, opts options.PulseOptions) (*Pulse, error) {
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
		telemetry:   tel,
		unifiedMcap: unifiedMcap,
		Logger:      logging.NewLogger(serviceOpts, opts.Logging, unifiedMcap, tel.GetLogger()),
		Metrics:     metrics.NewMetrics(serviceOpts, unifiedMcap, tel.GetMetrics()),
		Tracing:     tracing.NewTracing(serviceOpts, opts.Tracing, unifiedMcap, tel.GetTracer()),
		Profiler:    profiling.NewProfiler(serviceOpts, opts.Profiling, unifiedMcap),
	}

	return p, nil
}

// Shutdown gracefully shuts down all telemetry services
func (p *Pulse) Close(ctx context.Context) error {
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
		return p.telemetry.Shutdown(ctx)
	}
	return nil
}
