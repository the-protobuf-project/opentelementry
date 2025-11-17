package logging

import (
	"encoding/json"
	"fmt"
	"reflect"
	"runtime"
	"strings"
	"time"

	"github.com/charmbracelet/log"
	"github.com/machanirobotics/pulse/pulse-go/options"
	otellog "go.opentelemetry.io/otel/log"
)

// formatPrefix formats the prefix for the logger.
func formatPrefix(serviceOpts options.ServiceOptions) string {
	return fmt.Sprintf("%s (%s | %s)", serviceOpts.Name, serviceOpts.Version, serviceOpts.Environment)
}

// resolveTimeFormat determines the appropriate time format string.
func resolveTimeFormat(opts options.LoggingOptions) string {
	switch opts.Log.TimeFormatKey {
	case options.TimeFormatRFC3339:
		return time.RFC3339
	case options.TimeFormatRFC3339Nano:
		return time.RFC3339Nano
	case options.TimeFormatKitchen:
		return time.Kitchen
	case options.TimeFormatStamp:
		return "Jan _2 15:04:05"
	case options.TimeFormatCustom:
		if opts.Log.CustomFormat != "" {
			return opts.Log.CustomFormat
		}
		return time.RFC3339
	default:
		return time.RFC3339
	}
}

// resolveLogLevel sets default log level based on environment.
func resolveLogLevel(env options.Environment) log.Level {
	switch env {
	case options.Production:
		return log.InfoLevel
	case options.Staging:
		return log.WarnLevel
	case options.Jetson, options.Development:
		return log.DebugLevel
	default:
		return log.InfoLevel
	}
}

// resolveCallerOffset returns the correct caller offset.
func resolveCallerOffset(opts options.LoggingOptions) int {
	if opts.Log.CallerOffset > 0 {
		return opts.Log.CallerOffset
	}
	// Default offset to skip internal logging wrapper functions
	return 2
}

// extractStructTagAttributes extracts attributes from struct fields with `pulse:"attribute:key_name"` tags
func extractStructTagAttributes(rv reflect.Value) []otellog.KeyValue {
	if rv.Kind() != reflect.Struct {
		return nil
	}

	attrs := []otellog.KeyValue{}
	rt := rv.Type()

	for i := 0; i < rv.NumField(); i++ {
		field := rt.Field(i)
		fieldValue := rv.Field(i)

		// Skip unexported fields
		if !field.IsExported() {
			continue
		}

		// Check for pulse struct tag
		tag := field.Tag.Get("pulse")
		if tag == "" {
			continue
		}

		// Parse tag format: "attribute:key_name"
		if strings.HasPrefix(tag, "attribute:") {
			attrName := strings.TrimPrefix(tag, "attribute:")
			if attrName != "" {
				// Convert field value to appropriate OTEL attribute
				attrs = append(attrs, convertToOtelKeyValue(attrName, fieldValue.Interface()))
			}
		}
	}

	// Add dynamic/computed attributes
	// Example: Add a timestamp if not present
	attrs = append(attrs, otellog.Int64("extracted_at", time.Now().Unix()))

	// Example: Add struct type name
	attrs = append(attrs, otellog.String("struct_type", rt.Name()))

	return attrs
}

// dataToOtelAttributes converts various data types to OpenTelemetry KeyValue attributes
// It extracts struct tags with format `pulse:"attribute:key_name"` and adds them as attributes
func dataToOtelAttributes(v any) []otellog.KeyValue {
	if v == nil {
		return nil
	}

	rv := reflect.ValueOf(v)

	// Handle pointers
	if rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			return nil
		}
		rv = rv.Elem()
		v = rv.Interface()
	}

	attrs := []otellog.KeyValue{}

	// Extract struct tag attributes if it's a struct
	if rv.Kind() == reflect.Struct {
		attrs = append(attrs, extractStructTagAttributes(rv)...)
	}

	// For all types, convert to JSON string and send as "data" attribute
	switch rv.Kind() {
	case reflect.Map, reflect.Struct, reflect.Slice, reflect.Array:
		// Marshal complex types to JSON
		if b, err := json.Marshal(v); err == nil {
			attrs = append(attrs, otellog.String("data", string(b)))
		} else {
			// Fallback to string representation if marshal fails
			attrs = append(attrs, otellog.String("data", fmt.Sprintf("%+v", v)))
		}

	default:
		// For primitive types, use convertToOtelKeyValue
		attrs = append(attrs, convertToOtelKeyValue("data", v))
	}

	return attrs
}

// convertToOtelKeyValue converts a key-value pair to an OpenTelemetry KeyValue
func convertToOtelKeyValue(key string, value any) otellog.KeyValue {
	if value == nil {
		return otellog.String(key, "<nil>")
	}

	rv := reflect.ValueOf(value)

	// Handle pointers
	if rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			return otellog.String(key, "<nil>")
		}
		rv = rv.Elem()
		value = rv.Interface()
	}

	switch rv.Kind() {
	case reflect.String:
		return otellog.String(key, rv.String())
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return otellog.Int64(key, rv.Int())
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return otellog.Int64(key, int64(rv.Uint()))
	case reflect.Float32, reflect.Float64:
		return otellog.Float64(key, rv.Float())
	case reflect.Bool:
		return otellog.Bool(key, rv.Bool())
	case reflect.Slice, reflect.Array:
		// Check if it's a byte slice
		if rv.Type().Elem().Kind() == reflect.Uint8 {
			return otellog.Bytes(key, value.([]byte))
		}
		// For other slices, convert to JSON string
		if b, err := json.Marshal(value); err == nil {
			return otellog.String(key, string(b))
		}
		return otellog.String(key, fmt.Sprintf("%+v", value))
	case reflect.Map, reflect.Struct:
		// Convert complex types to JSON string
		if b, err := json.Marshal(value); err == nil {
			return otellog.String(key, string(b))
		}
		return otellog.String(key, fmt.Sprintf("%+v", value))
	default:
		return otellog.String(key, fmt.Sprintf("%+v", value))
	}
}

// formattedData attempts to marshal structs, maps, or slices into
// pretty-printed JSON for console output. Fallbacks to fmt-compatible output for others.
func formattedData(v any) any {
	if v == nil {
		return "<nil>"
	}

	rv := reflect.ValueOf(v)

	// Handle pointers
	if rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			return "<nil>"
		}
		rv = rv.Elem()
		v = rv.Interface()
	}

	// Check if the type is a struct or map, which need to be marshaled.
	switch rv.Kind() {
	case reflect.Struct, reflect.Map:
		// Marshal struct/map into a pretty-printed JSON.
		if b, err := json.MarshalIndent(v, "", "  "); err == nil {
			return string(b)
		}
		// Fallback to default formatting if marshal fails
		return fmt.Sprintf("%+v", v)

	case reflect.Slice, reflect.Array:
		// Check if it's a byte slice
		if rv.Type().Elem().Kind() == reflect.Uint8 {
			return string(v.([]byte))
		}
		// Marshal other slices/arrays into JSON
		if b, err := json.MarshalIndent(v, "", "  "); err == nil {
			return string(b)
		}
		return fmt.Sprintf("%+v", v)

	case reflect.String:
		return v.(string)

	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return rv.Int()

	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return rv.Uint()

	case reflect.Float32, reflect.Float64:
		return rv.Float()

	case reflect.Bool:
		return rv.Bool()

	default:
		return fmt.Sprintf("%+v", v)
	}
}

// convertToMap converts any value to a map[string]interface{} for MCAP logging
func convertToMap(v any) map[string]interface{} {
	if v == nil {
		return nil
	}

	rv := reflect.ValueOf(v)

	// Handle pointers
	if rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			return nil
		}
		rv = rv.Elem()
		v = rv.Interface()
	}

	// If already a map, try to convert it
	if rv.Kind() == reflect.Map {
		result := make(map[string]interface{})
		for _, key := range rv.MapKeys() {
			keyStr := fmt.Sprintf("%v", key.Interface())
			result[keyStr] = rv.MapIndex(key).Interface()
		}
		return result
	}

	// For structs, marshal to JSON and unmarshal to map
	if rv.Kind() == reflect.Struct {
		data, err := json.Marshal(v)
		if err == nil {
			var result map[string]interface{}
			if err := json.Unmarshal(data, &result); err == nil {
				return result
			}
		}
	}

	// For other types, create a simple map with the value
	return map[string]interface{}{
		"value": v,
	}
}

// getCallerInfo returns the file and line number of the caller
func getCallerInfo(skip int) (string, int) {
	_, file, line, ok := runtime.Caller(skip)
	if !ok {
		return "unknown", 0
	}

	// Extract just the filename from the full path
	parts := strings.Split(file, "/")
	if len(parts) > 0 {
		file = parts[len(parts)-1]
	}

	return file, line
}
