package options

type TimeFormat string

const (
	TimeFormatRFC3339     TimeFormat = "RFC3339"
	TimeFormatRFC3339Nano TimeFormat = "RFC3339Nano"
	TimeFormatKitchen     TimeFormat = "Kitchen"
	TimeFormatStamp       TimeFormat = "Stamp"
	TimeFormatCustom      TimeFormat = "Custom"
)

type LogOptions struct {
	Prefix          string     `json:"prefix"`           // Prefix string in log output
	ReportCaller    bool       `json:"report_caller"`    // Include caller info in logs
	ReportTimestamp bool       `json:"report_timestamp"` // Include timestamp in logs
	TimeFormatKey   TimeFormat `json:"time_format_key"`  // Named time format enum
	CustomFormat    string     `json:"custom_format"`    // Custom format if TimeFormatKey == TimeFormatCustom
	CallerOffset    int        `json:"caller_offset"`    // Adjust call depth for correct file/line display
}

type LoggingOptions struct {
	Log LogOptions `json:"log"` // Log options
}
