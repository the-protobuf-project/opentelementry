package metrics

import (
	"context"
	"fmt"
	"reflect"
	"strings"

	"github.com/machanirobotics/pulse/internal/foxglove"
	"github.com/machanirobotics/pulse/internal/telemetry"
	"github.com/machanirobotics/pulse/options"
	"go.opentelemetry.io/otel/metric"
)

// Metrics wraps OpenTelemetry metrics with struct tag support and MCAP export
type Metrics struct {
	otelMetrics *telemetry.Metrics
	mcapWriter  *MetricMcapWriter
	ctx         context.Context
	registered  map[string]bool // Track registered metrics
}

// NewMetrics creates a new Metrics instance
func NewMetrics(serviceOpts options.ServiceOptions, unifiedWriter *foxglove.UnifiedMcapWriter, otelMetrics *telemetry.Metrics) *Metrics {
	m := &Metrics{
		otelMetrics: otelMetrics,
		ctx:         context.Background(),
		registered:  make(map[string]bool),
	}

	// Initialize MCAP writer if unified writer is provided
	if unifiedWriter != nil {
		writer, err := NewMetricMcapWriter(serviceOpts, unifiedWriter)
		if err != nil {
			// Log error but continue - metrics will still work via OTEL
			// The error will be visible in the logs
			fmt.Printf("Warning: Failed to initialize MCAP metrics writer: %v\n", err)
		} else {
			m.mcapWriter = writer
		}
	}

	return m
}

// Record records a metric value from a struct with tags
// Tag format: `pulse:"metric:type:name"` where type is counter, histogram, gauge
func (m *Metrics) Record(v any, attrs ...metric.AddOption) error {
	if v == nil {
		return nil
	}

	rv := reflect.ValueOf(v)
	if rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			return nil
		}
		rv = rv.Elem()
	}

	if rv.Kind() != reflect.Struct {
		return fmt.Errorf("Record requires a struct, got %T", v)
	}

	return m.extractAndRecordMetrics(rv, attrs...)
}

// extractAndRecordMetrics extracts metrics from struct tags and records them
func (m *Metrics) extractAndRecordMetrics(rv reflect.Value, attrs ...metric.AddOption) error {
	rt := rv.Type()

	for i := 0; i < rv.NumField(); i++ {
		field := rt.Field(i)
		fieldValue := rv.Field(i)

		if !field.IsExported() {
			continue
		}

		tag := field.Tag.Get("pulse")
		if tag == "" || !strings.HasPrefix(tag, "metric:") {
			continue
		}

		// Parse tag: "metric:type:name"
		parts := strings.Split(tag, ":")
		if len(parts) < 3 {
			continue
		}

		metricType := parts[1]
		metricName := parts[2]

		// Record metric based on type
		if err := m.recordMetric(metricType, metricName, fieldValue, attrs...); err != nil {
			return err
		}
	}

	return nil
}

// recordMetric records a single metric value
func (m *Metrics) recordMetric(metricType, name string, value reflect.Value, attrs ...metric.AddOption) error {
	switch metricType {
	case "counter":
		return m.recordCounter(name, value, attrs...)
	case "histogram":
		return m.recordHistogram(name, value, attrs...)
	case "gauge":
		return m.recordGauge(name, value, attrs...)
	default:
		return fmt.Errorf("unknown metric type: %s", metricType)
	}
}

// recordCounter records a counter metric
func (m *Metrics) recordCounter(name string, value reflect.Value, attrs ...metric.AddOption) error {
	var val float64
	switch value.Kind() {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		val = float64(value.Int())
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		val = float64(value.Uint())
	case reflect.Float32, reflect.Float64:
		val = value.Float()
	default:
		return fmt.Errorf("counter requires numeric value, got %v", value.Kind())
	}

	counter, err := m.otelMetrics.FloatCounter(name)
	if err != nil {
		return err
	}
	counter.Add(m.ctx, val, attrs...)

	// Write to MCAP
	if m.mcapWriter != nil {
		return m.mcapWriter.WriteCounter(name, val)
	}
	return nil
}

// recordHistogram records a histogram metric
func (m *Metrics) recordHistogram(name string, value reflect.Value, attrs ...metric.AddOption) error {
	var val float64
	switch value.Kind() {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		val = float64(value.Int())
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		val = float64(value.Uint())
	case reflect.Float32, reflect.Float64:
		val = value.Float()
	default:
		return fmt.Errorf("histogram requires numeric value, got %v", value.Kind())
	}

	hist, err := m.otelMetrics.FloatHistogram(name)
	if err != nil {
		return err
	}
	hist.Record(m.ctx, val)

	// Write to MCAP
	if m.mcapWriter != nil {
		return m.mcapWriter.WriteHistogram(name, val)
	}
	return nil
}

// recordGauge records a gauge metric (using UpDownCounter for simplicity)
func (m *Metrics) recordGauge(name string, value reflect.Value, attrs ...metric.AddOption) error {
	var val float64
	switch value.Kind() {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		val = float64(value.Int())
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		val = float64(value.Uint())
	case reflect.Float32, reflect.Float64:
		val = value.Float()
	default:
		return fmt.Errorf("gauge requires numeric value, got %v", value.Kind())
	}

	// Use UpDownCounter as a gauge (can go up and down)
	gauge, err := m.otelMetrics.FloatUpDownCounter(name)
	if err != nil {
		return err
	}
	gauge.Add(m.ctx, val, attrs...)

	// Write to MCAP
	if m.mcapWriter != nil {
		return m.mcapWriter.WriteGauge(name, val)
	}
	return nil
}

// Close closes the metrics system
func (m *Metrics) Close() error {
	if m.mcapWriter != nil {
		return m.mcapWriter.Close()
	}
	return nil
}
